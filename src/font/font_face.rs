use std::{collections::HashMap, ops::Add, path::Path};

use bitflags::bitflags;

use crate::{
    image::{
        image::Image,
        image_format::ImageFormat,
        image_manager::{ImageId, ImageManager},
    },
    rectangle::Rectangle,
};

pub struct FontFace {
    ft_face: freetype::Face,
    charmap: HashMap<char, u32>, // Maps chars to glyph indices.
    per_size_data: HashMap<u32, FontPerSizeData>,
}

impl FontFace {
    pub fn from_path(path: &Path, ft_lib: &freetype::Library) -> Self {
        let ft_face = ft_lib.new_face(path, 0).expect("TODO");
        let charmap = Self::load_charmap(&ft_face);

        Self {
            ft_face,
            charmap,
            per_size_data: HashMap::new(),
        }
    }

    pub fn get_glyph_index(&self, c: char) -> Option<u32> {
        self.charmap.get(&c).map(|&g| g)
    }

    pub fn ensure_size_data(&mut self, font_size: u32) {
        self.per_size_data.entry(font_size).or_insert_with(|| {
            FontPerSizeData {
                advances: Self::load_advances(&self.ft_face, font_size),
                grayscale_atlas_metadata: None,
                subpixel_atlas_metadata: None,
                color_atlas_metadata: None,
            }
        });
    }

    pub fn get_size_data(&self, font_size: u32) -> &FontPerSizeData {
        self.per_size_data.get(&font_size).unwrap()
    }

    pub fn load_size_data(&mut self, font_size: u32, use_subpixel_rendering: bool, image_manager: &mut ImageManager) {
        self.ft_face.set_pixel_sizes(0, font_size).expect("TODO");

        let size_data = self
            .per_size_data
            .get_mut(&font_size)
            .expect("ensure_size_data() must have been called before load_size_data()");

        if use_subpixel_rendering {
            if size_data.subpixel_atlas_metadata.is_none() {
                size_data.subpixel_atlas_metadata = Some(create_font_atlas(
                    &self.ft_face,
                    GlyphPixelMode::Subpixel,
                    image_manager,
                ));
            }
        } else {
            if size_data.grayscale_atlas_metadata.is_none() {
                size_data.grayscale_atlas_metadata = Some(create_font_atlas(
                    &self.ft_face,
                    GlyphPixelMode::Grayscale,
                    image_manager,
                ));
            }
        }

        if self.ft_face.has_color() && size_data.color_atlas_metadata.is_none() {
            size_data.color_atlas_metadata =
                Some(create_font_atlas(&self.ft_face, GlyphPixelMode::Color, image_manager));
        }
    }

    pub fn scaled_descender(&self, font_size: f32) -> f32 {
        // Not quite perfect (the round() doesn't always work), but good enough.
        let unscaled_descender = self.ft_face.descender() as f32 / 64.0;
        let unscaled_height = (self.ft_face.ascender() - self.ft_face.descender()) as f32 / 64.0;
        let height_quotient = font_size / unscaled_height;
        let scaled_descender = (height_quotient * unscaled_descender).round();

        scaled_descender
    }

    fn load_advances(ft_face: &freetype::Face, font_size: u32) -> Vec<i32> {
        ft_face.set_pixel_sizes(0, font_size).expect("TODO");

        let mut advances = vec![];

        for g in 0..ft_face.num_glyphs() as u32 {
            ft_face.load_glyph(g, freetype::face::LoadFlag::DEFAULT).expect("TODO");

            let glyph = ft_face.glyph();
            let advance = glyph.advance().x / 64;

            advances.push(advance);
        }

        advances
    }

    fn load_charmap(ft_face: &freetype::Face) -> HashMap<char, u32> {
        let mut charmap = HashMap::new();

        for (c, g) in ft_face.chars() {
            let Some(c) = char::from_u32(c as u32) else {
                log::error!("error loading charmap: invalid char {}", c);
                continue;
            };

            if charmap.contains_key(&c) {
                log::warn!("error loading charmap: char {} already mapped. Skipping.", c);
                continue;
            }

            charmap.insert(c, g.get());
        }

        charmap
    }
}

