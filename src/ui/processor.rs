use core::f32;
use std::u64;

use super::border_thickness::BorderThickness;
use super::color_mesh_builder::ColorMeshBuilder;
use super::draw_element::DrawElement;
use super::margin::Margin;
use super::measurements_cache::MeasurementsCache;
use super::padding::Padding;
use super::{border_radius::BorderRadius, UiNode};
use super::{Axis, HashNode, Measurements, UiNodeLayout};
use glam::{Vec2, Vec4};

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
    pub fn process_ui(
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
        font_engine: &'a mut Box<dyn FontEngine>,
        draw_data: &'a mut Vec<DrawElement>,
        bounding_boxes: &'a mut Vec<(u64, Rectangle)>,
    ) {
        let mut s = Self::new(font_engine, draw_data, bounding_boxes);
        s.to_draw_data(ui_nodes, hash_nodes, boundary_pos, boundary_size);
    }

    pub fn compute_layout_tree(
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
        font_engine: &'a mut Box<dyn FontEngine>,
        draw_data: &'a mut Vec<DrawElement>,
        bounding_boxes: &'a mut Vec<(u64, Rectangle)>,
    ) -> UiNodeLayout {
        let mut s = Self::new(font_engine, draw_data, bounding_boxes);
        let (_, _, layout_node) = s.wrap_and_compute_layout_tree(ui_nodes, hash_nodes, boundary_pos, boundary_size);
        layout_node
    }

    fn new(
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

    fn wrap_and_compute_layout_tree(
        &mut self,
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
    ) -> (UiNode, HashNode, UiNodeLayout) {
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

        (root_node, root_hash_node, root_layout_node)
    }

    fn to_draw_data(
        &mut self,
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
    ) {
        let (root_node, root_hash_node, root_layout_node) =
            self.wrap_and_compute_layout_tree(ui_nodes, hash_nodes, boundary_pos, boundary_size);
        self.to_draw_data_rec(&root_node, &root_hash_node, &root_layout_node);
        self.draw_data.remove(0); // TODO: remove. This is mainly done to simplify testing (the first rectangle is completely transparent).
    }

    fn to_draw_data_rec(&mut self, ui_node: &UiNode, hash_node: &HashNode, layout_node: &UiNodeLayout) {
        let modifiers = &ui_node.modifiers;
        let children = &ui_node.children;

        let layout = &layout_node.layout;

        self.emit_rectangle(
            layout,
            modifiers.fill_color,
            modifiers.border_color,
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

    fn emit_rectangle(&mut self, layout: &Layout, fill_color: Color, border_color: Color, border_radius: BorderRadius) {
        let mut builder = ColorMeshBuilder::new();

        let border_thickness = layout.border_thickness;

        let bl = CornerVertices::new(
            layout.border_position(),
            border_radius.bottom_left(),
            Vec2::new(border_thickness.left(), border_thickness.bottom()),
            Vec2::new(1.0, 1.0),
            border_color,
            fill_color,
            &mut builder,
        );

        let br = CornerVertices::new(
            layout.border_position() + layout.border_size().with_y(0.0),
            border_radius.bottom_right(),
            Vec2::new(border_thickness.right(), border_thickness.bottom()),
            Vec2::new(-1.0, 1.0),
            border_color,
            fill_color,
            &mut builder,
        );

        let tr = CornerVertices::new(
            layout.border_position() + layout.border_size(),
            border_radius.top_right(),
            Vec2::new(border_thickness.right(), border_thickness.top()),
            Vec2::new(-1.0, -1.0),
            border_color,
            fill_color,
            &mut builder,
        );

        let tl = CornerVertices::new(
            layout.border_position() + layout.border_size().with_x(0.0),
            border_radius.top_left(),
            Vec2::new(border_thickness.left(), border_thickness.top()),
            Vec2::new(1.0, -1.0),
            border_color,
            fill_color,
            &mut builder,
        );

        // Fill inner center
        builder.add_quad(bl.fill_hor.idx, br.fill_hor.idx, tr.fill_hor.idx, tl.fill_hor.idx);

        // Fill inner left side
        match (bl.fill_hor.idx != bl.fill_ver.idx, tl.fill_hor.idx != tl.fill_ver.idx) {
            (false, false) => (),
            (false, true) => builder.add_triangle(bl.fill_hor.idx, tl.fill_hor.idx, tl.fill_ver.idx),
            (true, false) => builder.add_triangle(bl.fill_ver.idx, bl.fill_hor.idx, tl.fill_hor.idx),
            (true, true) => builder.add_quad(bl.fill_ver.idx, bl.fill_hor.idx, tl.fill_hor.idx, tl.fill_ver.idx),
        }

        // Fill inner right side
        match (br.fill_hor.idx != br.fill_ver.idx, tr.fill_hor.idx != tr.fill_ver.idx) {
            (false, false) => (),
            (false, true) => builder.add_triangle(br.fill_hor.idx, tr.fill_ver.idx, tr.fill_hor.idx),
            (true, false) => builder.add_triangle(br.fill_hor.idx, br.fill_ver.idx, tr.fill_hor.idx),
            (true, true) => builder.add_quad(br.fill_hor.idx, br.fill_ver.idx, tr.fill_ver.idx, tr.fill_hor.idx),
        }

        /* Fill borders */
        /* TODO: optimize unnecessary vertices and triangles */

        // Bottom border
        builder.add_quad(
            bl.border_outer_hor.idx,
            br.border_outer_hor.idx,
            br.border_inner_hor.idx,
            bl.border_inner_hor.idx,
        );

        // Right border
        builder.add_quad(
            br.border_inner_ver.idx,
            br.border_outer_ver.idx,
            tr.border_outer_ver.idx,
            tr.border_inner_ver.idx,
        );

        // Top border
        builder.add_quad(
            tl.border_inner_hor.idx,
            tr.border_inner_hor.idx,
            tr.border_outer_hor.idx,
            tl.border_outer_hor.idx,
        );

        // Left border
        builder.add_quad(
            bl.border_outer_ver.idx,
            bl.border_inner_ver.idx,
            tl.border_inner_ver.idx,
            tl.border_outer_ver.idx,
        );

        /* Fill corners (with corner borders) */

        Self::emit_rectangle_corners(
            &bl,
            border_radius.bottom_left(),
            &fill_color,
            &border_color,
            &mut builder,
        );
        Self::emit_rectangle_corners(
            &br,
            border_radius.bottom_right(),
            &fill_color,
            &border_color,
            &mut builder,
        );
        Self::emit_rectangle_corners(&tr, border_radius.top_right(), &fill_color, &border_color, &mut builder);
        Self::emit_rectangle_corners(&tl, border_radius.top_left(), &fill_color, &border_color, &mut builder);

        let mesh = builder.build();

        self.draw_data.push(DrawElement::Mesh(mesh));
    }

    fn emit_rectangle_corners(
        v: &CornerVertices,
        corner_radius: f32,
        fill_color: &Color,
        border_color: &Color,
        builder: &mut ColorMeshBuilder,
    ) {
        if corner_radius > 0.0 {
            Self::emit_rectangle_corners_rec(
                v,
                0.0,
                v.border_outer_ver.idx,
                v.border_inner_ver.idx,
                v.fill_ver.idx,
                std::f32::consts::FRAC_PI_2,
                v.border_outer_hor.idx,
                v.border_inner_hor.idx,
                v.fill_hor.idx,
                Self::compute_corner_depth(corner_radius),
                fill_color,
                border_color,
                builder,
            );
        }
    }

    fn compute_corner_depth(corner_radius: f32) -> u32 {
        const MAGIC: f32 = 0.5;
        (MAGIC * corner_radius.log2()).round() as u32
    }

    fn emit_rectangle_corners_rec(
        v: &CornerVertices,
        left_angle: f32,
        left_border_outer_idx: u32,
        left_border_inner_idx: u32,
        left_fill_idx: u32,
        right_angle: f32,
        right_border_outer_idx: u32,
        right_border_inner_idx: u32,
        right_fill_idx: u32,
        depth: u32,
        fill_color: &Color,
        border_color: &Color,
        builder: &mut ColorMeshBuilder,
    ) {
        let alpha = (left_angle + right_angle) * 0.5;

        let (sin_alpha, cos_alpha) = alpha.sin_cos();

        let outer_pos = Vec2::new(
            lerp(v.border_outer_hor.pos.x, v.border_outer_ver.pos.x, cos_alpha),
            lerp(v.border_outer_ver.pos.y, v.border_outer_hor.pos.y, sin_alpha),
        );

        let inner_pos = Vec2::new(
            lerp(v.fill_hor.pos.x, v.fill_ver.pos.x, cos_alpha),
            lerp(v.fill_ver.pos.y, v.fill_hor.pos.y, sin_alpha),
        );

        let border_outer_idx = builder.add_vertex(outer_pos, *border_color);
        let border_inner_idx = builder.add_vertex(inner_pos, *border_color);
        let fill_idx = builder.add_vertex(inner_pos, *fill_color);

        builder.add_triangle(left_fill_idx, fill_idx, right_fill_idx);

        if depth > 0 {
            Self::emit_rectangle_corners_rec(
                v,
                left_angle,
                left_border_outer_idx,
                left_border_inner_idx,
                left_fill_idx,
                alpha,
                border_outer_idx,
                border_inner_idx,
                fill_idx,
                depth - 1,
                fill_color,
                border_color,
                builder,
            );
            Self::emit_rectangle_corners_rec(
                v,
                alpha,
                border_outer_idx,
                border_inner_idx,
                fill_idx,
                right_angle,
                right_border_outer_idx,
                right_border_inner_idx,
                right_fill_idx,
                depth - 1,
                fill_color,
                border_color,
                builder,
            );
        } else {
            // Emit border thickness
            builder.add_triangle(left_border_inner_idx, border_inner_idx, border_outer_idx);
            builder.add_triangle(left_border_inner_idx, border_outer_idx, left_border_outer_idx);
            builder.add_triangle(border_inner_idx, right_border_inner_idx, right_border_outer_idx);
            builder.add_triangle(border_inner_idx, right_border_outer_idx, border_outer_idx);
        }
    }
}

#[derive(Debug)]
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

fn lerp(x: f32, y: f32, a: f32) -> f32 {
    x * (1.0 - a) + y * a
}

struct CornerVertices {
    fill_hor: CornerVertex,
    fill_ver: CornerVertex,
    border_inner_hor: CornerVertex,
    border_inner_ver: CornerVertex,
    border_outer_hor: CornerVertex,
    border_outer_ver: CornerVertex,
}

impl CornerVertices {
    pub fn new(
        corner_pos: Vec2,
        corner_radius: f32,
        corner_thickness: Vec2,
        rotation: Vec2,
        border_color: Vec4,
        fill_color: Vec4,
        builder: &mut ColorMeshBuilder,
    ) -> Self {
        let fill_hor_pos = corner_pos + Vec2::new(corner_radius.max(corner_thickness.x), corner_thickness.y) * rotation;
        let fill_hor_idx = builder.add_vertex(fill_hor_pos, fill_color);

        let fill_ver_pos = corner_pos + Vec2::new(corner_thickness.x, corner_radius.max(corner_thickness.y)) * rotation;
        let fill_ver_idx = if fill_ver_pos == fill_hor_pos {
            fill_hor_idx
        } else {
            builder.add_vertex(fill_ver_pos, fill_color)
        };

        let border_inner_hor_pos = fill_hor_pos;
        let border_inner_hor_idx = builder.add_vertex(border_inner_hor_pos, border_color);

        let border_inner_ver_pos = fill_ver_pos;
        let border_inner_ver_idx = if border_inner_ver_pos == border_inner_hor_pos {
            border_inner_hor_idx
        } else {
            builder.add_vertex(border_inner_ver_pos, border_color)
        };

        let border_outer_hor_pos = corner_pos + Vec2::new(corner_radius, 0.0) * rotation;
        let border_outer_hor_idx = if border_outer_hor_pos == border_inner_hor_pos {
            border_inner_hor_idx
        } else {
            builder.add_vertex(border_outer_hor_pos, border_color)
        };

        let border_outer_ver_pos = corner_pos + Vec2::new(0.0, corner_radius) * rotation;
        let border_outer_ver_idx = if border_outer_ver_pos == border_outer_hor_pos {
            border_outer_hor_idx
        } else {
            builder.add_vertex(border_outer_ver_pos, border_color)
        };

        Self {
            fill_hor: CornerVertex {
                pos: fill_hor_pos,
                idx: fill_hor_idx,
            },
            fill_ver: CornerVertex {
                pos: fill_ver_pos,
                idx: fill_ver_idx,
            },
            border_inner_hor: CornerVertex {
                pos: border_inner_hor_pos,
                idx: border_inner_hor_idx,
            },
            border_inner_ver: CornerVertex {
                pos: border_inner_ver_pos,
                idx: border_inner_ver_idx,
            },
            border_outer_hor: CornerVertex {
                pos: border_outer_hor_pos,
                idx: border_outer_hor_idx,
            },
            border_outer_ver: CornerVertex {
                pos: border_outer_ver_pos,
                idx: border_outer_ver_idx,
            },
        }
    }
}

struct CornerVertex {
    pos: Vec2,
    idx: u32,
}

#[cfg(test)]
mod layout_tests {
    use crate::{
        font::mock_font_engine::MockFontEngine,
        ui::{create_mock_hash_tree_rec, Alignment},
    };

    use super::*;
    use pretty_assertions::assert_eq;

    fn compute_layout_nodes(position: Vec2, size: Vec2, ui: UiNode) -> Vec<UiNodeLayout> {
        let root_ui_nodes = vec![ui];
        let root_hash_nodes = create_mock_hash_tree_rec(&root_ui_nodes);

        let mut font_engine: Box<dyn FontEngine> = Box::new(MockFontEngine::new());
        let mut draw_data = vec![];
        let mut bounding_boxes = vec![];

        let root_layout_node = UiNodeProcessor::compute_layout_tree(
            root_ui_nodes,
            root_hash_nodes,
            position,
            size,
            &mut font_engine,
            &mut draw_data,
            &mut bounding_boxes,
        );

        root_layout_node.children
    }

    fn test_layout(width: f32, height: f32, ui: UiNode, expected: UiNodeLayout) {
        let layout_nodes = compute_layout_nodes(Vec2::ZERO, Vec2::new(width, height), ui);
        assert_eq!(&[expected], &layout_nodes.as_slice());
    }

    #[test]
    fn default_block() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(BlockProps, Modifiers::new(), vec![]),
            UiNodeLayout {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(32.0, 32.0),
                    ..Default::default()
                },
                children: vec![],
            },
        )
    }

    #[test]
    fn padding() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new().padding(Padding::all(8.0)).clone(),
                vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
            ),
            UiNodeLayout {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(16.0, 16.0),
                    padding: Padding::all(8.0),
                    ..Default::default()
                },
                children: vec![UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(8.0, 8.0),
                        margin_size: Vec2::new(16.0, 16.0),
                        children_boundary_size: Vec2::new(16.0, 16.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn margin() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(BlockProps, Modifiers::new().margin(Margin::all(8.0)).clone(), vec![]),
            UiNodeLayout {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(16.0, 16.0),
                    margin: Margin::all(8.0),
                    ..Default::default()
                },
                children: vec![],
            },
        )
    }

    #[test]
    fn full_padding() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new().padding(Padding::all(16.0)).clone(),
                vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
            ),
            UiNodeLayout {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(0.0, 0.0),
                    padding: Padding::all(16.0),
                    ..Default::default()
                },
                children: vec![UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(16.0, 16.0),
                        margin_size: Vec2::new(0.0, 0.0),
                        children_boundary_size: Vec2::new(0.0, 0.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn too_much_padding() {
        // TODO: restore this test.
        // test_converter(
        //     32.0,
        //     32.0,
        //     UiNode::new(
        //         BlockProps,
        //         Modifiers::new().padding(Padding::all(24.0)).clone(),
        //         vec![UiNode::new(
        //             BlockProps,
        //             Modifiers::new(),
        //             vec![],
        //         )],
        //     ),
        //     &[
        //         DrawElement::Rectangle {
        //             bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
        //             fill_color: Color::ZERO,
        //             border_color: Color::ZERO,
        //             border_radius: Vec4::ZERO,
        //             border_width: Vec4::ZERO,
        //         },
        //         DrawElement::Rectangle {
        //             bounds: Rectangle::from_position_size(Vec2::new(16.0, 16.0), Vec2::new(0.0, 0.0)),
        //             fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
        //             border_color: Color::ZERO,
        //             border_radius: Vec4::ZERO,
        //             border_width: Vec4::ZERO,
        //         },
        //     ],
        // )
    }

    #[test]
    fn self_alignment_basic() {
        fn test_self_alignment_basic(alignment: Alignment, expected_margin_position: Vec2) {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(8.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(alignment)
                        .clone(),
                    vec![],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: expected_margin_position,
                        margin_size: Vec2::new(8.0, 8.0),
                        children_boundary_size: Vec2::new(8.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![],
                },
            )
        }

        test_self_alignment_basic(Alignment::Center, Vec2::new(12.0, 12.0));
        test_self_alignment_basic(Alignment::Right, Vec2::new(24.0, 12.0));
        test_self_alignment_basic(Alignment::TopRight, Vec2::new(24.0, 24.0));
        test_self_alignment_basic(Alignment::Top, Vec2::new(12.0, 24.0));
        test_self_alignment_basic(Alignment::TopLeft, Vec2::new(0.0, 24.0));
        test_self_alignment_basic(Alignment::Left, Vec2::new(0.0, 12.0));
        test_self_alignment_basic(Alignment::BottomLeft, Vec2::new(0.0, 0.0));
        test_self_alignment_basic(Alignment::Bottom, Vec2::new(12.0, 0.0));
        test_self_alignment_basic(Alignment::BottomRight, Vec2::new(24.0, 0.0));
    }

    #[test]
    fn subpixel_alignment() {
        test_layout(
            15.0,
            15.0,
            UiNode::new(
                BlockProps,
                Modifiers::new().width(Extent::Px(8.0)).height(Extent::Px(8.0)).clone(),
                vec![],
            ),
            UiNodeLayout {
                layout: Layout {
                    margin_position: Vec2::new(4.0, 4.0),
                    margin_size: Vec2::new(8.0, 8.0),
                    children_boundary_size: Vec2::new(8.0, 8.0),
                    ..Default::default()
                },
                children: vec![],
            },
        );

        test_layout(
            17.0,
            17.0,
            // Duplicate ui (.clone() not available)
            UiNode::new(
                BlockProps,
                Modifiers::new().width(Extent::Px(8.0)).height(Extent::Px(8.0)).clone(),
                vec![],
            ),
            UiNodeLayout {
                layout: Layout {
                    margin_position: Vec2::new(5.0, 5.0),
                    margin_size: Vec2::new(8.0, 8.0),
                    children_boundary_size: Vec2::new(8.0, 8.0),
                    ..Default::default()
                },
                children: vec![],
            },
        );
    }

    #[test]
    fn self_alignment_with_parent_border_thickness() {
        fn test_self_alignment_basic(alignment: Alignment, expected_margin_position: Vec2) {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    // Border thickness of 4.0 makes the parent container equivalent to a 24.0 size container.
                    Modifiers::new().border_thickness(BorderThickness::all(4.0)).clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::Px(8.0))
                            .height(Extent::Px(8.0))
                            .self_alignment(alignment)
                            .clone(),
                        vec![],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(32.0, 32.0),
                        children_boundary_size: Vec2::new(24.0, 24.0),
                        border_thickness: BorderThickness::all(4.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: expected_margin_position,
                            margin_size: Vec2::new(8.0, 8.0),
                            children_boundary_size: Vec2::new(8.0, 8.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }

        test_self_alignment_basic(Alignment::Center, Vec2::new(12.0, 12.0));
        test_self_alignment_basic(Alignment::Right, Vec2::new(20.0, 12.0));
        test_self_alignment_basic(Alignment::TopRight, Vec2::new(20.0, 20.0));
        test_self_alignment_basic(Alignment::Top, Vec2::new(12.0, 20.0));
        test_self_alignment_basic(Alignment::TopLeft, Vec2::new(4.0, 20.0));
        test_self_alignment_basic(Alignment::Left, Vec2::new(4.0, 12.0));
        test_self_alignment_basic(Alignment::BottomLeft, Vec2::new(4.0, 4.0));
        test_self_alignment_basic(Alignment::Bottom, Vec2::new(12.0, 4.0));
        test_self_alignment_basic(Alignment::BottomRight, Vec2::new(20.0, 4.0));
    }

    mod blocks {
        use super::*;

        #[test]
        fn block_different_padding_values() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new().padding(Padding::new(1.0, 2.0, 4.0, 8.0)).clone(),
                    vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(32.0, 32.0),
                        children_boundary_size: Vec2::new(32.0 - 10.0, 32.0 - 5.0),
                        padding: Padding::new(1.0, 2.0, 4.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(8.0, 1.0),
                            margin_size: Vec2::new(32.0 - 10.0, 32.0 - 5.0),
                            children_boundary_size: Vec2::new(32.0 - 10.0, 32.0 - 5.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }

        #[test]
        fn block_different_border_thickness_values() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .border_thickness(BorderThickness::new(1.0, 2.0, 4.0, 8.0))
                        .clone(),
                    vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(32.0, 32.0),
                        children_boundary_size: Vec2::new(32.0 - 5.0, 32.0 - 10.0),
                        border_thickness: BorderThickness::new(1.0, 2.0, 4.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(1.0, 2.0),
                            margin_size: Vec2::new(32.0 - 5.0, 32.0 - 10.0),
                            children_boundary_size: Vec2::new(32.0 - 5.0, 32.0 - 10.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }

        #[test]
        fn block_fit_content_with_fill_parent_child() {
            test_layout(
                128.0,
                128.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight)
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(96.0, 64.0),
                        children_boundary_size: Vec2::new(96.0, 64.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(96.0, 64.0),
                                children_boundary_size: Vec2::new(96.0, 64.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(8.0, 64.0),
                                children_boundary_size: Vec2::new(8.0, 64.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 64.0 - 8.0),
                                margin_size: Vec2::new(96.0, 8.0),
                                children_boundary_size: Vec2::new(96.0, 8.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }

        #[test]
        fn block_fit_content_with_fill_parent_child_all_children_with_borders() {
            /* In this case, children having border should not affect in any way the parent size
            (aside from leaf layouts having different children boundary size and border thickness). */
            test_layout(
                128.0,
                128.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .border_thickness(BorderThickness::all(8.0))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .border_thickness(BorderThickness::all(8.0))
                                .self_alignment(Alignment::BottomLeft)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .border_thickness(BorderThickness::all(8.0))
                                .self_alignment(Alignment::TopRight)
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(96.0, 64.0),
                        children_boundary_size: Vec2::new(96.0, 64.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(96.0, 64.0),
                                children_boundary_size: Vec2::new(96.0 - 16.0, 64.0 - 16.0),
                                border_thickness: BorderThickness::all(8.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(8.0, 64.0),
                                children_boundary_size: Vec2::new(0.0, 64.0 - 16.0),
                                border_thickness: BorderThickness::all(8.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 64.0 - 8.0),
                                margin_size: Vec2::new(96.0, 8.0),
                                children_boundary_size: Vec2::new(96.0 - 16.0, 0.0),
                                border_thickness: BorderThickness::all(8.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }

        #[test]
        fn block_fit_content_with_fill_parent_child_parent_with_border() {
            test_layout(
                128.0,
                128.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .border_thickness(BorderThickness::all(8.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight)
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(96.0 + 16.0, 64.0 + 16.0),
                        children_boundary_size: Vec2::new(96.0, 64.0),
                        border_thickness: BorderThickness::all(8.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(8.0, 8.0),
                                margin_size: Vec2::new(96.0, 64.0),
                                children_boundary_size: Vec2::new(96.0, 64.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(8.0, 8.0),
                                margin_size: Vec2::new(8.0, 64.0),
                                children_boundary_size: Vec2::new(8.0, 64.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(8.0, 64.0),
                                margin_size: Vec2::new(96.0, 8.0),
                                children_boundary_size: Vec2::new(96.0, 8.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }

        #[test]
        fn block_fit_content_with_fill_parent_child_parent_with_padding() {
            /* Should be almost functionally equivalent to block_fit_content_with_fill_parent_child_parent_with_border */
            test_layout(
                128.0,
                128.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight)
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(96.0 + 16.0, 64.0 + 16.0),
                        children_boundary_size: Vec2::new(96.0, 64.0),
                        padding: Padding::all(8.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(8.0, 8.0),
                                margin_size: Vec2::new(96.0, 64.0),
                                children_boundary_size: Vec2::new(96.0, 64.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(8.0, 8.0),
                                margin_size: Vec2::new(8.0, 64.0),
                                children_boundary_size: Vec2::new(8.0, 64.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(8.0, 64.0),
                                margin_size: Vec2::new(96.0, 8.0),
                                children_boundary_size: Vec2::new(96.0, 8.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }
    }

    mod columns {
        use crate::ui::node::column::ColumnProps;

        use super::*;

        #[test]
        fn basic_column() {
            test_layout(
                32.0,
                128.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new(),
                    vec![
                        UiNode::new(BlockProps, Modifiers::new().height(Extent::Px(24.0)).clone(), vec![]),
                        UiNode::new(BlockProps, Modifiers::new().height(Extent::Px(48.0)).clone(), vec![]),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(32.0, 128.0),
                        children_boundary_size: Vec2::new(32.0, 128.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 128.0 - 24.0),
                                margin_size: Vec2::new(32.0, 24.0),
                                children_boundary_size: Vec2::new(32.0, 24.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                                margin_size: Vec2::new(32.0, 48.0),
                                children_boundary_size: Vec2::new(32.0, 48.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }

        #[test]
        fn basic_column_child_with_padding() {
            test_layout(
                32.0,
                128.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(24.0))
                                .padding(Padding::all(2.0))
                                .clone(),
                            vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
                        ),
                        UiNode::new(BlockProps, Modifiers::new().height(Extent::Px(48.0)).clone(), vec![]),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(32.0, 128.0),
                        children_boundary_size: Vec2::new(32.0, 128.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 128.0 - 24.0),
                                margin_size: Vec2::new(32.0, 24.0),
                                children_boundary_size: Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                                padding: Padding::all(2.0),
                                ..Default::default()
                            },
                            children: vec![UiNodeLayout {
                                layout: Layout {
                                    margin_position: Vec2::new(2.0, 128.0 - 24.0 + 2.0),
                                    margin_size: Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                                    children_boundary_size: Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                                    ..Default::default()
                                },
                                children: vec![],
                            }],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                                margin_size: Vec2::new(32.0, 48.0),
                                children_boundary_size: Vec2::new(32.0, 48.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }

        #[test]
        fn basic_column_single_child_with_margin() {
            test_layout(
                32.0,
                128.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .height(Extent::Px(16.0))
                            .margin(Margin::all(2.0))
                            .clone(),
                        vec![],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(32.0, 128.0),
                        children_boundary_size: Vec2::new(32.0, 128.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 128.0 - (16.0 + 4.0)),
                            margin_size: Vec2::new(32.0, 16.0 + 4.0),
                            children_boundary_size: Vec2::new(32.0 - 4.0, 16.0),
                            margin: Margin::all(2.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            );
        }

        #[test]
        fn column_both_fit_content_with_fill_parent_child() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(64.0, 64.0),
                        children_boundary_size: Vec2::new(64.0, 32.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 32.0),
                                margin_size: Vec2::new(64.0, 32.0),
                                children_boundary_size: Vec2::new(64.0, 32.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(64.0, 32.0),
                                children_boundary_size: Vec2::new(64.0, 32.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }

        #[test]
        fn column_fit_content_padding() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::FillParent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new().height(Extent::Px(32.0)).clone(),
                        vec![],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(256.0, 32.0 + 16.0),
                        children_boundary_size: Vec2::new(256.0 - 16.0, 32.0),
                        padding: Padding::all(8.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(8.0, 8.0),
                            margin_size: Vec2::new(256.0 - 16.0, 32.0),
                            children_boundary_size: Vec2::new(256.0 - 16.0, 32.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            );
        }

        #[test]
        fn column_fit_content_single_child_fill_parent() {
            test_layout(
                32.0,
                128.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::FillParent)
                            .height(Extent::FillParent)
                            .clone(),
                        vec![],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::ZERO,
                        children_boundary_size: Vec2::ZERO,
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::ZERO,
                            margin_size: Vec2::ZERO,
                            children_boundary_size: Vec2::ZERO,
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            );
        }

        #[test]
        fn column_fit_content_single_child_fill_parent_with_margin() {
            test_layout(
                32.0,
                128.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::FillParent)
                            .height(Extent::FillParent)
                            .margin(Margin::all(4.0))
                            .clone(),
                        vec![],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::ZERO,
                        children_boundary_size: Vec2::ZERO,
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::ZERO,
                            children_boundary_size: Vec2::ZERO,
                            margin: Margin::all(4.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            );
        }

        #[test]
        fn column_fit_content_hor_child_fill_parent() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::Px(48.0))
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(64.0, 48.0),
                        children_boundary_size: Vec2::new(64.0, 48.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(64.0, 48.0),
                                children_boundary_size: Vec2::new(64.0, 48.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, -48.0),
                                margin_size: Vec2::new(64.0, 48.0),
                                children_boundary_size: Vec2::new(64.0, 48.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }
    }

    mod rows {
        use crate::ui::node::row::RowProps;

        use super::*;

        #[test]
        fn row_both_fit_content_with_fill_parent_child() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    RowProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(128.0, 32.0),
                        children_boundary_size: Vec2::new(64.0, 32.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(64.0, 32.0),
                                children_boundary_size: Vec2::new(64.0, 32.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(64.0, 0.0),
                                margin_size: Vec2::new(64.0, 32.0),
                                children_boundary_size: Vec2::new(64.0, 32.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            );
        }
    }

    mod weight {
        use crate::ui::node::{column::ColumnProps, row::RowProps};

        use super::*;

        #[test]
        fn column_weight() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(100.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new().height(Extent::Px(20.0)).weight(1.0).clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new().height(Extent::Px(40.0)).weight(1.0).clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(50.0, 100.0),
                        children_boundary_size: Vec2::new(50.0, 100.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 60.0),
                                margin_size: Vec2::new(50.0, 40.0),
                                children_boundary_size: Vec2::new(50.0, 40.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(50.0, 60.0),
                                children_boundary_size: Vec2::new(50.0, 60.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            )
        }

        #[test]
        fn row_weight() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    RowProps,
                    Modifiers::new()
                        .width(Extent::Px(100.0))
                        .height(Extent::Px(50.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new().width(Extent::Px(20.0)).weight(1.0).clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new().width(Extent::Px(40.0)).weight(1.0).clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(100.0, 50.0),
                        children_boundary_size: Vec2::new(100.0, 50.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 00.0),
                                margin_size: Vec2::new(40.0, 50.0),
                                children_boundary_size: Vec2::new(40.0, 50.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(40.0, 0.0),
                                margin_size: Vec2::new(60.0, 50.0),
                                children_boundary_size: Vec2::new(60.0, 50.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            )
        }

        #[test]
        fn column_weight_rounding() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(100.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new().height(Extent::Px(0.0)).weight(1.0).clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new().height(Extent::Px(0.0)).weight(1.0).clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new().height(Extent::Px(0.0)).weight(1.0).clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(50.0, 100.0),
                        children_boundary_size: Vec2::new(50.0, 100.0),
                        ..Default::default()
                    },
                    children: vec![
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 100.0 - 34.0),
                                margin_size: Vec2::new(50.0, 34.0),
                                children_boundary_size: Vec2::new(50.0, 34.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 100.0 - 34.0 - 33.0),
                                margin_size: Vec2::new(50.0, 33.0),
                                children_boundary_size: Vec2::new(50.0, 33.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(0.0, 0.0),
                                margin_size: Vec2::new(50.0, 33.0),
                                children_boundary_size: Vec2::new(50.0, 33.0),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
            )
        }

        #[test]
        fn column_weight_respects_margin() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .margin(Margin::all(4.0)) // This margin should only allow for a height of 0.
                            .clone(),
                        vec![],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(50.0, 8.0),
                        children_boundary_size: Vec2::new(50.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(50.0, 8.0),
                            children_boundary_size: Vec2::new(50.0 - 8.0, 0.0),
                            margin: Margin::all(4.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }

        #[test]
        fn column_weight_respects_border_thickness() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .border_thickness(BorderThickness::all(3.0)) // This border thickness should only allow for a height of the child of 2.0.
                            .clone(),
                        vec![UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        )],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(50.0, 8.0),
                        children_boundary_size: Vec2::new(50.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(50.0, 8.0),
                            children_boundary_size: Vec2::new(50.0 - 6.0, 8.0 - 6.0),
                            border_thickness: BorderThickness::all(3.0),
                            ..Default::default()
                        },
                        children: vec![UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(3.0, 3.0),
                                margin_size: Vec2::new(44.0, 2.0),
                                children_boundary_size: Vec2::new(44.0, 2.0),
                                ..Default::default()
                            },
                            children: vec![],
                        }],
                    }],
                },
            )
        }

        #[test]
        fn column_weight_respects_padding() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .padding(Padding::all(3.0)) // This padding should only allow for a height of the child of 2.0.
                            .clone(),
                        vec![UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![],
                        )],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(50.0, 8.0),
                        children_boundary_size: Vec2::new(50.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(50.0, 8.0),
                            children_boundary_size: Vec2::new(50.0 - 6.0, 8.0 - 6.0),
                            padding: Padding::all(3.0),
                            ..Default::default()
                        },
                        children: vec![UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::new(3.0, 3.0),
                                margin_size: Vec2::new(44.0, 2.0),
                                children_boundary_size: Vec2::new(44.0, 2.0),
                                ..Default::default()
                            },
                            children: vec![],
                        }],
                    }],
                },
            )
        }

        #[test]
        fn column_weight_transfers_to_nested_children_correctly() {
            test_layout(
                256.0,
                256.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new().height(Extent::Px(0.0)).weight(1.0).clone(),
                        vec![UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .clone(),
                            vec![UiNode::new(
                                BlockProps,
                                Modifiers::new()
                                    .width(Extent::FillParent)
                                    .height(Extent::FillParent)
                                    .clone(),
                                vec![],
                            )],
                        )],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(50.0, 8.0),
                        children_boundary_size: Vec2::new(50.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::ZERO,
                            margin_size: Vec2::new(50.0, 8.0),
                            children_boundary_size: Vec2::new(50.0, 8.0),
                            ..Default::default()
                        },
                        children: vec![UiNodeLayout {
                            layout: Layout {
                                margin_position: Vec2::ZERO,
                                margin_size: Vec2::new(50.0, 8.0),
                                children_boundary_size: Vec2::new(50.0, 8.0),
                                ..Default::default()
                            },
                            children: vec![UiNodeLayout {
                                layout: Layout {
                                    margin_position: Vec2::ZERO,
                                    margin_size: Vec2::new(50.0, 8.0),
                                    children_boundary_size: Vec2::new(50.0, 8.0),
                                    ..Default::default()
                                },
                                children: vec![],
                            }],
                        }],
                    }],
                },
            )
        }
    }

    mod rounding {
        use crate::ui::node::column::ColumnProps;

        use super::*;

        #[test]
        fn border_thickness_rounding() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new().border_thickness(BorderThickness::all(3.5)).clone(), // Should all be rounded to 4.0
                    vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(32.0, 32.0),
                        children_boundary_size: Vec2::new(24.0, 24.0),
                        border_thickness: BorderThickness::all(3.5),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(4.0, 4.0),
                            margin_size: Vec2::new(24.0, 24.0),
                            children_boundary_size: Vec2::new(24.0, 24.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }

        #[test]
        fn padding_rounding() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new().padding(Padding::all(3.5)).clone(), // Should all be rounded to 4.0
                    vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(32.0, 32.0),
                        children_boundary_size: Vec2::new(24.0, 24.0),
                        padding: Padding::all(3.5),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(4.0, 4.0),
                            margin_size: Vec2::new(24.0, 24.0),
                            children_boundary_size: Vec2::new(24.0, 24.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }

        #[test]
        fn margin_rounding() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new().margin(Margin::all(3.5)).clone(), // Should all be rounded to 4.0
                    vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(32.0, 32.0),
                        children_boundary_size: Vec2::new(24.0, 24.0),
                        margin: Margin::all(3.5),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(4.0, 4.0),
                            margin_size: Vec2::new(24.0, 24.0),
                            children_boundary_size: Vec2::new(24.0, 24.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }

        #[test]
        fn root_rounding() {
            /* This test checks that the position and the size passed to ui.to_draw_data() get correctly rounded. */
            let root_position = Vec2::splat(0.5); // Should be rounded to (1.0, 1.0)
            let root_size = Vec2::splat(31.5); // Should be rounded to (32.0, 32.0)

            let ui = UiNode::new(BlockProps, Modifiers::new(), vec![]);

            let draw_data = compute_layout_nodes(root_position, root_size, ui);

            let expected = vec![UiNodeLayout {
                layout: Layout {
                    margin_position: Vec2::new(1.0, 1.0),
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(32.0, 32.0),
                    ..Default::default()
                },
                children: vec![],
            }];

            pretty_assertions::assert_eq!(&expected, &draw_data);
        }

        #[test]
        fn column_horizontal_rounding() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::Px(9.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::Px(8.0))
                            .self_alignment(Alignment::Center)
                            .clone(),
                        vec![],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(9.0, 8.0),
                        children_boundary_size: Vec2::new(9.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::new(1.0, 0.0),
                            margin_size: Vec2::new(8.0, 8.0),
                            children_boundary_size: Vec2::new(8.0, 8.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }
    }

    mod text {
        use glam::Vec2;

        use crate::ui::node::text::TextProps;

        use super::{test_layout, Alignment, Color, Extent, Layout, Modifiers, UiNode, UiNodeLayout};

        #[test]
        fn text_fit_content() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    TextProps {
                        text: "abcdef".into(),
                        text_color: Color::ONE,
                        font_family: String::new(),
                        font_size: 13.0,
                        line_height: 16.0,
                        cursor_position: None,
                    },
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(13.0 * 6.0, 16.0),
                        children_boundary_size: Vec2::ZERO,
                        ..Default::default()
                    },
                    children: vec![],
                },
            )
        }

        #[test]
        fn text_fit_content_with_max_width() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    TextProps {
                        text: "abcdef".into(),
                        text_color: Color::ONE,
                        font_family: String::new(),
                        font_size: 13.0,
                        line_height: 16.0,
                        cursor_position: None,
                    },
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .max_width(Extent::FillParent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(32.0, 16.0 * 3.0),
                        children_boundary_size: Vec2::ZERO,
                        ..Default::default()
                    },
                    children: vec![],
                },
            )
        }
    }

    mod row_advanced {
        use glam::Vec2;

        use crate::ui::node::{row::RowProps, text::TextProps};

        use super::{test_layout, Alignment, Color, Extent, Layout, Modifiers, UiNode, UiNodeLayout};

        /// This test checks that if a row child has non-zero weight, then
        /// when weight is applied, the height of the element is recomputed
        /// (and the height of the row itself too, as it is FitContent).
        /// In this case, if this were not the case, then because the initial
        /// size of the text is Px(0.0), the initially computed height of the text
        /// would be very big, as it would try to spread it vertically. But because
        /// we use weight(1.0), it should be equivalent to having specified the size
        /// of the text to be Px(32.0) / FillParent.
        #[test]
        fn row_with_text_extent_0_weight_1() {
            test_layout(
                32.0,
                32.0,
                UiNode::new(
                    RowProps,
                    Modifiers::new()
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![UiNode::new(
                        TextProps {
                            text: "abcdef".into(),
                            text_color: Color::ONE,
                            font_family: String::new(),
                            font_size: 13.0,
                            line_height: 16.0,
                            cursor_position: None,
                        },
                        Modifiers::new()
                            .width(Extent::Px(0.0))
                            .weight(1.0)
                            .height(Extent::FitContent)
                            .self_alignment(Alignment::BottomLeft)
                            .clone(),
                        vec![],
                    )],
                ),
                UiNodeLayout {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(32.0, 16.0 * 3.0),
                        children_boundary_size: Vec2::new(32.0, 48.0),
                        ..Default::default()
                    },
                    children: vec![UiNodeLayout {
                        layout: Layout {
                            margin_position: Vec2::ZERO,
                            margin_size: Vec2::new(32.0, 16.0 * 3.0),
                            children_boundary_size: Vec2::ZERO,
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                },
            )
        }
    }
}
