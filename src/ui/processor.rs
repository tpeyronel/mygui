use core::f32;
use std::u64;

use super::border_thickness::BorderThickness;
use super::draw_element::DrawElement;
use super::margin::Margin;
use super::measurements_cache::MeasurementsCache;
use super::padding::Padding;
use super::{border_radius::BorderRadius, UiNode};
use super::{Axis, HashNode, Measurements, UiNodeLayout};
use glam::Vec2;

use crate::is_integer::IsInteger;
use crate::ui::node::block::BlockProps;
use crate::ui::{Extent, Layout, Modifiers};
use crate::{font::font_engine::FontEngine, rectangle::Rectangle, vertex::Color};

pub struct UiNodeProcessor<'a> {
    measurements_cache: MeasurementsCache,
    pub font_engine: &'a mut Box<dyn FontEngine>,
    pub draw_data: &'a mut Vec<DrawElement>,
    bounding_boxes: &'a mut Vec<(u64, Rectangle)>,
}

impl<'a> UiNodeProcessor<'a> {
    pub fn new(
        font_engine: &'a mut Box<dyn FontEngine>,
        draw_data: &'a mut Vec<DrawElement>,
        bounding_boxes: &'a mut Vec<(u64, Rectangle)>,
    ) -> Self {
        Self {
            measurements_cache: MeasurementsCache::new(),
            font_engine,
            draw_data,
            bounding_boxes,
        }
    }

    pub fn to_draw_data(
        &mut self,
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
    ) {
        assert_eq!(self.draw_data.len(), 0);
        assert_eq!(self.bounding_boxes.len(), 0);

        let boundary_pos = boundary_pos.round();
        let boundary_size = boundary_size.round();

        let root_node = UiNode {
            props: Box::new(BlockProps),
            modifiers: Modifiers::new()
                .width(Extent::Px(boundary_size.x))
                .height(Extent::Px(boundary_size.y))
                .clone(),
            children: ui_nodes,
        };

        let root_hash_node = HashNode {
            hash: u64::MAX,
            children: hash_nodes,
        };

        let root_layout = Layout {
            margin_position: boundary_pos,
            margin_size: boundary_size,
            children_boundary_size: boundary_size,
            margin: Margin::all(0.0),
            border_thickness: BorderThickness::all(0.0),
            padding: Padding::all(0.0),
        };

        let root_layout_node = self.compute_layout_rec(&root_node, root_layout);
        self.to_draw_data_rec(&root_node, &root_hash_node, &root_layout_node);
        self.draw_data.remove(0); // TODO: remove. This is mainly done to simplify testing (the first rectangle is completely transparent).
    }

    fn to_draw_data_rec(&mut self, ui_node: &UiNode, hash_node: &HashNode, layout_node: &UiNodeLayout) {
        let modifiers = &ui_node.modifiers;
        let children = &ui_node.children;

        let layout = &layout_node.layout;

        self.emit_rectangle(
            layout.border_position(),
            layout.border_size(),
            modifiers.fill_color,
            modifiers.border_color,
            modifiers.border_thickness,
            modifiers.border_radius,
        );

        self.bounding_boxes.push((
            hash_node.hash,
            Rectangle::from_position_size(layout.border_position(), layout.border_size()),
        ));

        ui_node.props.emit_draw_data(self, layout);

        debug_assert_eq!(children.len(), hash_node.children.len());
        debug_assert_eq!(children.len(), layout_node.children.len());
        for (i, c) in children.iter().enumerate() {
            self.to_draw_data_rec(c, &hash_node.children[i], &layout_node.children[i]);
        }
    }

    fn compute_layout_rec(&mut self, ui_node: &UiNode, layout: Layout) -> UiNodeLayout {
        assert!(layout.margin_position.x.is_integer());
        assert!(layout.margin_position.y.is_integer());
        assert!(layout.margin_size.x.is_integer());
        assert!(layout.margin_size.y.is_integer());

        let children_layouts = self.compute_children_layouts(ui_node, &layout);

        UiNodeLayout {
            layout,
            children: children_layouts,
        }
    }

    fn compute_children_layouts(&mut self, ui_node: &UiNode, layout: &Layout) -> Vec<UiNodeLayout> {
        ui_node
            .props
            .compute_children_layouts(&ui_node.modifiers, &ui_node.children, self, layout)
            .into_iter()
            .zip(ui_node.children.iter())
            .map(|(child_layout, child)| self.compute_layout_rec(child, child_layout))
            .collect()
    }

