use dyn_partial_eq::DynPartialEq;
use glam::Vec2;

use crate::ui::{processor::UiNodeProcessor, Alignment, Layout, Modifiers};

use super::{UiNode, UiNodeProps};

#[derive(Debug, Clone, PartialEq, DynPartialEq)]
pub struct BlockProps;

impl UiNodeProps for BlockProps {
    fn measure_fit_content(
        &self,
        _: &Modifiers,
        children: &[UiNode],
        processor: &mut UiNodeProcessor<'_>,
        content_width: Option<f32>,
        content_height: Option<f32>,
    ) -> (Vec2, Vec2) {
        let preliminar_children_boundary_size = Vec2::new(content_width.unwrap_or(0.0), content_height.unwrap_or(0.0));

        let min_intrinsic_children_measurements =
            processor.measure_children(preliminar_children_boundary_size, &children);

        let content_width = content_width.unwrap_or_else(|| min_intrinsic_children_measurements.max_width());
        let content_height = content_height.unwrap_or_else(|| min_intrinsic_children_measurements.max_height());

        let content_size = Vec2::new(content_width, content_height);
        (content_size, content_size)
    }

    fn compute_children_layouts(
        &self,
        _: &Modifiers,
        children: &[UiNode],
        processor: &mut UiNodeProcessor<'_>,
        layout: &Layout,
    ) -> Vec<Layout> {
        children
            .iter()
            .map(|child| {
                let child_measurements = processor.measure(child, layout.children_boundary_size());

                let child_margin_size = child_measurements.margin_size;
                let child_margin_position = match child.modifiers.self_alignment {
                    Alignment::Center => layout.content_center() - 0.5 * child_margin_size,
                    Alignment::Right => Vec2::new(
                        layout.content_position().x + layout.content_size().x - child_margin_size.x,
                        layout.content_center().y - 0.5 * child_margin_size.y,
                    ),
                    Alignment::TopRight => layout.content_position() + layout.content_size() - child_margin_size,
                    Alignment::Top => Vec2::new(
                        layout.content_center().x - 0.5 * child_margin_size.x,
                        layout.content_position().y + layout.content_size().y - child_margin_size.y,
                    ),
                    Alignment::TopLeft => Vec2::new(
                        layout.content_position().x,
                        layout.content_position().y + layout.content_size().y - child_margin_size.y,
                    ),
                    Alignment::Left => Vec2::new(
                        layout.content_position().x,
                        layout.content_center().y - 0.5 * child_margin_size.y,
                    ),
                    Alignment::BottomLeft => layout.content_position(),
                    Alignment::Bottom => Vec2::new(
                        layout.content_center().x - 0.5 * child_margin_size.x,
                        layout.content_position().y,
                    ),
                    Alignment::BottomRight => Vec2::new(
                        layout.content_position().x + layout.content_size().x - child_margin_size.x,
                        layout.content_position().y,
                    ),
                }
                .round();

                child_measurements.to_layout(child_margin_position)
            })
            .collect()
    }

    fn emit_draw_data(&self, _: &mut UiNodeProcessor<'_>, _: &Layout) {}
}
