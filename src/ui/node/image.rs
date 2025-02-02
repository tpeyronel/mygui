use dyn_partial_eq::DynPartialEq;
use glam::Vec2;

use crate::{
    color::Color,
    image::{image_manager::ImageId, AddressMode, FilterMode},
    ui::{
        draw_command::Shader, processor::UiNodeProcessor, texture_mesh_builder::TextureMeshBuilder, Layout, Modifiers,
    },
};

use super::{UiNode, UiNodeProps};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageMetadata {
    pub image_id: ImageId,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SamplerDescriptor {
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
}

#[derive(Debug, Clone, PartialEq, DynPartialEq)]
pub struct ImageProps {
    metadata: ImageMetadata,
    sampler: SamplerDescriptor,
}

impl ImageProps {
    pub fn new(metadata: ImageMetadata, sampler: SamplerDescriptor) -> Self {
        Self { metadata, sampler }
    }
}

impl UiNodeProps for ImageProps {
    fn measure_fit_content(
        &self,
        _: &Modifiers,
        _: &[UiNode],
        _: &mut UiNodeProcessor<'_>,
        content_width: Option<f32>,
        content_height: Option<f32>,
    ) -> (Vec2, Vec2) {
        (
            Vec2::new(
                content_width.unwrap_or_else(|| self.metadata.width as f32),
                content_height.unwrap_or_else(|| self.metadata.height as f32),
            ),
            Vec2::ZERO,
        )
    }

    fn compute_children_layouts(
        &self,
        _: &Modifiers,
        children: &[UiNode],
        _: &mut UiNodeProcessor<'_>,
        _: &Layout,
    ) -> Vec<Layout> {
        assert!(children.is_empty(), "Image can't contain children.");
        vec![]
    }

    fn emit_draw_data(&self, processor: &mut UiNodeProcessor<'_>, layout: &Layout) {
        let mut mesh_builder = TextureMeshBuilder::new();
        mesh_builder.image_id = Some(self.metadata.image_id);

        let content_position = layout.content_position();
        let content_size = layout.content_size();

        let bl = mesh_builder.add_vertex(content_position, Vec2::new(0.0, 0.0), Color::TRANSPARENT);
        let br = mesh_builder.add_vertex(
            content_position + content_size.with_y(0.0),
            Vec2::new(1.0, 0.0),
            Color::TRANSPARENT,
        );
        let tr = mesh_builder.add_vertex(content_position + content_size, Vec2::new(1.0, 1.0), Color::TRANSPARENT);
        let tl = mesh_builder.add_vertex(
            content_position + content_size.with_x(0.0),
            Vec2::new(0.0, 1.0),
            Color::TRANSPARENT,
        );

        mesh_builder.add_quad(bl, br, tr, tl);

        processor
            .command_list_builder
            .draw_mesh(mesh_builder.build(), Shader::Texture);
    }
}
