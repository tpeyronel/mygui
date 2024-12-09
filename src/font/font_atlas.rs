use std::{ffi::OsStr, u32};

pub struct FontAtlas {
    glyphs: Vec<AtlasGlyph>,
    image: Image,
}

pub struct AtlasGlyph {
    index: u32,
    left: u32,   // Inclusive.
    bottom: u32, // Inclusive.
    top: u32,    // Exclusive.
    right: u32,  // Exclusive.
}

impl FontAtlas {
    pub fn new(font_path: impl AsRef<OsStr>, font_size: u32, ft_lib: &freetype::Library) -> Self {
        let face = ft_lib.new_face(font_path, 0).unwrap();

        let width = 8192;
        let height = 8192;

        face.set_pixel_sizes(0, font_size).expect("TODO");

        let mut glyphs = vec![];
        for g in 0..face.num_glyphs() as u32 {
            face.load_glyph(g, freetype::face::LoadFlag::RENDER).expect("TODO");
            let glyph = face.glyph();

            let glyph_image = Image {
                data: glyph.bitmap().buffer().to_owned(),
                width: glyph.bitmap().width() as u32,
                height: glyph.bitmap().rows() as u32,
                pitch: glyph.bitmap().pitch() as u32,
            };

            let glyph = Glyph {
                image: glyph_image,
                bearing_left: glyph.bitmap_left() as u32,
                bearing_top: glyph.bitmap_top() as u32,
                advance: glyph.advance().x as u32,
            };

            glyphs.push(glyph);
        }

        glyphs.sort_by_key(|g| -(g.image.width as i32));
        glyphs.sort_by_key(|g| -(g.image.height as i32));

        let mut atlas_glyphs = vec![];
        let mut atlas_image = Image::new_empty(width, height);
        let mut cursor_x: u32 = width;
        let mut cursor_y: u32 = u32::MAX;
        let mut next_cursor_y = 0;

        for (glyph_index, glyph) in glyphs.iter().enumerate() {
            if cursor_x + glyph.image.width > width {
                cursor_x = 0;
                cursor_y = next_cursor_y;
                next_cursor_y += glyph.image.height;
            }

            copy_to_atlas(&glyph.image, &mut atlas_image, cursor_x, cursor_y);

            atlas_glyphs.push(AtlasGlyph {
                index: glyph_index as u32,
                left: cursor_x,
                bottom: cursor_y,
                right: cursor_x + glyph.image.width,
                top: cursor_y + glyph.image.height,
            });

            cursor_x += glyph.image.width;
        }

        Self {
            glyphs: atlas_glyphs,
            image: atlas_image,
        }
    }

    pub fn image(&self) -> &Image {
        &self.image
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
    bearing_left: u32,
    bearing_top: u32,
    advance: u32,
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
