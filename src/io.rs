//  IO.rs
//    by Lut99
//
//  Description:
//!   Implements reading for the [`Mtl`] type.
//

use std::collections::HashMap;
use std::io::Read;
use std::str::FromStr as _;

use thiserror::Error;

use crate::{Color, IlluminationModel, Material, Mtl};


/***** ERRORS *****/
#[derive(Debug, Error)]
pub enum Error {
    #[error("Expected the dissolve at position {i}")]
    DMissingDissolve { i: u64 },
    #[error("Expected a floating-point number at position {i}")]
    F64 { i: u64 },
    #[error("Failed to read {:?} as valid floating-point number", String::from_utf8_lossy(&raw))]
    F64Value { raw: Vec<u8>, err: std::num::ParseFloatError },
    #[error("Expected the model at position {i}")]
    IllumMissingModel { i: u64 },
    #[error("Failed to read from reader")]
    Read(#[source] std::io::Error),
    #[error("Expected at least 3 color channels, got {got}")]
    KxNotEnoughChannels { i: u64, got: u8 },
    #[error("Expected a keyword at position {i}")]
    Keyword { i: u64 },
    #[error("Expected the name of the material at position {i}")]
    NewmtlMissingName { i: u64 },
    #[error("Attempted to assign a property without stating a material at position {i}")]
    NoNewMtl { i: u64 },
    #[error("Expected the exponent at position {i}")]
    NsMissingExponent { i: u64 },
    #[error("Expected the transparency at position {i}")]
    TrMissingTransparency { i: u64 },
    #[error("Expected an unsigned number at position {i}")]
    U32 { i: u64 },
    #[error("Failed to read {:?} as valid unsigned number", String::from_utf8_lossy(&raw))]
    U32Value { raw: Vec<u8>, err: std::num::ParseIntError },
    #[error("Unknown illumination model {model}")]
    UnknownIlluminationModel { model: u32 },
    #[error("Unknown keyword {:?}", String::from_utf8_lossy(&keyword))]
    UnknownKeyword { keyword: Vec<u8> },
}





/***** PARSER *****/
/// A wrapper around a reader that does what we want.
struct Parser<R> {
    reader: R,
    i:      u64,
}
impl<R> Parser<R> {
    const fn new(reader: R) -> Self { Self { reader, i: 1 } }
}
impl<R: Read> Parser<R> {
    /// Gets anything off the stream.
    ///
    /// # Returns
    /// The index of the next byte and the byte, or [`None`].
    fn next(&mut self) -> Result<Option<(u64, u8)>, Error> {
        let mut b: u8 = 0;
        if self.reader.read(std::slice::from_mut(&mut b)).map_err(Error::Read)? > 0 {
            let i: u64 = self.i;
            self.i += 1;
            Ok(Some((i, b)))
        } else {
            Ok(None)
        }
    }

    /// Gets the next character off the stream.
    ///
    /// This ignores comments & whitespace up to the first other byte.
    ///
    /// # Returns
    /// The byte or [`None`].
    fn byte(&mut self) -> Result<Option<(u64, u8)>, Error> {
        enum State {
            Start,
            Comment,
        }

        let mut state = State::Start;
        while let Some((i, b)) = self.next()? {
            match state {
                State::Start if b == b' ' || b == b'\t' || b == b'\r' || b == b'\n' => continue,
                State::Start if b == b'#' => state = State::Comment,
                State::Start => return Ok(Some((i, b))),

                State::Comment if b == b'\n' => state = State::Start,
                State::Comment => continue,
            }
        }
        Ok(None)
    }

    /// Gets a keyword off the stream.
    ///
    /// This ignores comments & whitespace up to the first other byte.
    fn keyword(&mut self) -> Result<Option<Vec<u8>>, Error> {
        // Get a first byte that's a keyword byte.
        let (i, b): (u64, u8) = match self.byte()? {
            Some(b) => b,
            None => return Ok(None),
        };
        if (b < b'a' || b > b'z') && (b < b'A' || b > b'Z') {
            return Err(Error::Keyword { i });
        }

        // Then only read keyword bytes
        let mut keyword = vec![b];
        while let Some((_, b)) = self.next()? {
            if (b < b'a' || b > b'z') && (b < b'A' || b > b'Z') && b != b'_' {
                return Ok(Some(keyword));
            }
            keyword.push(b);
        }
        Ok(Some(keyword))
    }

