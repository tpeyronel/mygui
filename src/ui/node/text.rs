use dyn_partial_eq::DynPartialEq;
use glam::Vec2;

use crate::{
    font::font_engine::TextLayoutOptions,
    rectangle::Rectangle,
    ui::{draw_element::DrawElement, processor::UiNodeProcessor, Layout, Modifiers},
    vertex::Color,
};

use super::{UiNode, UiNodeProps};

#[derive(Debug, Clone, PartialEq, DynPartialEq)]
pub struct TextProps {
    pub text: String,
    pub text_color: Color,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
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

        let (mut dimensions, _) = processor.font_engine.lay_out_text(text, &text_options);

        // If width is FitContent and max_line_width is not enough for some characters,
        // then take advantage of the extra line length for all lines.
        if dimensions.x > max_line_width && content_width.is_none() {
            text_options.max_line_width = dimensions.x;
            (dimensions, _) = processor.font_engine.lay_out_text(text, &text_options);
        }

        let content_width = content_width.unwrap_or_else(|| dimensions.x);
        let content_height = content_height.unwrap_or_else(|| dimensions.y);
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
            ..
        } = self;

        let origin = layout.content_position() + Vec2::new(0.0, layout.content_size().y);
        let options = TextLayoutOptions {
            font_family,
            font_size: *font_size,
            line_height: *line_height,
            max_line_width: layout.content_size().x,
        };

        let (_, glyphs) = processor.font_engine.lay_out_text(text, &options);

        for glyph in glyphs {
            let texture = DrawElement::TextGlyph {
                bounds: Rectangle::from_position_size(origin + glyph.position, glyph.size),
                uv_rectangle: glyph.atlas_uv_rectangle,
                text_color: *text_color,
                image_id: glyph.image_id,
                pixel_mode: glyph.pixel_mode,
            };

            processor.draw_data.push(texture);
        }
    }
}
