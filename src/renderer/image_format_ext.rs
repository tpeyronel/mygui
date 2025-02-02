use crate::image::image_format::ImageFormat;

impl ImageFormat {
    pub fn to_wgpu_texture_format(&self) -> Option<wgpu::TextureFormat> {
        Some(match self {
            ImageFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
            ImageFormat::Rgb8Unorm => return None,
            ImageFormat::Rgba8UnormPost | ImageFormat::Rgba8UnormPre => wgpu::TextureFormat::Rgba8Unorm,
            ImageFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            ImageFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        })
    }
}