    pub fn measure(&mut self, ui_node: &UiNode, boundary_size: Vec2) -> Measurements {
        if let Some(measurements) = self.measurements_cache.get(ui_node, boundary_size) {
            measurements.clone()
        } else {
            let measurements = self.do_measure(ui_node, boundary_size);
            self.measurements_cache
                .insert(ui_node, boundary_size, measurements.clone());
            measurements
        }
    }

    fn do_measure(&mut self, ui_node: &UiNode, boundary_size: Vec2) -> Measurements {
        let modifiers = &ui_node.modifiers;

        let mut width = modifiers.width;
        let mut height = modifiers.height;

        let (mut margin_size, mut children_boundary_size) = self.resolve_extents(ui_node, boundary_size, width, height);

        if let Some(min_width) = modifiers.min_width {
            let (min_width_margin_size, min_width_children_boundary_size) =
                self.resolve_extents(ui_node, boundary_size, min_width, height);

            if margin_size.x < min_width_margin_size.x {
                width = min_width;
                margin_size = min_width_margin_size;
                children_boundary_size = min_width_children_boundary_size;
            }
        }

        if let Some(min_height) = modifiers.min_height {
            let (min_height_margin_size, min_height_children_boundary_size) =
                self.resolve_extents(ui_node, boundary_size, width, min_height);

            if margin_size.y < min_height_margin_size.y {
                height = min_height;
                margin_size = min_height_margin_size;
                children_boundary_size = min_height_children_boundary_size;
            }
        }

        if let Some(max_width) = modifiers.max_width {
            let (max_width_margin_size, max_width_children_boundary_size) =
                self.resolve_extents(ui_node, boundary_size, max_width, height);

            if margin_size.x > max_width_margin_size.x {
                width = max_width;
                margin_size = max_width_margin_size;
                children_boundary_size = max_width_children_boundary_size;
            }
        }

        if let Some(max_height) = modifiers.max_height {
            let (max_height_margin_size, max_height_children_boundary_size) =
                self.resolve_extents(ui_node, boundary_size, width, max_height);

            if margin_size.y > max_height_margin_size.y {
                // height = max_height;
                margin_size = max_height_margin_size;
                children_boundary_size = max_height_children_boundary_size;
            }
        }

        Measurements {
            margin_size,
            margin: modifiers.margin,
            border_thickness: modifiers.border_thickness,
            padding: modifiers.padding,
            children_boundary_size,
        }
    }

    fn resolve_extents(
        &mut self,
        ui_node: &UiNode,
        boundary_size: Vec2,
        width: Extent,
        height: Extent,
    ) -> (Vec2, Vec2) {
        let modifiers = &ui_node.modifiers;

        let margin_width = match width {
            Extent::FillParent => Some(boundary_size.x),
            Extent::Px(px) => Some(px.round() + modifiers.margin.delta_size().x),
            Extent::FitContent => None,
        };

        let margin_height = match height {
            Extent::FillParent => Some(boundary_size.y),
            Extent::Px(px) => Some(px.round() + modifiers.margin.delta_size().y),
            Extent::FitContent => None,
        };

        let (margin_size, children_boundary_size) = if margin_width.is_none() || margin_height.is_none() {
            self.measure_fit_content(ui_node, margin_width, margin_height)
        } else {
            (
                Vec2::new(margin_width.unwrap(), margin_height.unwrap()),
                Vec2::max(
                    Vec2::new(margin_width.unwrap(), margin_height.unwrap())
                        - modifiers.margin.delta_size()
                        - modifiers.border_thickness.delta_size()
                        - modifiers.padding.delta_size(),
                    Vec2::ZERO,
                ),
            )
        };

        margin_width.inspect(|&w| debug_assert_eq!(w, margin_size.x));
        margin_height.inspect(|&h| debug_assert_eq!(h, margin_size.y));

        (margin_size, children_boundary_size)
    }

    fn measure_fit_content(
        &mut self,
        ui_node: &UiNode,
        margin_width: Option<f32>,
        margin_height: Option<f32>,
    ) -> (Vec2, Vec2) {
        let modifiers = &ui_node.modifiers;

        let total_delta_size =
            modifiers.margin.delta_size() + modifiers.border_thickness.delta_size() + modifiers.padding.delta_size();

        let content_width = margin_width.map(|w| (w - total_delta_size.x).max(0.0));
        let content_height = margin_height.map(|h| (h - total_delta_size.y).max(0.0));

        let (content_size, children_boundary_size) = ui_node.props.measure_fit_content(
            &ui_node.modifiers,
            &ui_node.children,
            self,
            content_width,
            content_height,
        );

        let margin_size = Vec2::new(
            margin_width.unwrap_or_else(|| content_size.x + total_delta_size.x),
            margin_height.unwrap_or_else(|| content_size.y + total_delta_size.y),
        );

        (margin_size, children_boundary_size)
    }

