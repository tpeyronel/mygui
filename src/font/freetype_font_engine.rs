use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::PathBuf,
};

use glam::Vec2;
use walkdir::WalkDir;

use crate::{config::ENABLE_SUBPIXEL_RENDERING, image::image_manager::ImageManager};

use super::{
    font_engine::{FontEngine, LaidOutGlyph, TextLayoutOptions},
    font_face::FontFace,
    font_style::FontStyle,
    font_weight::FontWeight,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FontFaceDescriptor {
    font_family: String,
    font_weight: FontWeight,
    font_style: FontStyle,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct FontFaceLoadRequest {
    font_face_id: FontFaceId,
    font_size: u32,
    use_subpixel_rendering: bool,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct FontFaceId(usize);

pub struct FreetypeFontEngine {
    ft_lib: freetype::Library,
    font_dir_path: String,
    font_faces: Vec<FontFace>,
    font_faces_map: HashMap<FontFaceDescriptor, FontFaceId>,
    font_files_map: HashMap<FontFaceDescriptor, PathBuf>,
    load_requests: HashSet<FontFaceLoadRequest>,
}

impl FontEngine for FreetypeFontEngine {
    fn update(&mut self, image_manager: &mut ImageManager) {
        for req in &self.load_requests {
            let font_face = &mut self.font_faces[req.font_face_id.0];
            font_face.load_size_data(req.font_size, req.use_subpixel_rendering, image_manager);
        }

        self.load_requests.clear();
    }

    fn lay_out_text(&mut self, text: &str, options: &TextLayoutOptions, mut f: impl FnMut(&LaidOutGlyph)) -> Vec2 {
        let font_size = options.font_size as u32;

        let font_face_id = self.get_or_create_font_face(options.font_family);
        let face = &mut self.font_faces[font_face_id.0];
        face.ensure_size_data(options.font_size as u32);
        let size_data = face.get_size_data(font_size);

        if ENABLE_SUBPIXEL_RENDERING && size_data.subpixel_atlas_metadata.is_none() {
            self.load_requests.insert(FontFaceLoadRequest {
                font_face_id,
                font_size,
                use_subpixel_rendering: true,
            });
        } else if !ENABLE_SUBPIXEL_RENDERING && size_data.grayscale_atlas_metadata.is_none() {
            self.load_requests.insert(FontFaceLoadRequest {
                font_face_id,
                font_size,
                use_subpixel_rendering: false,
            });
        }

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
            let g = face.get_glyph_index(c).unwrap_or(0);

            let advance = size_data.get_glyph_advance(g);

            if pen.x + advance as f32 > options.max_line_width {
                max_computed_line_width = max_computed_line_width.max(pen.x);
                pen.x = 0.0;
                pen.y -= options.line_height;
            }

            if let Ok(glyph_uv_data) = size_data.get_glyph_uv_data(g, ENABLE_SUBPIXEL_RENDERING) {
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
            pen.y.abs() + options.line_height - face.scaled_descender(options.font_size),
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
            font_faces: Vec::new(),
            font_faces_map: HashMap::new(),
            load_requests: HashSet::new(),
            font_files_map: HashMap::new(),
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

            let font_weight = Self::get_font_weight(&ft_face);
            let font_style = Self::get_font_style(&ft_face);

            let font_file_descriptor = FontFaceDescriptor {
                font_family: font_family.to_lowercase(),
                font_weight,
                font_style,
            };

            if let Some(font_file_path) = self.font_files_map.get(&font_file_descriptor) {
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
            self.font_files_map
                .insert(font_file_descriptor, font_path.to_path_buf());
        }
    }

    fn get_font_weight(ft_face: &freetype::face::Face) -> FontWeight {
        match Self::get_os2_table(ft_face) {
            Some(os2_table) => {
                let weight_class = os2_table.usWeightClass;
                FontWeight::new(weight_class as u32)
            }
            None => {
                let style_flags = ft_face.style_flags();
                if style_flags.contains(freetype::face::StyleFlag::BOLD) {
                    FontWeight::BOLD
                } else {
                    FontWeight::REGULAR
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

    fn get_or_create_font_face(&mut self, font_family: &str) -> FontFaceId {
        let font_face_id = *self
            .font_faces_map
            .entry(FontFaceDescriptor {
                font_family: font_family.to_ascii_lowercase(),
                font_weight: FontWeight::REGULAR,
                font_style: FontStyle::Regular,
            })
            .or_insert_with_key(|descriptor| {
                let font_file_path = self.font_files_map.get(descriptor).expect("TODO");
                let font_face = FontFace::from_path(&font_file_path, &self.ft_lib);
                let font_face_id = FontFaceId(self.font_faces.len());
                self.font_faces.push(font_face);
                font_face_id
            });

        font_face_id
    }
}
