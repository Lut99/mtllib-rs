//  IO.rs
//    by Lut99
//
//  Description:
//!   Implements reading for the [`Mtl`] type.
//

use std::collections::HashMap;
use std::io::Read;

use thiserror::Error;

use crate::{Material, Mtl};


/***** ERRORS *****/
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read from reader")]
    Read(#[source] std::io::Error),
    #[error("Expected a keyword at position {i}")]
    Keyword { i: u64 },
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
    /// That is, the remainder after a whitespace 
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
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, Error> {
        let mut mtls: HashMap<String, Material> = HashMap::new();
        let mut parser = Parser::new(reader);
        loop {
            // Get a keyword first
            let keyword = match parser.keyword()? {
                Some(kw) => kw,
                None => break,
            };
            match keyword.as_slice() {
                b"newmtl" => {},
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
