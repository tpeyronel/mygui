use std::{ffi::OsStr, u32};

use freetype::Face;
use glam::Vec2;

use crate::vertex::Vertex;

pub struct FontAtlas {
    face: Face,
    glyphs: Vec<AtlasGlyph>,
    image: Image,
}

pub struct AtlasGlyph {
    left: u32,           // Atlas coordinates. Inclusive.
    bottom: u32,         // Atlas coordinates. Inclusive.
    right: u32,          // Atlas coordinates. Exclusive.
    top: u32,            // Atlas coordinates. Exclusive.
    bearing_left: i32,   // "glyph.bbox.left"
    bearing_bottom: i32, // "glyph.bbox.bottom"
    bearing_right: i32,  // "glyph.bbox.right"
    bearing_top: i32,    // "glyph.bbox.top"
    advance: i32,
}

impl FontAtlas {
    pub fn new(font_path: impl AsRef<OsStr>, font_size: u32, ft_lib: &freetype::Library) -> Self {
        let face = ft_lib.new_face(font_path, 0).unwrap();

        // TODO: choose dimensions
        let width = 4096;
        let height = 4096;

        face.set_pixel_sizes(0, font_size).expect("TODO");

        let mut glyphs = vec![];
        for g in 0..face.num_glyphs() as u32 {
            face.load_glyph(g, freetype::face::LoadFlag::RENDER).expect("TODO");
            let glyph = face.glyph();

            let glyph_image = Image {
                data: glyph.bitmap().buffer().to_owned(),
                width: glyph.bitmap().width() as u32,
                height: glyph.bitmap().rows() as u32,
                pitch: glyph.bitmap().pitch() as u32, // TODO: handle negative
            };

            let glyph = Glyph {
                image: glyph_image,
                bearing_left: glyph.bitmap_left(),
                bearing_top: glyph.bitmap_top(),
                advance: glyph.advance().x,
            };

            glyphs.push(glyph);
        }

        let mut glyph_indices = (0..glyphs.len() as u32).collect::<Vec<u32>>();
        glyph_indices.sort_by_key(|&i| -(glyphs[i as usize].image.width as i32));
        glyph_indices.sort_by_key(|&i| -(glyphs[i as usize].image.height as i32));

        let mut atlas_glyphs = vec![];
        let mut atlas_image = Image::new_empty(width, height);
        let mut cursor_x: u32 = width;
        let mut cursor_y: u32 = u32::MAX;
        let mut next_cursor_y = 0;

        for glyph_index in glyph_indices {
            let glyph = &glyphs[glyph_index as usize];

            if cursor_x + glyph.image.width > width {
                cursor_x = 0;
                cursor_y = next_cursor_y;
                next_cursor_y += glyph.image.height;
            }

            copy_to_atlas(&glyph.image, &mut atlas_image, cursor_x, cursor_y);

            atlas_glyphs.push(AtlasGlyph {
                left: cursor_x,
                bottom: cursor_y,
                right: cursor_x + glyph.image.width,
                top: cursor_y + glyph.image.height,
                bearing_left: glyph.bearing_left,
                bearing_bottom: glyph.bearing_top - glyph.image.height as i32,
                bearing_right: glyph.bearing_left + glyph.image.width as i32,
                bearing_top: glyph.bearing_top,
                advance: glyph.advance,
            });

            cursor_x += glyph.image.width;
        }

        Self {
            face,
            glyphs: atlas_glyphs,
            image: atlas_image,
        }
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn get_uv_for_char(&self, c: char) -> Option<(Vec2, Vec2)> {
        let g = self.face.get_char_index(c as usize)?;
        let glyph = &self.glyphs[g as usize];

        return Some((
            Vec2::new(
                glyph.left as f32 / self.image.width as f32,
                glyph.bottom as f32 / self.image.height as f32,
            ),
            Vec2::new(
                glyph.right as f32 / self.image.width as f32,
                glyph.top as f32 / self.image.height as f32,
            ),
        ));
    }

    fn get_uv_for_glyph(&self, glyph: &AtlasGlyph) -> (Vec2, Vec2) {
        return (
            Vec2::new(
                glyph.left as f32 / self.image.width as f32,
                glyph.bottom as f32 / self.image.height as f32,
            ),
            Vec2::new(
                glyph.right as f32 / self.image.width as f32,
                glyph.top as f32 / self.image.height as f32,
            ),
        );
    }

    pub fn get_vertices_for_char_at(&self, c: char, origin: Vec2) -> Option<Vec<Vertex>> {
        let g = self.face.get_char_index(c as usize)?;
        let glyph = &self.glyphs[g as usize];
        let (uv_bl, uv_tr) = self.get_uv_for_glyph(glyph);

        let bl = origin + Vec2::new(glyph.bearing_left as f32, glyph.bearing_bottom as f32);
        let br = origin + Vec2::new(glyph.bearing_right as f32, glyph.bearing_bottom as f32);
        let tr = origin + Vec2::new(glyph.bearing_right as f32, glyph.bearing_top as f32);
        let tl = origin + Vec2::new(glyph.bearing_left as f32, glyph.bearing_top as f32);

        return Some(vec![
            Vertex { pos: bl, uv: uv_bl },
            Vertex {
                pos: br,
                uv: Vec2::new(uv_tr.x, uv_bl.y),
            },
            Vertex { pos: tr, uv: uv_tr },
            Vertex {
                pos: tl,
                uv: Vec2::new(uv_bl.x, uv_tr.y),
            },
        ]);
    }
}

fn copy_to_atlas(glyph_image: &Image, atlas_image: &mut Image, dst_left: u32, dst_bottom: u32) {
    for y in 0..glyph_image.height {
        for x in 0..glyph_image.width {
            let p = glyph_image.get(x, y);
            let flipped_y = glyph_image.height - 1 - y; // TODO: handle negative pitch.
            atlas_image.set(dst_left + x, dst_bottom + flipped_y, p);
        }
    }
}

struct Glyph {
    image: Image,
    bearing_left: i32,
    bearing_top: i32,
    advance: i32,
}

pub struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
    pitch: u32, // Bytes per row. Not necessarily the same as width.
}

impl Image {
    fn new_empty(width: u32, height: u32) -> Self {
        Self {
            data: vec![0u8; (width * height) as usize],
            width,
            height,
            pitch: width,
        }
    }

    fn coords_to_index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);

        (y * self.pitch + x) as usize
    }

    fn get(&self, x: u32, y: u32) -> u8 {
        self.data[self.coords_to_index(x, y)]
    }

    fn set(&mut self, x: u32, y: u32, value: u8) {
        let index = self.coords_to_index(x, y);
        self.data[index] = value;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn pitch(&self) -> u32 {
        self.pitch
    }
}

impl ToString for Image {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for y in 0..self.height {
            let y = self.height - y - 1;
            for x in 0..self.width {
                let c = match self.get(x, y) {
                    0 => ' ',
                    255 => '$',
                    _ => '+',
                };
                s.push(c);
            }
            s.push('\n');
        }
        s
    }
}