pub struct FontPerSizeData {
    advances: Vec<i32>, // Maps glyph indices to their advances.
    pub grayscale_atlas_metadata: Option<GlyphAtlasMetadata>,
    pub subpixel_atlas_metadata: Option<GlyphAtlasMetadata>,
    pub color_atlas_metadata: Option<GlyphAtlasMetadata>,
}

impl FontPerSizeData {
    pub fn get_glyph_advance(&self, g: u32) -> i32 {
        self.advances[g as usize]
    }

    pub fn get_glyph_uv_data(&self, g: u32, use_subpixel_rendering: bool) -> Result<GlyphUvData, GetGlyphError> {
        if let Some(metadata) = self
            .color_atlas_metadata
            .as_ref()
            .map(|a| a.get_glyph_metadata(g))
            .flatten()
        {
            Ok(GlyphUvData {
                metadata,
                image_id: self.color_atlas_metadata.as_ref().unwrap().image_id,
                pixel_mode: GlyphPixelMode::Color,
            })
        } else if use_subpixel_rendering {
            let atlas_metadata = self
                .subpixel_atlas_metadata
                .as_ref()
                .ok_or(GetGlyphError::AtlasNotLoaded)?;

            let glyph_metadata = atlas_metadata
                .get_glyph_metadata(g)
                .ok_or(GetGlyphError::GlyphNotMapped)?;

            Ok(GlyphUvData {
                metadata: glyph_metadata,
                image_id: self.subpixel_atlas_metadata.as_ref().unwrap().image_id,
                pixel_mode: GlyphPixelMode::Subpixel,
            })
        } else {
            let atlas_metadata = self
                .grayscale_atlas_metadata
                .as_ref()
                .ok_or(GetGlyphError::AtlasNotLoaded)?;

            let glyph_metadata = atlas_metadata
                .get_glyph_metadata(g)
                .ok_or(GetGlyphError::GlyphNotMapped)?;

            Ok(GlyphUvData {
                metadata: glyph_metadata,
                image_id: self.grayscale_atlas_metadata.as_ref().unwrap().image_id,
                pixel_mode: GlyphPixelMode::Grayscale,
            })
        }
    }
}

pub struct GlyphUvData<'a> {
    pub metadata: &'a GlyphMetadata,
    pub image_id: ImageId,
    pub pixel_mode: GlyphPixelMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphPixelMode {
    Grayscale,
    Subpixel,
    Color, // NOTE: this uses premultiplied alpha.
}

bitflags! {
    #[derive(Debug)]
    pub struct FontFaceUsageFlags: u32 {
        const GRAYSCALE = 1 << 0;
        const SUBPIXEL = 1 << 1;
    }
}

#[derive(Debug)]
pub struct GlyphAtlasMetadata {
    pub image_id: ImageId,
    pub glyph_metadata: Vec<GlyphMetadata>,
    pub glyph_indices_table: Vec<u32>, // Value of u32::MAX indicates that the glyph is not in the atlas.
}

impl GlyphAtlasMetadata {
    pub fn get_glyph_metadata(&self, g: u32) -> Option<&GlyphMetadata> {
        let index = self.glyph_indices_table[g as usize];
        if index == u32::MAX {
            None
        } else {
            Some(&self.glyph_metadata[index as usize])
        }
    }
}

#[derive(Debug)]
pub struct GlyphMetadata {
    pub uv: Rectangle,
    pub bearing_left: i32,   // "glyph.bbox.left"
    pub bearing_bottom: i32, // "glyph.bbox.bottom"
    pub bearing_right: i32,  // "glyph.bbox.right"
    pub bearing_top: i32,    // "glyph.bbox.top"
}

fn create_font_atlas(
    ft_face: &freetype::Face,
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

    for g in 0..ft_face.num_glyphs() as u32 {
        ft_face
            .load_glyph(g, freetype::face::LoadFlag::RENDER | extra_load_flags)
            .expect("TODO");

        let glyph = ft_face.glyph();
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

    let mut glyph_indices_table = vec![u32::MAX; ft_face.num_glyphs() as usize];
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

struct GlyphBearings {
    bearing_left: i32,
    bearing_top: i32,
}

pub enum GetGlyphError {
    AtlasNotLoaded,
    GlyphNotMapped,
}
