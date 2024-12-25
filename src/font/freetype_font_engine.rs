use std::{
    collections::HashMap,
    ffi::OsStr,
    ops::Add,
    path::{Path, PathBuf},
};

use glam::Vec2;
use walkdir::WalkDir;

use crate::{
    config::ENABLE_SUBPIXEL_RENDERING,
    font::font_face::GlyphMetadata,
    image::{image::Image, image_format::ImageFormat, image_manager::ImageManager},
    rectangle::Rectangle,
};

use super::{
    font_engine::{FontEngine, LaidOutGlyph, TextLayoutOptions},
    font_face::{FontFace, FontFaceUsageFlags, GlyphAtlasMetadata, GlyphPixelMode},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontKey {
    path: String,
    font_size: u32,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FontFileKey {
    font_family: String,
    font_weight: FontWeight,
    font_style: FontStyle,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl FontWeight {
    fn value(&self) -> u32 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Regular => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
        }
    }

    fn from_value(value: u32) -> Option<FontWeight> {
        match value {
            100 => Some(FontWeight::Thin),
            200 => Some(FontWeight::ExtraLight),
            300 => Some(FontWeight::Light),
            400 => Some(FontWeight::Regular),
            500 => Some(FontWeight::Medium),
            600 => Some(FontWeight::SemiBold),
            700 => Some(FontWeight::Bold),
            800 => Some(FontWeight::ExtraBold),
            900 => Some(FontWeight::Black),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FontStyle {
    Regular,
    Italic,
}

struct GlyphBearings {
    bearing_left: i32,
    bearing_top: i32,
}

pub struct FreetypeFontEngine {
    ft_lib: freetype::Library,
    font_dir_path: String,
    faces: HashMap<FontKey, FontFace>,
    font_files: HashMap<FontFileKey, PathBuf>,
}

impl FontEngine for FreetypeFontEngine {
    fn update(&mut self, image_manager: &mut ImageManager) {
        for (_, face) in &mut self.faces {
            if face.usage_flags.contains(FontFaceUsageFlags::SUBPIXEL) && face.subpixel_atlas_metadata.is_none() {
                face.subpixel_atlas_metadata = Some(create_font_atlas(face, GlyphPixelMode::Subpixel, image_manager));
            }
            if face.usage_flags.contains(FontFaceUsageFlags::GRAYSCALE) && face.grayscale_atlas_metadata.is_none() {
                face.grayscale_atlas_metadata = Some(create_font_atlas(face, GlyphPixelMode::Grayscale, image_manager));
            }
            if !face.usage_flags.is_empty() && face.ft_face.has_color() && face.color_atlas_metadata.is_none() {
                face.color_atlas_metadata = Some(create_font_atlas(face, GlyphPixelMode::Color, image_manager));
            }
        }
    }

    fn lay_out_text(&mut self, text: &str, options: &TextLayoutOptions, mut f: impl FnMut(&LaidOutGlyph)) -> Vec2 {
        let face: &mut FontFace = self.get_or_create_font_face(options.font_family, options.font_size as u32);

        let mut pen = Vec2::ZERO;
        let mut max_computed_line_width: f32 = 0.0;

        for c in text.chars() {
            if c == '\n' {
                max_computed_line_width = max_computed_line_width.max(pen.x);
                pen.x = 0.0;
                pen.y -= options.line_height;
                continue;
            }

            // glyph_index 0 corresponds to ".notdef" glyph (which is sometimes transparent).
            // TODO: use fallback font.
            let advance = face.get_glyph_advance(c).unwrap_or(0);

            if pen.x + advance as f32 > options.max_line_width {
                max_computed_line_width = max_computed_line_width.max(pen.x);
                pen.x = 0.0;
                pen.y -= options.line_height;
            }

            if let Some(glyph_uv_data) = face.get_glyph_uv_data(c, ENABLE_SUBPIXEL_RENDERING) {
                let metadata = glyph_uv_data.metadata;

                let position = pen
                    + Vec2::new(
                        metadata.bearing_left as f32,
                        metadata.bearing_bottom as f32 - options.line_height,
                    );

                let size = Vec2::new(
                    (metadata.bearing_right - metadata.bearing_left) as f32,
                    (metadata.bearing_top - metadata.bearing_bottom) as f32,
                );

                let laid_out_glyph = LaidOutGlyph {
                    position,
                    size,
                    atlas_uv_rectangle: metadata.uv,
                    image_id: glyph_uv_data.image_id,
                    pixel_mode: glyph_uv_data.pixel_mode,
                };

                f(&laid_out_glyph);
            }

            pen.x += advance as f32;
        }

        max_computed_line_width = max_computed_line_width.max(pen.x);

        let dimensions = Vec2::new(
            max_computed_line_width,
            pen.y.abs() + options.line_height - Self::estimate_descender(face, options.font_size),
        );

        dimensions
    }
}

impl FreetypeFontEngine {
    pub fn new(font_dir_path: impl AsRef<OsStr>) -> Self {
        let ft_lib = freetype::Library::init().unwrap();
        ft_lib
            .set_lcd_filter(freetype::LcdFilter::LcdFilterDefault)
            .expect("TODO");

        let mut s = Self {
            ft_lib,
            font_dir_path: font_dir_path.as_ref().to_string_lossy().into_owned(),
            faces: HashMap::new(),
            font_files: HashMap::new(),
        };

        s.discover_fonts();

        s
    }

    fn discover_fonts(&mut self) {
        log::trace!("discovering fonts at {}", self.font_dir_path);

        for ttf_entry in WalkDir::new(&self.font_dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|e| e.to_ascii_lowercase() == "ttf"))
        {
            let font_path = ttf_entry.path();

            let Ok(ft_face) = self.ft_lib.new_face(font_path, 0) else {
                log::error!("{}: failed to load face 0", font_path.display());
                continue;
            };

            let Some(font_family) = ft_face.family_name() else {
                log::error!("{}: font family name missing", font_path.display());
                continue;
            };

            let font_weight = Self::get_font_weight(font_path, &ft_face);
            let font_style = Self::get_font_style(&ft_face);

            let font_file_key = FontFileKey {
                font_family: font_family.to_lowercase(),
                font_weight,
                font_style,
            };

            if let Some(font_file_path) = self.font_files.get(&font_file_key) {
                log::warn!(
                    "{}: font file has same properties as {}. Skipping.",
                    font_path.display(),
                    font_file_path.display()
                );
                continue;
            }

            log::info!(
                "indexed {}: F:'{}' W:{:#?} S:{:#?}",
                font_path.display(),
                font_family,
                font_weight,
                font_style,
            );
            self.font_files.insert(font_file_key, font_path.to_path_buf());
        }
    }

    fn get_font_weight(font_path: &Path, ft_face: &freetype::face::Face) -> FontWeight {
        match Self::get_os2_table(ft_face) {
            Some(os2_table) => {
                let weight_class = os2_table.usWeightClass;
                if let Some(font_weight) = FontWeight::from_value(weight_class as u32) {
                    font_weight
                } else {
                    log::warn!(
                        "{}: unknown weight class {}. Using FontWeight::Regular.",
                        font_path.display(),
                        weight_class
                    );
                    FontWeight::Regular
                }
            }
            None => {
                let style_flags = ft_face.style_flags();
                if style_flags.contains(freetype::face::StyleFlag::BOLD) {
                    FontWeight::Bold
                } else {
                    FontWeight::Regular
                }
            }
        }
    }

    fn get_font_style(ft_face: &freetype::face::Face) -> FontStyle {
        const ITALIC_FLAG: u16 = 1 << 0;

        let is_italic = match Self::get_os2_table(ft_face) {
            Some(os2_table) => {
                let fs_selection = os2_table.fsSelection;
                (fs_selection & ITALIC_FLAG) != 0
            }
            None => {
                let style_flags = ft_face.style_flags();
                style_flags.contains(freetype::face::StyleFlag::ITALIC)
            }
        };

        if is_italic {
            FontStyle::Italic
        } else {
            FontStyle::Regular
        }
    }

    fn get_os2_table(ft_face: &freetype::face::Face) -> Option<&freetype::freetype_sys::TT_OS2> {
        unsafe {
            let os2_table = freetype::freetype_sys::FT_Get_Sfnt_Table(
                ft_face.raw() as *const _ as *mut _,
                freetype::freetype_sys::ft_sfnt_os2,
            );

            if !os2_table.is_null() {
                let os2_table = &*(os2_table as *const freetype::freetype_sys::TT_OS2);

                Some(os2_table)
            } else {
                None
            }
        }
    }

    // Not quite perfect (the round() doesn't always work), but good enough.
    fn estimate_descender(face: &FontFace, font_size: f32) -> f32 {
        let unscaled_descender = face.ft_face.descender() as f32 / 64.0;
        let unscaled_height = (face.ft_face.ascender() - face.ft_face.descender()) as f32 / 64.0;
        let height_quotient = font_size / unscaled_height;
        let scaled_descender = (height_quotient * unscaled_descender).round();

        scaled_descender
    }

    fn get_or_create_font_face(&mut self, font_family: &str, font_size: u32) -> &mut FontFace {
        let font_file_path = self
            .font_files
            .get(&FontFileKey {
                font_family: font_family.to_ascii_lowercase(),
                font_weight: FontWeight::Regular,
                font_style: FontStyle::Regular,
            })
            .expect("TODO");

        self.faces
            .entry(FontKey {
                path: font_file_path.to_str().unwrap().to_string(),
                font_size,
            })
            .or_insert_with(|| {
                let ft_face = self.ft_lib.new_face(font_file_path, 0).unwrap();
                ft_face.set_pixel_sizes(0, font_size).expect("TODO");
                let face = FontFace::new(ft_face);

                face
            })
    }
}

fn create_font_atlas(
    face: &FontFace,
    glyph_pixel_mode: GlyphPixelMode,
    image_manager: &mut ImageManager,
) -> GlyphAtlasMetadata {
    let ft_glyph_pixel_mode: freetype::bitmap::PixelMode = match glyph_pixel_mode {
        GlyphPixelMode::Grayscale => freetype::bitmap::PixelMode::Gray,
        GlyphPixelMode::Subpixel => freetype::bitmap::PixelMode::Lcd,
        GlyphPixelMode::Color => freetype::bitmap::PixelMode::Bgra,
    };

    let extra_load_flags = match glyph_pixel_mode {
        GlyphPixelMode::Grayscale => freetype::face::LoadFlag::empty(),
        GlyphPixelMode::Subpixel => freetype::face::LoadFlag::TARGET_LCD,
        GlyphPixelMode::Color => freetype::face::LoadFlag::COLOR,
    };

    let glyph_width_modulo = match glyph_pixel_mode {
        GlyphPixelMode::Subpixel => 3,
        _ => 1,
    };

    let glyph_image_format = match glyph_pixel_mode {
        GlyphPixelMode::Grayscale => ImageFormat::R8Unorm,
        GlyphPixelMode::Subpixel => ImageFormat::Rgb8Unorm,
        GlyphPixelMode::Color => ImageFormat::Bgra8Unorm,
    };

    let atlas_image_format = match glyph_pixel_mode {
        GlyphPixelMode::Grayscale => ImageFormat::R8Unorm,
        GlyphPixelMode::Subpixel => ImageFormat::Rgba8Unorm,
        GlyphPixelMode::Color => ImageFormat::Rgba8Unorm,
    };

    let mut glyph_indices = vec![];
    let mut glyph_images = vec![];
    let mut glyph_bearings = vec![];

    for g in 0..face.ft_face.num_glyphs() as u32 {
        face.ft_face
            .load_glyph(g, freetype::face::LoadFlag::RENDER | extra_load_flags)
            .expect("TODO");

        let glyph = face.ft_face.glyph();
        let bitmap = glyph.bitmap();

        if bitmap.pixel_mode() != Ok(ft_glyph_pixel_mode) {
            continue;
        }

        assert_eq!(glyph.bitmap().width() as u32 % glyph_width_modulo, 0);

        glyph_indices.push(g);

        glyph_images.push(Image::from_data(
            glyph.bitmap().buffer().to_owned(),
            glyph.bitmap().width() as u32 / glyph_width_modulo,
            glyph.bitmap().rows() as u32,
            glyph.bitmap().pitch() as u32, // TODO: handle negative pitch
            glyph_image_format,
        ));

        glyph_bearings.push(GlyphBearings {
            bearing_left: glyph.bitmap_left(),
            bearing_top: glyph.bitmap_top(),
        });
    }

    let (atlas, positions) = create_atlas(&glyph_images, atlas_image_format);
    assert_eq!(positions.len(), glyph_images.len());

    let mut glyph_metadata = Vec::with_capacity(glyph_images.len());
    for (i, &(x, y)) in positions.iter().enumerate() {
        let image = &glyph_images[i];
        let bearings = &glyph_bearings[i];

        glyph_metadata.push(GlyphMetadata {
            uv: Rectangle {
                left: x as f32 / atlas.width() as f32,
                bottom: y as f32 / atlas.height() as f32,
                right: (x + image.width()) as f32 / atlas.width() as f32,
                top: (y + image.height()) as f32 / atlas.height() as f32,
            },
            bearing_left: bearings.bearing_left,
            bearing_bottom: bearings.bearing_top - image.height() as i32,
            bearing_right: bearings.bearing_left + image.width() as i32,
            bearing_top: bearings.bearing_top,
        });
    }

    let mut glyph_indices_table = vec![u32::MAX; face.ft_face.num_glyphs() as usize];
    for (i, &g) in glyph_indices.iter().enumerate() {
        glyph_indices_table[g as usize] = i as u32;
    }

    GlyphAtlasMetadata {
        image_id: image_manager.add_image(atlas),
        glyph_metadata,
        glyph_indices_table,
    }
}

fn create_atlas(images: &[Image], atlas_image_format: ImageFormat) -> (Image, Vec<(u32, u32)>) {
    let sorted_indices = {
        let mut indices = (0..images.len() as u32).collect::<Vec<u32>>();
        indices.sort_by_key(|&i| -(images[i as usize].width() as i32));
        indices.sort_by_key(|&i| -(images[i as usize].height() as i32));
        indices
    };

    let (mut atlas, sorted_positions) = {
        let total_width = images.iter().map(|i| i.width()).sum::<u32>() as f32;
        let max_width = images.iter().map(|i| i.width()).max().unwrap_or(0);
        let atlas_width = (total_width.sqrt().log2().ceil().add(3.0).exp2() as u32).max(max_width);

        let positions = run_packing_algorithm(atlas_width, images, &sorted_indices);
        let atlas_height = positions
            .iter()
            .enumerate()
            .map(|(i, (_, y))| y + images[i].height())
            .max()
            .unwrap_or(0);

        let atlas = Image::new_empty(atlas_width, atlas_height, atlas_image_format);

        (atlas, positions)
    };

    let mut positions = vec![(u32::MAX, u32::MAX); images.len()];
    for (&(x, y), &i) in sorted_positions.iter().zip(&sorted_indices) {
        copy_to_atlas_at(&images[i as usize], &mut atlas, x, y);
        positions[i as usize] = (x, y);
    }

    for &(x, y) in &positions {
        debug_assert_ne!(x, u32::MAX);
        debug_assert_ne!(y, u32::MAX);
    }

    (atlas, positions)
}

fn run_packing_algorithm(width: u32, images: &[Image], sorted_indices: &[u32]) -> Vec<(u32, u32)> {
    assert_eq!(images.len(), sorted_indices.len());

    let mut cursor_x: u32 = width;
    let mut cursor_y: u32 = u32::MAX;
    let mut next_cursor_y = 0;

    let mut positions = Vec::with_capacity(images.len());
    for &i in sorted_indices {
        let image = &images[i as usize];

        if cursor_x + image.width() > width {
            cursor_x = 0;
            cursor_y = next_cursor_y;
            next_cursor_y += image.height();
        }

        positions.push((cursor_x, cursor_y));

        cursor_x += image.width();
    }

    positions
}

fn copy_to_atlas_at(glyph_image: &Image, atlas_image: &mut Image, dst_left: u32, dst_bottom: u32) {
    match glyph_image.format() {
        ImageFormat::R8Unorm => {
            assert_eq!(atlas_image.format(), ImageFormat::R8Unorm);

            for y in 0..glyph_image.height() {
                for x in 0..glyph_image.width() {
                    let r = glyph_image.get_r(x, y);
                    let flipped_y = glyph_image.height() - 1 - y; // TODO: handle negative pitch.

                    atlas_image.set_r(dst_left + x, dst_bottom + flipped_y, r);
                }
            }
        }
        ImageFormat::Rgb8Unorm => {
            assert_eq!(atlas_image.format(), ImageFormat::Rgba8Unorm);

            for y in 0..glyph_image.height() {
                for x in 0..glyph_image.width() {
                    let rgb = glyph_image.get_rgb(x, y);
                    let flipped_y = glyph_image.height() - 1 - y; // TODO: handle negative pitch.

                    atlas_image.set_rgba(dst_left + x, dst_bottom + flipped_y, [rgb[0], rgb[1], rgb[2], u8::MAX]);
                }
            }
        }
        ImageFormat::Bgra8Unorm => {
            assert_eq!(atlas_image.format(), ImageFormat::Rgba8Unorm);

            for y in 0..glyph_image.height() {
                for x in 0..glyph_image.width() {
                    let bgra = glyph_image.get_bgra(x, y);
                    let flipped_y = glyph_image.height() - 1 - y; // TODO: handle negative pitch.

                    atlas_image.set_rgba(
                        dst_left + x,
                        dst_bottom + flipped_y,
                        [bgra[2], bgra[1], bgra[0], bgra[3]],
                    );
                }
            }
        }
        ImageFormat::Rgba8Unorm => panic!(),
    }
}