    /// Pop a single string off the stream.
    ///
    /// That is, the remainder of the line after a whitespace.
    fn string(&mut self) -> Result<Option<String>, Error> {
        // Get a first byte
        let b: u8 = match self.byte()? {
            Some((_, b)) => b,
            None => return Ok(None),
        };

        // Read until it's a newline
        let mut s = vec![b];
        while let Some((_, b)) = self.next()? {
            if b == b'\n' {
                return Ok(Some(String::from_utf8_lossy(&s).into()));
            }
            s.push(b);
        }
        Ok(Some(String::from_utf8_lossy(&s).into()))
    }

    /// Pop a single (unsigned) integer number off the stream.
    fn u32(&mut self) -> Result<Option<u32>, Error> {
        // Get a first byte in the range
        let (i, b): (u64, u8) = match self.byte()? {
            Some(b) => b,
            None => return Ok(None),
        };
        if b < b'0' || b > b'9' {
            return Err(Error::U32 { i });
        }

        // Read until it's a newline
        let mut raw = vec![b];
        while let Some((_, b)) = self.next()? {
            if b < b'0' || b > b'9' {
                return Ok(Some(u32::from_str(&String::from_utf8_lossy(&raw)).map_err(|err| Error::U32Value { raw, err })?));
            }
            raw.push(b);
        }
        Ok(Some(u32::from_str(&String::from_utf8_lossy(&raw)).map_err(|err| Error::U32Value { raw, err })?))
    }

