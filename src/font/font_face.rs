use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    ops::Add,
    path::{Path, PathBuf},
};

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::{
    image::{
        image::Image,
        image_format::ImageFormat,
        image_manager::{ImageId, ImageManager},
    },
    rectangle::Rectangle,
};

pub struct FontFace {
    file_path: PathBuf,
    ft_face: freetype::Face,
    charmap: HashMap<char, u32>, // Maps chars to glyph indices.
    per_size_data: HashMap<u32, FontPerSizeData>,
}

impl FontFace {
    pub fn from_path(path: &Path, ft_lib: &freetype::Library) -> Self {
        let file_path = path.to_path_buf();
        let ft_face = ft_lib.new_face(&file_path, 0).expect("TODO");
        let charmap = Self::load_charmap(&ft_face);

        Self {
            file_path,
            ft_face,
            charmap,
            per_size_data: HashMap::new(),
        }
    }

    pub fn try_from_cache(
        path: &Path,
        cache_dir_path: &Path, // The path of the cache for this *specific* font file.
        ft_lib: &freetype::Library,
        image_manager: &mut ImageManager,
    ) -> io::Result<Self> {
        let file_path = path.to_path_buf();
        let ft_face = ft_lib.new_face(&file_path, 0).expect("TODO");

        let mut metadata_path = cache_dir_path.to_path_buf();
        metadata_path.push("metadata");
        let metadata_file = File::open(metadata_path)?;
        log::info!("loading {:?} from cache", path.file_name().unwrap());

        let reader = BufReader::new(metadata_file);

        let FontFaceMetadata {
            glyph_count,
            charmap,
            instances,
        } = bincode::deserialize_from(reader).expect("TODO");

        let per_size_data: HashMap<u32, FontPerSizeData> = instances
            .into_iter()
            .map(|(font_size, instance_metadata)| {
                let size_data = FontPerSizeData {
                    advances: instance_metadata.advances,
                    grayscale_atlas_metadata: instance_metadata.grayscale_atlas_metadata.map(|m| {
                        Self::load_atlas_metadata_from_cache(
                            m,
                            cache_dir_path,
                            font_size,
                            GlyphPixelMode::Grayscale,
                            glyph_count,
                            image_manager,
                        )
                        .expect("TODO")
                    }),
                    subpixel_atlas_metadata: instance_metadata.subpixel_atlas_metadata.map(|m| {
                        Self::load_atlas_metadata_from_cache(
                            m,
                            cache_dir_path,
                            font_size,
                            GlyphPixelMode::Subpixel,
                            glyph_count,
                            image_manager,
                        )
                        .expect("TODO")
                    }),
                    color_atlas_metadata: instance_metadata.color_atlas_metadata.map(|m| {
                        Self::load_atlas_metadata_from_cache(
                            m,
                            cache_dir_path,
                            font_size,
                            GlyphPixelMode::Color,
                            glyph_count,
                            image_manager,
                        )
                        .expect("TODO")
                    }),
                };

                (font_size, size_data)
            })
            .collect();

        Ok(Self {
            file_path,
            ft_face,
            charmap,
            per_size_data,
        })
    }

    pub fn get_glyph_index(&self, c: char) -> Option<u32> {
        self.charmap.get(&c).map(|&g| g)
    }

