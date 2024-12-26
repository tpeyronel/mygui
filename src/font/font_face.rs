use std::{collections::HashMap, ops::Deref};

use bitflags::bitflags;

use crate::{image::image_manager::ImageId, rectangle::Rectangle};

pub struct FontFace {
    pub ft_face: freetype::Face,
    pub advances: Vec<i32>,      // Maps glyph indices to their advances.
    charmap: HashMap<char, u32>, // Maps chars to glyph indices.
    pub usage_flags: FontFaceUsageFlags,
    pub grayscale_atlas_metadata: Option<GlyphAtlasMetadata>,
    pub subpixel_atlas_metadata: Option<GlyphAtlasMetadata>,
    pub color_atlas_metadata: Option<GlyphAtlasMetadata>,
}

impl FontFace {
    pub fn new(ft_face: freetype::Face) -> Self {
        let advances = Self::load_advances(&ft_face);
        let charmap = Self::load_charmap(&ft_face);

        Self {
            ft_face,
            advances,
            charmap,
            usage_flags: FontFaceUsageFlags::empty(),
            grayscale_atlas_metadata: None,
            subpixel_atlas_metadata: None,
            color_atlas_metadata: None,
        }
    }

    fn load_advances(ft_face: &freetype::Face) -> Vec<i32> {
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

    fn get_glyph_index(&mut self, c: char) -> Option<usize> {
        self.charmap.get(&c).map(|&g| g as usize)
    }

    pub fn get_glyph_advance(&mut self, c: char) -> Option<i32> {
        self.get_glyph_index(c).map(|g| self.advances[g])
    }

    pub fn get_glyph_uv_data(&mut self, c: char, use_subpixel_rendering: bool) -> Option<GlyphUvData> {
        self.usage_flags |= if use_subpixel_rendering {
            FontFaceUsageFlags::SUBPIXEL
        } else {
            FontFaceUsageFlags::GRAYSCALE
        };

        let g = self.get_glyph_index(c)? as u32;

        if let Some(metadata) = self
            .color_atlas_metadata
            .as_ref()
            .map(|a| a.get_glyph_metadata(g))
            .flatten()
        {
            Some(GlyphUvData {
                metadata,
                image_id: self.color_atlas_metadata.as_ref().unwrap().image_id,
                pixel_mode: GlyphPixelMode::Color,
            })
        } else if use_subpixel_rendering {
            self.subpixel_atlas_metadata
                .as_ref()
                .map(|a| a.get_glyph_metadata(g))
                .flatten()
                .map(|metadata| GlyphUvData {
                    metadata,
                    image_id: self.subpixel_atlas_metadata.as_ref().unwrap().image_id,
                    pixel_mode: GlyphPixelMode::Subpixel,
                })
        } else {
            self.grayscale_atlas_metadata
                .as_ref()
                .map(|a| a.get_glyph_metadata(g))
                .flatten()
                .map(|metadata| GlyphUvData {
                    metadata,
                    image_id: self.grayscale_atlas_metadata.as_ref().unwrap().image_id,
                    pixel_mode: GlyphPixelMode::Grayscale,
                })
        }
    }
}

pub struct GlyphData {
    advance: i32,
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

pub enum FontFaceJob {}

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
