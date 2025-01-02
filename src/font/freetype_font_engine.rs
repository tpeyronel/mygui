use std::{
    collections::{HashMap, HashSet},
    io::{self},
    path::{Path, PathBuf},
};

use glam::Vec2;
use walkdir::WalkDir;

use crate::{config::ENABLE_SUBPIXEL_RENDERING, image::image_manager::ImageManager};

use super::{
    font_engine::{FontEngine, LaidOutGlyph, TextLayout, TextLayoutOptions, TextMap, TextPosition},
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
    font_dir_path: PathBuf,
    font_cache_dir_path: PathBuf,
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

    fn lay_out_text(&mut self, text: &str, options: &TextLayoutOptions) -> TextLayout {
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

        let mut pen_state = PenState::new(
            options.font_size,
            options.line_height,
            face.scaled_descender(options.font_size),
        );
        let mut text_map = TextMap::new();
        text_map.insert(pen_state.text_position, pen_state.line_position());

        let mut glyphs = vec![];
        for c in text.chars() {
            if c == '\n' {
                pen_state.break_line(options.line_height);
                text_map.insert(pen_state.text_position, pen_state.line_position());
                continue;
            }

            // glyph_index 0 corresponds to ".notdef" glyph (which is sometimes transparent).
            // TODO: use fallback font.
            let g = face.get_glyph_index(c).unwrap_or(0);

            let advance = size_data.get_glyph_advance(g);

            if pen_state.x + advance as f32 > options.max_line_width {
                pen_state.break_line(options.line_height);
                text_map.insert(pen_state.text_position, pen_state.line_position());
            }

            if let Ok(glyph_uv_data) = size_data.get_glyph_uv_data(g, ENABLE_SUBPIXEL_RENDERING) {
                let metadata = glyph_uv_data.metadata;

                let position = pen_state.baseline_position()
                    + Vec2::new(metadata.bearing_left as f32, metadata.bearing_bottom as f32);

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

                glyphs.push(laid_out_glyph);
            }

            pen_state.advance(advance as f32);
            text_map.insert(pen_state.text_position, pen_state.line_position());
        }

        let size = Vec2::new(pen_state.max_computed_line_width, -pen_state.line_y);

        TextLayout { size, glyphs, text_map }
    }

    fn on_exit(&mut self, image_manager: &ImageManager) {
        self.save_to_disk(image_manager);
    }
}

impl FreetypeFontEngine {
    pub fn new(
        font_dir_path: impl AsRef<Path>,
        font_cache_dir_path: impl AsRef<Path>,
        image_manager: &mut ImageManager,
    ) -> Self {
        let ft_lib = freetype::Library::init().unwrap();
        ft_lib
            .set_lcd_filter(freetype::LcdFilter::LcdFilterDefault)
            .expect("TODO");

        let font_dir_path = font_dir_path.as_ref().to_path_buf();
        let font_cache_dir_path = font_cache_dir_path.as_ref().to_path_buf();

        let mut s = Self {
            ft_lib,
            font_dir_path,
            font_cache_dir_path,
            font_faces: Vec::new(),
            font_faces_map: HashMap::new(),
            load_requests: HashSet::new(),
            font_files_map: HashMap::new(),
        };

        s.discover_fonts();
        s.load_cache(image_manager).unwrap();

        s
    }

    pub fn load_cache(&mut self, image_manager: &mut ImageManager) -> io::Result<()> {
        for (font_face_descriptor, font_file_path) in &self.font_files_map {
            let mut cache_dir_path = self.font_cache_dir_path.to_path_buf();
            cache_dir_path.push(font_file_path.file_name().unwrap());

            let Ok(font_face) = FontFace::try_from_cache(font_file_path, &cache_dir_path, &self.ft_lib, image_manager)
            else {
                continue;
            };

            let font_face_id = FontFaceId(self.font_faces.len());
            self.font_faces.push(font_face);
            self.font_faces_map.insert(font_face_descriptor.clone(), font_face_id);
        }

        Ok(())
    }

    pub fn save_to_disk(&self, image_manager: &ImageManager) {
        for f in &self.font_faces {
            f.save_to_disk(&self.font_cache_dir_path, &image_manager).unwrap();
        }
    }

    fn discover_fonts(&mut self) {
        log::trace!("discovering fonts at {}", self.font_dir_path.display());

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

struct PenState {
    x: f32,
    baseline_y: f32, // Distance from the top of the text bounds to the *baseline* of the current line, in pixels.
    line_y: f32,     // Distance from the top of the text bounds to the *bottom* of the current line, in pixels.
    text_position: TextPosition,
    max_computed_line_width: f32,
}

impl PenState {
    fn new(font_size: f32, line_height: f32, scaled_descender: f32) -> Self {
        Self {
            x: 0.0,
            baseline_y: -(line_height * 0.5 + font_size * 0.5).round() - scaled_descender,
            line_y: -line_height,
            text_position: TextPosition { line: 0, column: 0 },
            max_computed_line_width: 0.0,
        }
    }

    fn baseline_position(&self) -> Vec2 {
        Vec2::new(self.x, self.baseline_y)
    }

    fn line_position(&self) -> Vec2 {
        Vec2::new(self.x, self.line_y)
    }

    fn advance(&mut self, advance: f32) {
        self.x += advance;
        self.text_position.column += 1;

        self.max_computed_line_width = self.max_computed_line_width.max(self.x);
    }

    fn break_line(&mut self, line_height: f32) {
        self.x = 0.0;
        self.text_position.column = 0;

        self.baseline_y -= line_height;
        self.line_y -= line_height;
        self.text_position.line += 1;
    }
}
