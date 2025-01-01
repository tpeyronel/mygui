use dyn_partial_eq::DynPartialEq;
use glam::Vec2;

use crate::ui::{processor::UiNodeProcessor, Alignment, Layout, Modifiers};

use super::{UiNode, UiNodeProps};

#[derive(Debug, Clone, PartialEq, DynPartialEq)]
pub struct ColumnProps;

impl UiNodeProps for ColumnProps {
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

                let final_children_boundary_size = {
                    let min_intrinsic_height = min_intrinsic_children_measurements.height_sum();
                    Vec2::new(content_width, min_intrinsic_height)
                };

                let content_height = {
                    let children_measurements = processor.measure_children(final_children_boundary_size, children);
                    children_measurements.height_sum()
                };

                let content_size = Vec2::new(content_width, content_height);
                (content_size, final_children_boundary_size)
            }
            (None, Some(content_height)) => {
                let preliminar_children_boundary_size = Vec2::new(0.0, content_height);

                let min_intrinsic_children_measurements =
                    processor.measure_children(preliminar_children_boundary_size, children);

                let children_measurements =
                    processor.apply_vertical_weights(content_height, children, min_intrinsic_children_measurements);

                let content_width = children_measurements.max_width();

                let content_size = Vec2::new(content_width, content_height);
                // TODO: should pass preliminar_children_boundry_size ?
                (content_size, content_size)
            }
            (None, None) => {
                let preliminar_children_boundary_size = Vec2::new(0.0, 0.0);

                let min_intrinsic_children_measurements =
                    processor.measure_children(preliminar_children_boundary_size, children);

                let content_width = min_intrinsic_children_measurements.max_width();

                let final_children_boundary_size = {
                    let min_intrinsic_height = min_intrinsic_children_measurements.height_sum();
                    Vec2::new(content_width, min_intrinsic_height)
                };

                let content_height = {
                    let children_measurements = processor.measure_children(final_children_boundary_size, children);
                    children_measurements.height_sum()
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
            processor.apply_vertical_weights(layout.content_size().y, children, initial_children_measurements);

        let mut vertical_offset = 0.0;
        let column_top = layout.content_position().y + layout.content_size().y;
        std::iter::zip(children.iter(), children_measurements.into_iter())
            .map(|(child, child_measurements)| {
                let child_margin_size = child_measurements.margin_size;
                vertical_offset += child_margin_size.y;

                let child_margin_position = match child.modifiers.self_alignment {
                    Alignment::TopLeft | Alignment::Left | Alignment::BottomLeft => {
                        Vec2::new(layout.content_position().x, column_top - vertical_offset)
                    }
                    Alignment::Top | Alignment::Center | Alignment::Bottom => Vec2::new(
                        (layout.content_center().x - 0.5 * child_margin_size.x).round(),
                        column_top - vertical_offset,
                    ),
                    Alignment::TopRight | Alignment::Right | Alignment::BottomRight => Vec2::new(
                        layout.content_position().x + layout.content_size().x - child_margin_size.x,
                        column_top - vertical_offset,
                    ),
                };

                child_measurements.to_layout(child_margin_position)
            })
            .collect()
    }

    fn emit_draw_data(&self, _: &mut UiNodeProcessor<'_>, _: &Layout) {}
}
