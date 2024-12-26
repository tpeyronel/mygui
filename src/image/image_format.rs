use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageFormat {
    R8Unorm,
    Rgb8Unorm,
    Rgba8Unorm,
    Bgra8Unorm,
}

impl ImageFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            ImageFormat::R8Unorm => 1,
            ImageFormat::Rgb8Unorm => 3,
            ImageFormat::Rgba8Unorm | ImageFormat::Bgra8Unorm => 4,
        }
    }
}
