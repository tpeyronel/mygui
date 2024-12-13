use crate::image::image_format::ImageFormat;

impl ImageFormat {
    pub fn to_wgpu_texture_format(&self) -> Option<wgpu::TextureFormat> {
        Some(match self {
            ImageFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
            ImageFormat::Rgb8Unorm => return None,
            ImageFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        })
    }
}
