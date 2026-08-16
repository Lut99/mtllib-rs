//  LIB.rs
//    by Lut99
//
//  Description:
//!   Parser for Wavefront's `.mtllib` format.
//

// Submodules
#[cfg(feature = "io")]
mod io;

// Imports
use std::collections::HashMap;
use std::path::PathBuf;


/***** AUXILLARY *****/
/// Defines how a color looks like.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    /// Red color channel, 0.0 - 1.0.
    pub r: f64,
    /// Green color channel, 0.0 - 1.0.
    pub g: f64,
    /// Blue color channel, 0.0 - 1.0.
    pub b: f64,
}





/***** LIBRARY *****/
/// Defines a material file.
#[derive(Clone, Debug)]
pub struct Mtl {
    /// A list of materials, mapped by name.
    pub mtls: HashMap<String, Material>,
}



/// Defines a single material.
#[derive(Clone, Debug)]
pub struct Material {
    // Coloring
    /// Defines the ambient color of a material.
    pub color_ambient:   Option<Color>,
    /// Defines the diffuse color of a material.
    pub color_diffuse:   Option<Color>,
    /// Defines the specular color of a material.
    pub color_specular:  Option<Color>,
    /// If there is a specular color, then this is a weight for it.
    pub weight_specular: Option<f64>,

    // Transparency
    /// Defines the transparency level, 0.0 - 1.0.
    pub transparency: Option<f64>,
    /// Defines the transmission filter for transparent objects.
    pub transmission_filter_color: Option<TFC>,
    /// Refraction index if transparent.
    pub refraction_index: Option<f64>,

    // Textures
    /// A map for the ambient lighting texture.
    pub map_color_ambient: Option<(TextureOptions, PathBuf)>,
    /// A map for the diffuse lighting texture.
    pub map_color_diffuse: Option<(TextureOptions, PathBuf)>,
    /// A map for the specular lighting texture.
    pub map_color_specular: Option<(TextureOptions, PathBuf)>,
    /// A map for the alpha channel.
    pub map_alpha: Option<(TextureOptions, PathBuf)>,
    /// A map for the bump channel.
    pub map_bump: Option<(TextureOptions, PathBuf)>,
    /// A map for the displacement.
    pub map_disp: Option<(TextureOptions, PathBuf)>,
    /// A map for the stencil decal.
    pub map_stencil: Option<(TextureOptions, PathBuf)>,
    /// A map for reflection maps.
    pub map_refl: Option<(ReflectionOptions, PathBuf)>,

    // Miscellaneous
    /// Lighting model.
    pub model: Option<IlluminationModel>,
}
impl Default for Material {
    #[inline]
    fn default() -> Self {
        Self {
            color_ambient: None,
            color_diffuse: None,
            color_specular: None,
            weight_specular: None,
            transparency: None,
            transmission_filter_color: None,
            refraction_index: None,
            map_color_ambient: None,
            map_color_diffuse: None,
            map_color_specular: None,
            map_alpha: None,
            map_bump: None,
            map_disp: None,
            map_refl: None,
            map_stencil: None,
            model: None,
        }
    }
}



/// Defines the possible Transmission Filter Color options.
#[derive(Clone, Copy, Debug)]
pub enum TFC {
    RGB(Color),
}



/// Options for texture maps.
#[derive(Clone, Copy, Debug)]
pub struct TextureOptions {
    /// Horizontal texture blending.
    pub blendu: bool,
    /// Vertical texture blending.
    pub blendv: bool,
    /// Boosts mipmap sharpness.
    pub boost: Option<f64>,
    /// Boost for texture map values (brightness, contrast)
    pub mm: (f64, f64),
    /// Origin offset.
    pub o: (f64, f64, f64),
    /// Scale.
    pub s: (f64, f64, f64),
    /// Turbulance.
    pub t: (f64, f64, f64),
    /// Texture resolution to create.
    pub texres: Option<(u32, u32)>,
    /// Only render texels in the 0.0-1.0 range.
    pub clamp: bool,
    /// Bump multiplier (for bump maps only).
    pub bm: Option<f64>,
    /// The image channel used to create a scalar/bump texture.
    pub imfchan: Option<Channel>,
}
impl Default for TextureOptions {
    #[inline]
    fn default() -> Self {
        Self {
            blendu: true,
            blendv: true,
            boost: None,
            mm: (0.0, 1.0),
            o: (0.0, 0.0, 0.0),
            s: (1.0, 1.0, 1.0),
            t: (0.0, 0.0, 0.0),
            texres: None,
            clamp: false,
            bm: None,
            imfchan: None,
        }
    }
}



/// Options for reflection maps.
#[derive(Clone, Copy, Debug)]
pub struct ReflectionOptions {
    /// The nested, normal texture options.
    pub opts: TextureOptions,
    /// The type of the reflection map.
    pub ty:   Option<ReflectionType>,
}
impl Default for ReflectionOptions {
    #[inline]
    fn default() -> Self { Self { opts: Default::default(), ty: None } }
}



/// Defines possible color channels.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Channel {
    /// Red color.
    R,
    /// Green color.
    G,
    /// Blue color.
    B,
    /// Matte.
    M,
    /// Luminance.
    L,
    /// Z-depth.
    Z,
}



/// Defines the possible reflection map types.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReflectionType {
    /// It's a spherical reflection.
    Sphere,
    /// It's a cubical reflection, top face.
    CubeTop,
    /// It's a cubical reflection, bottom face.
    CubeBottom,
    /// It's a cubical reflection, front face.
    CubeFront,
    /// It's a cubical reflection, back face.
    CubeBack,
    /// It's a cubical reflection, left face.
    CubeLeft,
    /// It's a cubical reflection, right face.
    CubeRight,
}



/// Defines the recognized illumination models.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IlluminationModel {
    /// 0. Color on and Ambient off
    Off,
    /// 1. Color on and Ambient on
    Ambient,
    /// 2. Highlight on
    Highlight,
    /// 3. Reflection on and Ray trace on
    ReflectionRaytrace,
    /// 4. Transparency: Glass on, Reflection: Ray trace on
    TransparencyGlassRaytrace,
    /// 5. Reflection: Fresnel on and Ray trace on
    ReflectionFresnelRaytrace,
    /// 6. Transparency: Refraction on, Reflection: Fresnel off and Ray trace on
    TransparencyRefractionNoFresnelRaytrace,
    /// 7. Transparency: Refraction on, Reflection: Fresnel on and Ray trace on
    TransparencyRefractionFresnelRaytrace,
    /// 8. Reflection on and Ray trace off
    ReflectionNoRaytrace,
    /// 9. Transparency: Glass on, Reflection: Ray trace off
    GlassNoRaytrace,
    /// 10. Casts shadows onto invisible surfaces
    ShadowsInvisibleSurfaces,
}