    pub fn apply_horizontal_weights(
        &mut self,
        content_width: f32,
        children: &[UiNode],
        children_measurements: ChildrenMeasurements,
    ) -> ChildrenMeasurements {
        self.apply_weights(content_width, Axis::X, children, children_measurements)
    }

    pub fn apply_vertical_weights(
        &mut self,
        content_height: f32,
        children: &[UiNode],
        children_measurements: ChildrenMeasurements,
    ) -> ChildrenMeasurements {
        self.apply_weights(content_height, Axis::Y, children, children_measurements)
    }

    fn apply_weights(
        &mut self,
        content_extent: f32,
        extent_axis: Axis,
        children: &[UiNode],
        children_measurements: ChildrenMeasurements,
    ) -> ChildrenMeasurements {
        assert_eq!(children.len(), children_measurements.0.len());

        let total_children_weight: f32 = children.iter().map(|c| c.modifiers.weight.max(0.0)).sum();

        if total_children_weight == 0.0 {
            return children_measurements;
        }

        let total_children_extent: f32 = match extent_axis {
            Axis::X => children_measurements.width_sum(),
            Axis::Y => children_measurements.height_sum(),
        };

        let extra_extent = (content_extent - total_children_extent).max(0.0);
        if extra_extent == 0.0 {
            return children_measurements;
        }

        let mut remaining_extent = extra_extent;
        let mut remaining_children_weight = total_children_weight;

        let measurements: Vec<_> = std::iter::zip(children.iter(), children_measurements.into_iter())
            .map(|(child, mut child_measurements)| {
                if remaining_children_weight <= 0.0 {
                    return child_measurements;
                }

                let child_weight = child.modifiers.weight;
                let child_extra_extent = remaining_extent * (child_weight / remaining_children_weight);
                let child_extra_extent = child_extra_extent.ceil().min(remaining_extent);
                remaining_extent -= child_extra_extent;
                remaining_children_weight -= child_weight;

                let child_cross_extent = match extent_axis {
                    Axis::X => child.modifiers.height,
                    Axis::Y => child.modifiers.width,
                };

                if child_cross_extent == Extent::FitContent {
                    let (margin_size, children_boundary_size) = match extent_axis {
                        Axis::X => self.measure_fit_content(
                            child,
                            Some(child_measurements.margin_size.x + child_extra_extent),
                            None,
                        ),
                        Axis::Y => self.measure_fit_content(
                            child,
                            None,
                            Some(child_measurements.margin_size.y + child_extra_extent),
                        ),
                    };

                    child_measurements.margin_size = margin_size;
                    child_measurements.children_boundary_size = children_boundary_size;
                } else {
                    match extent_axis {
                        Axis::X => child_measurements.margin_size.x += child_extra_extent,
                        Axis::Y => child_measurements.margin_size.y += child_extra_extent,
                    };
                    child_measurements.children_boundary_size = child_measurements.content_size();
                }

                child_measurements
            })
            .collect();

        ChildrenMeasurements(measurements)
    }

    pub fn measure_children(&mut self, parent_size: Vec2, children: &[UiNode]) -> ChildrenMeasurements {
        return ChildrenMeasurements(children.iter().map(|c| self.measure(c, parent_size)).collect());
    }

    fn emit_rectangle(
        &mut self,
        position: Vec2,
        size: Vec2,
        fill_color: Color,
        border_color: Color,
        border_thickness: BorderThickness,
        border_radius: BorderRadius,
    ) {
        let rectangle = DrawElement::Rectangle {
            bounds: Rectangle::from_position_size(position, size),
            fill_color,
            border_color,
            border_radius: border_radius.to_vec4(),
            border_width: border_thickness.to_vec4(),
        };

        self.draw_data.push(rectangle);
    }
}

pub struct ChildrenMeasurements(Vec<Measurements>);

impl ChildrenMeasurements {
    pub fn max_width(&self) -> f32 {
        self.0
            .iter()
            .map(|cs| cs.margin_size.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn width_sum(&self) -> f32 {
        self.0.iter().map(|cm| cm.margin_size.x).sum()
    }

    pub fn max_height(&self) -> f32 {
        self.0
            .iter()
            .map(|cs| cs.margin_size.y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn height_sum(&self) -> f32 {
        self.0.iter().map(|cm| cm.margin_size.y).sum()
    }
}

impl IntoIterator for ChildrenMeasurements {
    type Item = Measurements;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
