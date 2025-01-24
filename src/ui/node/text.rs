use dyn_partial_eq::DynPartialEq;
use glam::Vec2;

use crate::{
    color::Color,
    font::{
        font_engine::{LaidOutGlyph, TextLayoutOptions},
        font_face::GlyphPixelMode,
    },
    mesh::mesh::Mesh,
    rectangle::Rectangle,
    text::text_position::TextPosition,
    ui::{
        color_mesh_builder::ColorMeshBuilder,
        draw_command::Shader,
        processor::{MeshWithShader, UiNodeProcessor},
        texture_mesh_builder::TextureMeshBuilder,
        Layout, Modifiers,
    },
};

use super::{UiNode, UiNodeProps};

#[derive(Debug, Clone, PartialEq, DynPartialEq)]
pub struct TextProps {
    pub text: String,
    pub text_color: Color,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub cursor_position: Option<TextPosition>,
}

impl UiNodeProps for TextProps {
    fn measure_fit_content(
        &self,
        _: &Modifiers,
        _: &[UiNode],
        processor: &mut UiNodeProcessor<'_>,
        content_width: Option<f32>,
        content_height: Option<f32>,
    ) -> (Vec2, Vec2) {
        let TextProps {
            text,
            font_family,
            font_size,
            line_height,
            ..
        } = self;

        let max_line_width = content_width.unwrap_or(f32::INFINITY);

        let mut text_options = TextLayoutOptions {
            font_family,
            font_size: *font_size,
            line_height: *line_height,
            max_line_width,
        };

        let mut text_layout = processor.font_engine.lay_out_text(text, &text_options);

        // If width is FitContent and max_line_width is not enough for some characters,
        // then take advantage of the extra line length for all lines.
        if text_layout.size.x > max_line_width && content_width.is_none() {
            text_options.max_line_width = text_layout.size.x;
            text_layout = processor.font_engine.lay_out_text(text, &text_options);
        }

        let content_width = content_width.unwrap_or_else(|| text_layout.size.x);
        let content_height = content_height.unwrap_or_else(|| text_layout.size.y);
        let content_size = Vec2::new(content_width, content_height);

        (content_size, Vec2::ZERO)
    }

    fn compute_children_layouts(
        &self,
        _: &Modifiers,
        children: &[UiNode],
        _: &mut UiNodeProcessor<'_>,
        _: &Layout,
    ) -> Vec<Layout> {
        assert!(children.is_empty(), "Text can't contain children.");
        vec![]
    }

    fn emit_draw_data(&self, processor: &mut UiNodeProcessor<'_>, layout: &Layout) {
        let TextProps {
            text,
            text_color,
            font_family,
            font_size,
            line_height,
            cursor_position,
            ..
        } = self;

        let origin = layout.content_position() + Vec2::new(0.0, layout.content_size().y);
        let options = TextLayoutOptions {
            font_family,
            font_size: *font_size,
            line_height: *line_height,
            max_line_width: layout.content_size().x,
        };

        let text_layout = processor.font_engine.lay_out_text(text, &options);
        let mut text_mesh_builder = TextMeshBuilder::new();

        for glyph in &text_layout.glyphs {
            text_mesh_builder.add_glyph(origin, *text_color, glyph);
        }

        let meshes = text_mesh_builder.build();

        for (pixel_mode, mesh) in meshes {
            let shader = match pixel_mode {
                GlyphPixelMode::Grayscale => Shader::TextGrayscale,
                GlyphPixelMode::Subpixel => Shader::TextSubpixel,
                GlyphPixelMode::Color => Shader::Texture,
            };

            processor.draw_elements.push(MeshWithShader(mesh, shader));
        }

        if let Some(cursor_position) = cursor_position {
            const CURSOR_WIDTH: f32 = 2.0;

            let mut position = origin + text_layout.text_map.get_clamped(*cursor_position);
            position.x -= (0.5 * CURSOR_WIDTH).round();

            let cursor_rectangle =
                Rectangle::from_position_size(position, Vec2::new(CURSOR_WIDTH, options.line_height));
            let cursor_color = Color::rgba(1.0, 1.0, 1.0, 1.0);

            let mut mesh_builder = ColorMeshBuilder::new();
            mesh_builder.add_vertex(cursor_rectangle.bottom_left(), cursor_color);
            mesh_builder.add_vertex(cursor_rectangle.bottom_right(), cursor_color);
            mesh_builder.add_vertex(cursor_rectangle.top_right(), cursor_color);
            mesh_builder.add_vertex(cursor_rectangle.top_left(), cursor_color);
            mesh_builder.add_triangle(0, 1, 2);
            mesh_builder.add_triangle(0, 2, 3);

            processor
                .draw_elements
                .push(MeshWithShader(mesh_builder.build(), Shader::Shape));
        }
    }
}

struct TextMeshBuilder {
    grayscale_builder: TextureMeshBuilder,
    subpixel_builder: TextureMeshBuilder,
    color_builder: TextureMeshBuilder,
}

impl TextMeshBuilder {
    fn new() -> Self {
        Self {
            grayscale_builder: TextureMeshBuilder::new(),
            subpixel_builder: TextureMeshBuilder::new(),
            color_builder: TextureMeshBuilder::new(),
        }
    }

    fn add_glyph(&mut self, position: Vec2, text_color: Color, glyph: &LaidOutGlyph) {
        let bounds = Rectangle::from_position_size(position + glyph.position, glyph.size);
        let uv_rectangle = glyph.atlas_uv_rectangle;

        let builder = match glyph.pixel_mode {
            GlyphPixelMode::Grayscale => &mut self.grayscale_builder,
            GlyphPixelMode::Subpixel => &mut self.subpixel_builder,
            GlyphPixelMode::Color => &mut self.color_builder,
        };

        let bl = builder.add_vertex(bounds.bottom_left(), uv_rectangle.bottom_left(), text_color);
        let br = builder.add_vertex(bounds.bottom_right(), uv_rectangle.bottom_right(), text_color);
        let tr = builder.add_vertex(bounds.top_right(), uv_rectangle.top_right(), text_color);
        let tl = builder.add_vertex(bounds.top_left(), uv_rectangle.top_left(), text_color);

        builder.add_quad(bl, br, tr, tl);

        if let Some(image_id) = builder.image_id {
            assert_eq!(image_id, glyph.image_id);
        } else {
            builder.image_id = Some(glyph.image_id);
        }
    }

    fn build(self) -> Vec<(GlyphPixelMode, Mesh)> {
        let mut meshes = vec![];

        let grayscale_mesh = self.grayscale_builder.build();
        let subpixel_mesh = self.subpixel_builder.build();
        let color_mesh = self.color_builder.build();

        if grayscale_mesh.vertex_count > 0 {
            meshes.push((GlyphPixelMode::Grayscale, grayscale_mesh));
        }

        if subpixel_mesh.vertex_count > 0 {
            meshes.push((GlyphPixelMode::Subpixel, subpixel_mesh));
        }

        if color_mesh.vertex_count > 0 {
            meshes.push((GlyphPixelMode::Color, color_mesh));
        }

        meshes
    }
}
