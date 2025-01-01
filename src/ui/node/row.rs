use dyn_partial_eq::DynPartialEq;
use glam::Vec2;

use crate::ui::{processor::UiNodeProcessor, Alignment, Layout, Modifiers};

use super::{UiNode, UiNodeProps};

#[derive(Debug, Clone, PartialEq, DynPartialEq)]
pub struct RowProps;

impl UiNodeProps for RowProps {
    fn measure_fit_content(
        &self,
        _: &Modifiers,
        children: &[UiNode],
        processor: &mut UiNodeProcessor<'_>,
        content_width: Option<f32>,
        content_height: Option<f32>,
    ) -> (Vec2, Vec2) {
        match (content_width, content_height) {
            (Some(content_width), Some(content_height)) => {
                let content_size = Vec2::new(content_width, content_height);
                (content_size, content_size)
            }
            (Some(content_width), None) => {
                let preliminar_children_boundary_size = Vec2::new(content_width, 0.0);

                let min_intrinsic_children_measurements =
                    processor.measure_children(preliminar_children_boundary_size, children);

                let children_measurements =
                    processor.apply_horizontal_weights(content_width, children, min_intrinsic_children_measurements);

                let content_height = children_measurements.max_height();

                let content_size = Vec2::new(content_width, content_height);
                // TODO: should pass preliminar_children_boundry_size ?
                (content_size, content_size)
            }
            (None, Some(content_height)) => {
                let preliminar_children_boundary_size = Vec2::new(0.0, content_height);

                let min_intrinsic_children_measurements =
                    processor.measure_children(preliminar_children_boundary_size, children);

                let final_children_boundary_size = {
                    let min_intrinsic_width = min_intrinsic_children_measurements.width_sum();
                    Vec2::new(min_intrinsic_width, content_height)
                };

                let content_width = {
                    let children_measurements = processor.measure_children(final_children_boundary_size, children);
                    children_measurements.width_sum()
                };

                let content_size = Vec2::new(content_width, content_height);
                (content_size, final_children_boundary_size)
            }
            (None, None) => {
                let preliminar_children_boundary_size = Vec2::new(0.0, 0.0);

                let min_intrinsic_children_measurements =
                    processor.measure_children(preliminar_children_boundary_size, children);

                // We compute height first to use it in final_children_boundary_size.
                let content_height = min_intrinsic_children_measurements.max_height();

                let final_children_boundary_size = {
                    let min_intrinsic_width = min_intrinsic_children_measurements.width_sum();
                    Vec2::new(min_intrinsic_width, content_height)
                };

                let content_width = {
                    let children_measurements = processor.measure_children(final_children_boundary_size, children);
                    children_measurements.width_sum()
                };

                let content_size = Vec2::new(content_width, content_height);
                (content_size, final_children_boundary_size)
            }
        }
    }

    fn compute_children_layouts(
        &self,
        _: &Modifiers,
        children: &[UiNode],
        processor: &mut UiNodeProcessor<'_>,
        layout: &Layout,
    ) -> Vec<Layout> {
        let initial_children_measurements = processor.measure_children(layout.children_boundary_size(), children);
        let children_measurements =
            processor.apply_horizontal_weights(layout.content_size().x, children, initial_children_measurements);

        let mut horizontal_offset = 0.0;

        std::iter::zip(children.iter(), children_measurements.into_iter())
            .map(|(child, child_measurements)| {
                let child_margin_size = child_measurements.margin_size;

                let child_margin_position = match child.modifiers.self_alignment {
                    Alignment::BottomLeft | Alignment::Bottom | Alignment::BottomRight => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        layout.content_position().y,
                    ),
                    Alignment::Left | Alignment::Center | Alignment::Right => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        (layout.content_center().y - 0.5 * child_margin_size.y).round(),
                    ),
                    Alignment::TopLeft | Alignment::Top | Alignment::TopRight => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        layout.content_position().y + layout.content_size().y - child_margin_size.y,
                    ),
                };

                horizontal_offset += child_margin_size.x;

                child_measurements.to_layout(child_margin_position)
            })
            .collect()
    }

    fn emit_draw_data(&self, _: &mut UiNodeProcessor<'_>, _: &Layout) {}
}