    /// Pop a single floating-point number off the stream.
    fn f64(&mut self) -> Result<Option<f64>, Error> {
        // Get a first byte in the range
        let (i, b): (u64, u8) = match self.byte()? {
            Some(b) => b,
            None => return Ok(None),
        };
        if (b < b'0' || b > b'9') && b != b'.' {
            return Err(Error::F64 { i });
        }

        // Read until it's a newline
        let mut raw = vec![b];
        while let Some((_, b)) = self.next()? {
            if (b < b'0' || b > b'9') && b != b'.' {
                return Ok(Some(f64::from_str(&String::from_utf8_lossy(&raw)).map_err(|err| Error::F64Value { raw, err })?));
            }
            raw.push(b);
        }
        Ok(Some(f64::from_str(&String::from_utf8_lossy(&raw)).map_err(|err| Error::F64Value { raw, err })?))
    }
}





/***** LIBRARY *****/
impl Mtl {
    /// Attempts to read this Mtl-file from the given reader.
    ///
    /// # Arguments
    /// - `reader`: Some [`Read`]er to read the Mtl file from.
    ///
    /// # Returns
    /// A new Mtl file loaded from the given reader.
    ///
    /// # Errors
    /// This function can fail if the reader's contents were no valid mtl file.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, Error> {
        let mut mtls: HashMap<String, Material> = HashMap::new();
        let mut currentmtl: Option<String> = None;
        let mut parser = Parser::new(reader);
        loop {
            // Get a keyword first
            let keyword = match parser.keyword()? {
                Some(kw) => kw,
                None => return Ok(Self { mtls }),
            };
            match keyword.as_slice() {
                // Meta
                b"newmtl" => {
                    // The name of the keyword
                    let name: String = parser.string()?.ok_or_else(|| Error::NewmtlMissingName { i: parser.i })?;
                    // Add it and as the current material
                    mtls.insert(name.clone(), Material::default());
                    currentmtl = Some(name);
                },

                // Colors
                b"Ka" => {
                    // Three floating-point numbers encoding the colors
                    let (r, g, b): (f64, f64, f64) = (
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 0 })?,
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 1 })?,
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 2 })?,
                    );

                    // Add the colour
                    mtls.get_mut(currentmtl.as_ref().ok_or_else(|| Error::NoNewMtl { i: parser.i })?)
                        .unwrap_or_else(|| panic!("Current material does not exist"))
                        .color_ambient = Some(Color { r, g, b });
                },
                b"Kd" => {
                    // Three floating-point numbers encoding the colors
                    let (r, g, b): (f64, f64, f64) = (
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 0 })?,
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 1 })?,
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 2 })?,
                    );

                    // Add the colour
                    mtls.get_mut(currentmtl.as_ref().ok_or_else(|| Error::NoNewMtl { i: parser.i })?)
                        .unwrap_or_else(|| panic!("Current material does not exist"))
                        .color_diffuse = Some(Color { r, g, b });
                },
                b"Ks" => {
                    // Three floating-point numbers encoding the colors
                    let (r, g, b): (f64, f64, f64) = (
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 0 })?,
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 1 })?,
                        parser.f64()?.ok_or_else(|| Error::KxNotEnoughChannels { i: parser.i, got: 2 })?,
                    );

                    // Add the colour
                    mtls.get_mut(currentmtl.as_ref().ok_or_else(|| Error::NoNewMtl { i: parser.i })?)
                        .unwrap_or_else(|| panic!("Current material does not exist"))
                        .color_specular = Some(Color { r, g, b });
                },
                b"Ns" => {
                    // A single floating-point number
                    let exp: f64 = parser.f64()?.ok_or_else(|| Error::NsMissingExponent { i: parser.i })?;

                    // Add the exponent
                    mtls.get_mut(currentmtl.as_ref().ok_or_else(|| Error::NoNewMtl { i: parser.i })?)
                        .unwrap_or_else(|| panic!("Current material does not exist"))
                        .weight_specular = Some(exp);
                },
                b"d" => {
                    // A single floating-point number
                    let dis: f64 = parser.f64()?.ok_or_else(|| Error::DMissingDissolve { i: parser.i })?;

                    // Add the exponent
                    mtls.get_mut(currentmtl.as_ref().ok_or_else(|| Error::NoNewMtl { i: parser.i })?)
                        .unwrap_or_else(|| panic!("Current material does not exist"))
                        .transparency = Some(1.0 - dis);
                },
                b"Tr" => {
                    // A single floating-point number
                    let tr: f64 = parser.f64()?.ok_or_else(|| Error::TrMissingTransparency { i: parser.i })?;

                    // Add the exponent
                    mtls.get_mut(currentmtl.as_ref().ok_or_else(|| Error::NoNewMtl { i: parser.i })?)
                        .unwrap_or_else(|| panic!("Current material does not exist"))
                        .transparency = Some(tr);
                },

                // Misc
                b"illum" => {
                    // A single unsigned integer
                    let model: u32 = parser.u32()?.ok_or_else(|| Error::IllumMissingModel { i: parser.i })?;
                    let model: IlluminationModel = match model {
                        0 => IlluminationModel::Off,
                        1 => IlluminationModel::Ambient,
                        2 => IlluminationModel::Highlight,
                        3 => IlluminationModel::ReflectionRaytrace,
                        4 => IlluminationModel::TransparencyGlassRaytrace,
                        5 => IlluminationModel::ReflectionFresnelRaytrace,
                        6 => IlluminationModel::TransparencyRefractionNoFresnelRaytrace,
                        7 => IlluminationModel::TransparencyRefractionFresnelRaytrace,
                        8 => IlluminationModel::ReflectionNoRaytrace,
                        9 => IlluminationModel::GlassNoRaytrace,
                        10 => IlluminationModel::ShadowsInvisibleSurfaces,
                        model => return Err(Error::UnknownIlluminationModel { model }),
                    };

                    // Add the exponent
                    mtls.get_mut(currentmtl.as_ref().ok_or_else(|| Error::NoNewMtl { i: parser.i })?)
                        .unwrap_or_else(|| panic!("Current material does not exist"))
                        .model = Some(model);
                },

                // Unknown
                kw => return Err(Error::UnknownKeyword { keyword: kw.into() }),
            }
        }
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_from_reader() {
        // Go over all the .obj files in the tests dir
        let tests_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("mtl");
        for entry in std::fs::read_dir(&tests_dir).unwrap_or_else(|err| panic!("Failed to read tests directory {tests_dir:?}: {err}")) {
            // Keep only the files that are marked as .mtl
            let entry = entry.unwrap_or_else(|err| panic!("Failed to read entry in tests directory {tests_dir:?}: {err}"));
            let entry_path = entry.path();
            let entry_str = entry_path.to_string_lossy();
            if !entry_str.ends_with(".mtl") {
                continue;
            }

            // Report we're opening this one
            println!("Testing {entry_path:?}...");
            let handle = std::fs::File::open(&entry_path).unwrap_or_else(|err| panic!("Failed to open file {entry_path:?}: {err}"));
            let _mtl = Mtl::from_reader(handle).unwrap_or_else(|err| panic!("Failed to parse file {entry_path:?} as .mtl file: {err}"));
        }
    }
}
