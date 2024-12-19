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
        assert_eq!(self.format, ImageFormat::Rgba8Unorm);

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
        assert_eq!(self.format, ImageFormat::Rgba8Unorm);

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
