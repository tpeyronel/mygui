use std::{u32, u8};

use crate::image::{
    image::Image,
    image_format::ImageFormat,
    image_manager::{ImageId, ImageManager},
};

#[derive(Debug)]
pub struct FontAtlas {
    glyphs: Vec<AtlasGlyph>,
    image_id: ImageId,
}

#[derive(Debug)]
pub struct AtlasGlyph {
    pub glyph_index: u32,
    pub left: u32,           // Atlas coordinates. Inclusive.
    pub bottom: u32,         // Atlas coordinates. Inclusive.
    pub right: u32,          // Atlas coordinates. Exclusive.
    pub top: u32,            // Atlas coordinates. Exclusive.
    pub bearing_left: i32,   // "glyph.bbox.left"
    pub bearing_bottom: i32, // "glyph.bbox.bottom"
    pub bearing_right: i32,  // "glyph.bbox.right"
    pub bearing_top: i32,    // "glyph.bbox.top"
    pub advance: i32,
}

impl FontAtlas {
    pub fn new(face: &freetype::Face, image_manager: &mut ImageManager) -> Self {
        // TODO: choose dimensions
        let width = 4096;
        let height = 4096;

        let mut load_flags = freetype::face::LoadFlag::RENDER;
        load_flags |= freetype::face::LoadFlag::TARGET_LCD;

        let mut glyphs = vec![];
        for g in 0..face.num_glyphs() as u32 {
            face.load_glyph(g, load_flags).expect("TODO");
            let glyph = face.glyph();

            let glyph_image = Image::from_data(
                glyph.bitmap().buffer().to_owned(),
                (glyph.bitmap().width() / 3) as u32,
                glyph.bitmap().rows() as u32,
                glyph.bitmap().pitch() as u32, // TODO: handle negative
                ImageFormat::Rgb8Unorm,
            );

            let glyph = Glyph {
                image: glyph_image,
                bearing_left: glyph.bitmap_left(),
                bearing_top: glyph.bitmap_top(),
                advance: glyph.advance().x / 64,
            };

            glyphs.push(glyph);
        }

        let mut glyph_indices = (0..glyphs.len() as u32).collect::<Vec<u32>>();
        glyph_indices.sort_by_key(|&i| -(glyphs[i as usize].image.width() as i32));
        glyph_indices.sort_by_key(|&i| -(glyphs[i as usize].image.height() as i32));

        let mut atlas_glyphs = vec![];
        let mut atlas_image = Image::new_empty(width, height, ImageFormat::Rgba8Unorm);
        let mut cursor_x: u32 = width;
        let mut cursor_y: u32 = u32::MAX;
        let mut next_cursor_y = 0;

        for glyph_index in glyph_indices {
            let glyph = &glyphs[glyph_index as usize];

            if cursor_x + glyph.image.width() > width {
                cursor_x = 0;
                cursor_y = next_cursor_y;
                next_cursor_y += glyph.image.height();
            }

            copy_to_atlas(&glyph.image, &mut atlas_image, cursor_x, cursor_y);

            let atlas_glyph = AtlasGlyph {
                glyph_index,
                left: cursor_x,
                bottom: cursor_y,
                right: cursor_x + glyph.image.width(),
                top: cursor_y + glyph.image.height(),
                bearing_left: glyph.bearing_left,
                bearing_bottom: glyph.bearing_top - glyph.image.height() as i32,
                bearing_right: glyph.bearing_left + glyph.image.width() as i32,
                bearing_top: glyph.bearing_top,
                advance: glyph.advance,
            };
            atlas_glyphs.push(atlas_glyph);

            cursor_x += glyph.image.width();
        }

        atlas_glyphs.sort_by_key(|g| g.glyph_index);
        let atlas_image_id = image_manager.add_image(atlas_image);

        Self {
            glyphs: atlas_glyphs,
            image_id: atlas_image_id,
        }
    }

    pub fn image_id(&self) -> ImageId {
        self.image_id
    }

    pub fn get_glyph(&self, glyph_index: u32) -> &AtlasGlyph {
        &self.glyphs[glyph_index as usize]
    }
}

fn copy_to_atlas(glyph_image: &Image, atlas_image: &mut Image, dst_left: u32, dst_bottom: u32) {
    assert_eq!(glyph_image.format(), ImageFormat::Rgb8Unorm);
    assert_eq!(atlas_image.format(), ImageFormat::Rgba8Unorm);

    for y in 0..glyph_image.height() {
        for x in 0..glyph_image.width() {
            let rgb = glyph_image.get_rgb(x, y);
            let flipped_y = glyph_image.height() - 1 - y; // TODO: handle negative pitch.

            atlas_image.set_rgba(dst_left + x, dst_bottom + flipped_y, [rgb[0], rgb[1], rgb[2], u8::MAX]);
        }
    }
}

struct Glyph {
    image: Image,
    bearing_left: i32,
    bearing_top: i32,
    advance: i32,
}
