use std::path::Path;

use image::ImageReader;

use super::image_format::ImageFormat;

#[derive(Debug, Clone)]
pub struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
    pitch: u32, // Bytes per row.
    format: ImageFormat,
}

impl Image {
    pub fn new_empty(width: u32, height: u32, format: ImageFormat) -> Self {
        Self {
            data: vec![0u8; (width * height * format.bytes_per_pixel()) as usize],
            width,
            height,
            pitch: width * format.bytes_per_pixel(),
            format,
        }
    }

    pub fn from_data(data: Vec<u8>, width: u32, height: u32, pitch: u32, format: ImageFormat) -> Self {
        assert_eq!(data.len() as u32, height * pitch);
        assert!(width * format.bytes_per_pixel() <= pitch);

        Self {
            data,
            width,
            height,
            pitch,
            format,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        unsafe {
            stb_image::stb_image::stbi_set_flip_vertically_on_load(1);
        }
        let image = stb_image::image::load_with_depth(path, 4, true);

        match image {
            stb_image::image::LoadResult::Error(_) => todo!(),
            stb_image::image::LoadResult::ImageF32(_) => todo!(),
            stb_image::image::LoadResult::ImageU8(image) => {
                assert_eq!(image.depth, 4);

                Image::from_data(
                    image.data,
                    image.width as u32,
                    image.height as u32,
                    (image.width * 4) as u32,
                    ImageFormat::Rgba8Srgb,
                )
            }
        }
    }

    #[allow(unused)]
    fn from_path_image_rs(path: &Path) -> Self {
        let image = ImageReader::open(path).expect("TODO").decode().expect("TODO");

        Image::from_dynamic_image(image)
    }

    fn from_dynamic_image(image: image::DynamicImage) -> Self {
        let format = match &image {
            image::DynamicImage::ImageLuma8(_) => ImageFormat::R8Unorm,
            image::DynamicImage::ImageLumaA8(_) => todo!(),
            image::DynamicImage::ImageLuma16(_) => todo!(),
            image::DynamicImage::ImageLumaA16(_) => todo!(),
            image::DynamicImage::ImageRgb8(_) => ImageFormat::Rgb8Unorm,
            image::DynamicImage::ImageRgba8(_) => ImageFormat::Rgba8UnormPre,
            image::DynamicImage::ImageRgb16(_) => todo!(),
            image::DynamicImage::ImageRgba16(_) => todo!(),
            image::DynamicImage::ImageRgb32F(_) => todo!(),
            image::DynamicImage::ImageRgba32F(_) => todo!(),
            _ => todo!(),
        };

        let width = image.width();
        let height = image.height();
        let bytes = image.into_bytes();

        let image = Image::from_data(bytes, width, height, width * format.bytes_per_pixel(), format);

        image
    }

    fn coords_to_index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);

        (y * self.pitch + x * self.format.bytes_per_pixel()) as usize
    }

    #[allow(unused)]
    pub fn get_r(&self, x: u32, y: u32) -> u8 {
        assert_eq!(self.format, ImageFormat::R8Unorm);

        self.data[self.coords_to_index(x, y)]
    }

    #[allow(unused)]
    pub fn set_r(&mut self, x: u32, y: u32, value: u8) {
        assert_eq!(self.format, ImageFormat::R8Unorm);

        let index = self.coords_to_index(x, y);
        self.data[index] = value;
    }

    #[allow(unused)]
    pub fn get_rgb(&self, x: u32, y: u32) -> [u8; 3] {
        assert_eq!(self.format, ImageFormat::Rgb8Unorm);

        let r_index = self.coords_to_index(x, y);
        [self.data[r_index], self.data[r_index + 1], self.data[r_index + 2]]
    }

    #[allow(unused)]
    pub fn set_rgb(&mut self, x: u32, y: u32, value: [u8; 3]) {
        assert_eq!(self.format, ImageFormat::Rgb8Unorm);

        let r_index = self.coords_to_index(x, y);
        self.data[r_index] = value[0];
        self.data[r_index + 1] = value[1];
        self.data[r_index + 2] = value[2];
    }

    #[allow(unused)]
    pub fn get_rgba(&self, x: u32, y: u32) -> [u8; 4] {
        assert_eq!(self.format, ImageFormat::Rgba8UnormPre);

        let r_index = self.coords_to_index(x, y);
        [
            self.data[r_index],
            self.data[r_index + 1],
            self.data[r_index + 2],
            self.data[r_index + 3],
        ]
    }

    #[allow(unused)]
    pub fn set_rgba(&mut self, x: u32, y: u32, value: [u8; 4]) {
        assert_eq!(self.format, ImageFormat::Rgba8UnormPre);

        let r_index = self.coords_to_index(x, y);
        self.data[r_index] = value[0];
        self.data[r_index + 1] = value[1];
        self.data[r_index + 2] = value[2];
        self.data[r_index + 3] = value[3];
    }

    #[allow(unused)]
    pub fn get_bgra(&self, x: u32, y: u32) -> [u8; 4] {
        assert_eq!(self.format, ImageFormat::Bgra8Unorm);

        let r_index = self.coords_to_index(x, y);
        [
            self.data[r_index],
            self.data[r_index + 1],
            self.data[r_index + 2],
            self.data[r_index + 3],
        ]
    }

    #[allow(unused)]
    pub fn set_bgra(&mut self, x: u32, y: u32, value: [u8; 4]) {
        assert_eq!(self.format, ImageFormat::Bgra8Unorm);

        let r_index = self.coords_to_index(x, y);
        self.data[r_index] = value[0];
        self.data[r_index + 1] = value[1];
        self.data[r_index + 2] = value[2];
        self.data[r_index + 3] = value[3];
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

    pub fn format(&self) -> ImageFormat {
        self.format
    }
}

impl ToString for Image {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for y in 0..self.height {
            let y = self.height - y - 1;
            for x in 0..self.width {
                let c = match self.get_r(x, y) {
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