    pub fn ensure_size_data(&mut self, font_size: u32) {
        self.per_size_data.entry(font_size).or_insert_with(|| FontPerSizeData {
            advances: Self::load_advances(&self.ft_face, font_size),
            grayscale_atlas_metadata: None,
            subpixel_atlas_metadata: None,
            color_atlas_metadata: None,
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

    pub fn save_to_disk(&self, cache_dir_path: &Path, image_manager: &ImageManager) -> io::Result<()> {
        let mut dir_path = cache_dir_path.to_path_buf();
        dir_path.push(self.file_path.file_name().unwrap());
        std::fs::create_dir_all(&dir_path)?;

        let main_file_contents = FontFaceMetadata {
            glyph_count: self.ft_face.num_glyphs() as u32,
            charmap: self.charmap.clone(),
            instances: self
                .per_size_data
                .iter()
                .map(|(&font_size, size_data)| {
                    let instance_metadata = FontFaceInstanceMetadata {
                        advances: size_data.advances.clone(),
                        color_atlas_metadata: size_data
                            .color_atlas_metadata
                            .as_ref()
                            .map(|m| Self::atlas_metadata_from_atlas(m, image_manager)),
                        subpixel_atlas_metadata: size_data
                            .subpixel_atlas_metadata
                            .as_ref()
                            .map(|m| Self::atlas_metadata_from_atlas(m, image_manager)),
                        grayscale_atlas_metadata: size_data
                            .grayscale_atlas_metadata
                            .as_ref()
                            .map(|m| Self::atlas_metadata_from_atlas(m, image_manager)),
                    };

                    (font_size, instance_metadata)
                })
                .collect::<HashMap<u32, FontFaceInstanceMetadata>>(),
        };

        {
            let mut metadata_path = dir_path.clone();
            metadata_path.push("metadata");
            let main_file = File::create(metadata_path)?;
            let mut writer = BufWriter::new(main_file);
            writer.write(&bincode::serialize(&main_file_contents).unwrap())?;
        }

        for (&font_size, size_data) in &self.per_size_data {
            if let Some(color_atlas_metadata) = &size_data.color_atlas_metadata {
                Self::save_atlas_to_disk(
                    &dir_path,
                    font_size,
                    GlyphPixelMode::Color,
                    color_atlas_metadata,
                    image_manager,
                )?;
            }
            if let Some(subpixel_atlas_metadata) = &size_data.subpixel_atlas_metadata {
                Self::save_atlas_to_disk(
                    &dir_path,
                    font_size,
                    GlyphPixelMode::Subpixel,
                    subpixel_atlas_metadata,
                    image_manager,
                )?;
            }
            if let Some(grayscale_atlas_metadata) = &size_data.grayscale_atlas_metadata {
                Self::save_atlas_to_disk(
                    &dir_path,
                    font_size,
                    GlyphPixelMode::Grayscale,
                    grayscale_atlas_metadata,
                    image_manager,
                )?;
            }
        }

        Ok(())
    }

    fn save_atlas_to_disk(
        dir_path: &Path,
        font_size: u32,
        pixel_mode: GlyphPixelMode,
        metadata: &GlyphAtlasMetadata,
        image_manager: &ImageManager,
    ) -> io::Result<()> {
        let mut color_atlas_data_path = dir_path.to_path_buf();
        color_atlas_data_path.push(Self::atlas_file_name(font_size, pixel_mode));
        let color_atlas_file = File::create(color_atlas_data_path)?;
        let writer = BufWriter::new(color_atlas_file);
        let atlas_image = image_manager.get_image(metadata.image_id);
        let atlas_data = atlas_image.data();
        zstd::stream::copy_encode(atlas_data, writer, 5)?;

        Ok(())
    }

    fn atlas_file_name(font_size: u32, pixel_mode: GlyphPixelMode) -> String {
        match pixel_mode {
            GlyphPixelMode::Grayscale => format!("{}-{}", font_size, "grayscale"),
            GlyphPixelMode::Subpixel => format!("{}-{}", font_size, "subpixel"),
            GlyphPixelMode::Color => format!("{}-{}", font_size, "color"),
        }
    }

    fn atlas_metadata_from_atlas(atlas: &GlyphAtlasMetadata, image_manager: &ImageManager) -> FontFaceAtlasMetadata {
        let image = image_manager.get_image(atlas.image_id);
        let width = image.width();
        let height = image.height();
        let format = image.format();

        let glyph_metadata: Vec<_> = atlas
            .glyph_indices_table
            .iter()
            .enumerate()
            .filter_map(|(g, &i)| {
                if i == u32::MAX {
                    return None;
                }

                let glyph_metadata = atlas.glyph_metadata[i as usize].clone();

                Some((g as u32, glyph_metadata))
            })
            .collect();

        FontFaceAtlasMetadata {
            width,
            height,
            format,
            glyph_metadata,
        }
    }

    fn load_atlas_metadata_from_cache(
        atlas_metadata: FontFaceAtlasMetadata,
        font_cache_path: &Path,
        font_size: u32,
        pixel_mode: GlyphPixelMode,
        glyph_count: u32,
        image_manager: &mut ImageManager,
    ) -> io::Result<GlyphAtlasMetadata> {
        let width = atlas_metadata.width;
        let height = atlas_metadata.height;
        let format = atlas_metadata.format;
        let data = {
            let mut atlas_path = font_cache_path.to_path_buf();
            atlas_path.push(Self::atlas_file_name(font_size, pixel_mode));
            let atlas_file = File::open(atlas_path)?;
            let reader = BufReader::new(atlas_file);
            zstd::stream::decode_all(reader)?
        };

        let atlas_image = Image::from_data(data, width, height, width * format.bytes_per_pixel(), format);
        let image_id = image_manager.add_image(atlas_image);

        let mut glyph_metadata = vec![];
        let mut glyph_indices_table = vec![u32::MAX; glyph_count as usize];

        for (g, metadata) in atlas_metadata.glyph_metadata {
            let i = glyph_metadata.len();
            glyph_metadata.push(metadata);
            assert_eq!(glyph_indices_table[g as usize], u32::MAX);
            glyph_indices_table[g as usize] = i as u32;
        }

        Ok(GlyphAtlasMetadata {
            image_id,
            glyph_metadata,
            glyph_indices_table,
        })
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
struct FontFaceMetadata {
    glyph_count: u32,
    charmap: HashMap<char, u32>, // Maps chars to glyph indices.
    instances: HashMap<u32, FontFaceInstanceMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FontFaceInstanceMetadata {
    advances: Vec<i32>,
    color_atlas_metadata: Option<FontFaceAtlasMetadata>,
    subpixel_atlas_metadata: Option<FontFaceAtlasMetadata>,
    grayscale_atlas_metadata: Option<FontFaceAtlasMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FontFaceAtlasMetadata {
    width: u32,
    height: u32,
    format: ImageFormat,
    glyph_metadata: Vec<(u32, GlyphMetadata)>, // First element is glyph index.
}
