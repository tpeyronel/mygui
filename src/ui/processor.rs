use core::f32;
use std::u64;

use super::border_thickness::BorderThickness;
use super::draw_command::{DrawCommand, ScissorRectangle};
use super::extent::ExtrinsicExtent;
use super::inset::Inset;
use super::margin::Margin;
use super::measurements_cache::MeasurementsCache;
use super::padding::Padding;
use super::UiNode;
use super::{Axis, HashNode, LayoutNode, Measurements};
use glam::Vec2;

use crate::is_integer::IsInteger;
use crate::mesh::mesh::Mesh;
use crate::mesh::mesh_manager::MeshManager;
use crate::ui::draw_command::Shader;
use crate::ui::node::block::BlockProps;
use crate::ui::{Clip, Extent, Layout, Modifiers};
use crate::{font::font_engine::FontEngine, rectangle::Rectangle};

pub struct UiNodeProcessor<'a> {
    scale_factor: f32,
    measurements_cache: MeasurementsCache,
    pub font_engine: &'a mut Box<dyn FontEngine>,
    pub command_list_builder: CommandListBuilder<'a>,
    command_list: &'a mut Vec<DrawCommand>,
    bounding_boxes: &'a mut Vec<(u64, Rectangle)>,
}

impl<'a> UiNodeProcessor<'a> {
    pub fn process_ui(
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
        scale_factor: f32,
        mesh_manager: &'a mut MeshManager,
        font_engine: &'a mut Box<dyn FontEngine>,
        command_list: &'a mut Vec<DrawCommand>,
        bounding_boxes: &'a mut Vec<(u64, Rectangle)>,
    ) {
        let s = Self::new(scale_factor, mesh_manager, font_engine, command_list, bounding_boxes);
        s.to_draw_data(ui_nodes, hash_nodes, boundary_pos, boundary_size);
    }

    pub fn compute_layout_tree(
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
        scale_factor: f32,
        mesh_manager: &'a mut MeshManager,
        font_engine: &'a mut Box<dyn FontEngine>,
        command_list: &'a mut Vec<DrawCommand>,
        bounding_boxes: &'a mut Vec<(u64, Rectangle)>,
    ) -> LayoutNode {
        let mut s = Self::new(scale_factor, mesh_manager, font_engine, command_list, bounding_boxes);
        let (_, _, layout_node) = s.wrap_and_compute_layout_tree(ui_nodes, hash_nodes, boundary_pos, boundary_size);
        layout_node
    }

    fn new(
        scale_factor: f32,
        mesh_manager: &'a mut MeshManager,
        font_engine: &'a mut Box<dyn FontEngine>,
        command_list: &'a mut Vec<DrawCommand>,
        bounding_boxes: &'a mut Vec<(u64, Rectangle)>,
    ) -> Self {
        Self {
            scale_factor,
            measurements_cache: MeasurementsCache::new(),
            font_engine,
            command_list,
            bounding_boxes,
            command_list_builder: CommandListBuilder::new(mesh_manager),
        }
    }

    fn wrap_and_compute_layout_tree(
        &mut self,
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
    ) -> (UiNode, HashNode, LayoutNode) {
        let boundary_pos = boundary_pos.round();
        let boundary_size = boundary_size.round();

        let root_node = UiNode {
            props: Box::new(BlockProps),
            modifiers: Modifiers::new()
                .width(ExtrinsicExtent::Px(boundary_size.x))
                .height(ExtrinsicExtent::Px(boundary_size.y))
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
            margin: Inset::new(0.0, 0.0, 0.0, 0.0),
            border_thickness: Inset::new(0.0, 0.0, 0.0, 0.0),
            padding: Inset::new(0.0, 0.0, 0.0, 0.0),
        };

        let root_layout_node = self.compute_layout_rec(&root_node, root_layout);

        (root_node, root_hash_node, root_layout_node)
    }

    fn to_draw_data(
        mut self,
        ui_nodes: Vec<UiNode>,
        hash_nodes: Vec<HashNode>,
        boundary_pos: Vec2,
        boundary_size: Vec2,
    ) {
        let (root_node, root_hash_node, root_layout_node) =
            self.wrap_and_compute_layout_tree(ui_nodes, hash_nodes, boundary_pos, boundary_size);
        self.to_draw_data_rec(&root_node, &root_hash_node, &root_layout_node);

        self.command_list.clear();
        self.command_list.extend(self.command_list_builder.build());
    }

    fn to_draw_data_rec(&mut self, ui_node: &UiNode, hash_node: &HashNode, layout_node: &LayoutNode) {
        let modifiers = &ui_node.modifiers;
        let children = &ui_node.children;

        let layout = &layout_node.layout;

        let border_rectangle = Rectangle::from_position_size(layout.border_position(), layout.border_size());

        self.bounding_boxes.push((hash_node.hash, border_rectangle));

        let shape_mesh = modifiers.shape.to_shape_data(layout, modifiers);

        match &modifiers.clip {
            Clip::InheritAndShape => {
                self.command_list_builder
                    .push_scissor_rectangle(&ScissorRectangle::from_rectangle(&border_rectangle));
                self.command_list_builder
                    .draw_mesh(shape_mesh.background_mesh.clone(), Shader::ShapeClip);
                self.command_list_builder.inc_stencil_reference();
            }
            Clip::Inherit => {}
            Clip::None => todo!(),
            Clip::Shape => todo!(),
        }

        self.command_list_builder
            .draw_mesh(shape_mesh.background_mesh.clone(), Shader::Shape);

        ui_node.props.emit_draw_data(self, layout);

        debug_assert_eq!(children.len(), hash_node.children.len());
        debug_assert_eq!(children.len(), layout_node.children.len());
        for (i, c) in children.iter().enumerate() {
            self.to_draw_data_rec(c, &hash_node.children[i], &layout_node.children[i]);
        }

        match &modifiers.clip {
            Clip::InheritAndShape => {
                self.command_list_builder
                    .draw_mesh(shape_mesh.background_mesh, Shader::ShapeClipRevert);
                self.command_list_builder.dec_stencil_reference();
                self.command_list_builder.pop_scissor_rectangle();
            }
            Clip::Inherit => {}
            Clip::None => todo!(),
            Clip::Shape => todo!(),
        }

        self.command_list_builder
            .draw_mesh(shape_mesh.foreground_mesh, Shader::Shape);
    }

    fn compute_layout_rec(&mut self, ui_node: &UiNode, layout: Layout) -> LayoutNode {
        assert!(layout.margin_position.x.is_integer());
        assert!(layout.margin_position.y.is_integer());
        assert!(layout.margin_size.x.is_integer());
        assert!(layout.margin_size.y.is_integer());

        let children_layout_nodes = self.compute_children_layout_nodes(ui_node, &layout);

        LayoutNode {
            layout,
            children: children_layout_nodes,
        }
    }

    fn compute_children_layout_nodes(&mut self, ui_node: &UiNode, layout: &Layout) -> Vec<LayoutNode> {
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

        let all_insets = AllInsets::new(
            modifiers.margin,
            modifiers.border_thickness,
            modifiers.padding,
            boundary_size,
            self.scale_factor,
        );

        let mut width = modifiers.width;
        let mut height = modifiers.height;

        let (mut margin_size, mut children_boundary_size) =
            self.resolve_extents(ui_node, &all_insets, boundary_size, width, height);

        if let Some(min_width) = modifiers.min_width {
            let (min_width_margin_size, min_width_children_boundary_size) =
                self.resolve_extents(ui_node, &all_insets, boundary_size, min_width, height);

            if margin_size.x < min_width_margin_size.x {
                width = min_width;
                margin_size = min_width_margin_size;
                children_boundary_size = min_width_children_boundary_size;
            }
        }

        if let Some(min_height) = modifiers.min_height {
            let (min_height_margin_size, min_height_children_boundary_size) =
                self.resolve_extents(ui_node, &all_insets, boundary_size, width, min_height);

            if margin_size.y < min_height_margin_size.y {
                height = min_height;
                margin_size = min_height_margin_size;
                children_boundary_size = min_height_children_boundary_size;
            }
        }

        if let Some(max_width) = modifiers.max_width {
            let (max_width_margin_size, max_width_children_boundary_size) =
                self.resolve_extents(ui_node, &all_insets, boundary_size, max_width, height);

            if margin_size.x > max_width_margin_size.x {
                width = max_width;
                margin_size = max_width_margin_size;
                children_boundary_size = max_width_children_boundary_size;
            }
        }

        if let Some(max_height) = modifiers.max_height {
            let (max_height_margin_size, max_height_children_boundary_size) =
                self.resolve_extents(ui_node, &all_insets, boundary_size, width, max_height);

            if margin_size.y > max_height_margin_size.y {
                // height = max_height;
                margin_size = max_height_margin_size;
                children_boundary_size = max_height_children_boundary_size;
            }
        }

        Measurements {
            margin_size,
            children_boundary_size,
            margin: all_insets.margin,
            border_thickness: all_insets.border_thickness,
            padding: all_insets.padding,
        }
    }

    fn resolve_extents(
        &mut self,
        ui_node: &UiNode,
        insets: &AllInsets,
        boundary_size: Vec2,
        width: Extent,
        height: Extent,
    ) -> (Vec2, Vec2) {
        let margin_delta_size = insets.margin.delta_size();
        let margin_width = width.resolve_if_extrinsic(boundary_size.x, margin_delta_size.x, self.scale_factor);
        let margin_height = height.resolve_if_extrinsic(boundary_size.y, margin_delta_size.y, self.scale_factor);

        let (margin_size, children_boundary_size) = if margin_width.is_none() || margin_height.is_none() {
            self.measure_fit_content(ui_node, insets.total_delta_size, margin_width, margin_height)
        } else {
            let margin_size = Vec2::new(margin_width.unwrap(), margin_height.unwrap());
            let children_boundary_size = Vec2::max(margin_size - insets.total_delta_size, Vec2::ZERO);
            (margin_size, children_boundary_size)
        };

        margin_width.inspect(|&w| debug_assert_eq!(w, margin_size.x));
        margin_height.inspect(|&h| debug_assert_eq!(h, margin_size.y));

        (margin_size, children_boundary_size)
    }

    fn measure_fit_content(
        &mut self,
        ui_node: &UiNode,
        total_delta_size: Vec2,
        margin_width: Option<f32>,
        margin_height: Option<f32>,
    ) -> (Vec2, Vec2) {
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
                            child_measurements.total_delta_size(),
                            Some(child_measurements.margin_size.x + child_extra_extent),
                            None,
                        ),
                        Axis::Y => self.measure_fit_content(
                            child,
                            child_measurements.total_delta_size(),
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
}

struct AllInsets {
    margin: Inset,
    border_thickness: Inset,
    padding: Inset,
    total_delta_size: Vec2,
}

impl AllInsets {
    pub fn new(
        margin: Margin,
        border_thickness: BorderThickness,
        padding: Padding,
        boundary_size: Vec2,
        dp_factor: f32,
    ) -> Self {
        let margin = margin.resolve(boundary_size, dp_factor);
        let border_thickness = border_thickness.resolve(boundary_size, dp_factor);
        let padding = padding.resolve(boundary_size, dp_factor);

        Self {
            margin,
            border_thickness,
            padding,
            total_delta_size: margin.delta_size() + border_thickness.delta_size() + padding.delta_size(),
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

pub struct MeshWithShader(pub Mesh, pub Shader);

pub struct CommandListBuilder<'a> {
    commands: Vec<DrawCommand>,
    current_batch: Option<MeshWithShader>,
    mesh_manager: &'a mut MeshManager,
    stencil_reference: u32,
    scissor_stack: Vec<ScissorRectangle>,
}

impl<'a> CommandListBuilder<'a> {
    fn new(mesh_manager: &'a mut MeshManager) -> Self {
        Self {
            commands: Vec::new(),
            current_batch: None,
            mesh_manager,
            stencil_reference: 0,
            scissor_stack: Vec::new(),
        }
    }

    pub fn draw_mesh(&mut self, mesh: Mesh, shader: Shader) {
        if let Some(current_batch) = self.current_batch.as_mut() {
            let MeshWithShader(current_mesh, current_shader) = current_batch;

            if *current_shader == shader && current_mesh.image_data == mesh.image_data {
                assert_eq!(current_mesh.vertex_attributes.len(), mesh.vertex_attributes.len());
                for (attrib, data) in &mut current_mesh.vertex_attributes {
                    data.extend(&mesh.vertex_attributes[attrib]);
                }

                let base_index = current_mesh.vertex_count;
                current_mesh.indices.extend(mesh.indices.iter().map(|i| base_index + i));

                current_mesh.vertex_count += mesh.vertex_count;
            } else {
                self.flush_batch();
                self.current_batch = Some(MeshWithShader(mesh, shader));
            }
        } else {
            self.current_batch = Some(MeshWithShader(mesh, shader));
        }
    }

    fn flush_batch(&mut self) {
        let Some(completed_batch) = self.current_batch.take() else {
            return;
        };

        let mesh_id = self.mesh_manager.register_mesh(completed_batch.0);
        self.commands.push(DrawCommand::BindShader(completed_batch.1));
        self.commands.push(DrawCommand::DrawMesh(mesh_id));
    }

    pub fn push_scissor_rectangle(&mut self, rectangle: &ScissorRectangle) {
        let new_scissor = if let Some(prev_rect) = self.scissor_stack.last() {
            prev_rect.intersect(rectangle)
        } else {
            *rectangle
        };

        self.flush_batch();
        self.scissor_stack.push(new_scissor);
        self.commands.push(DrawCommand::SetScissor(new_scissor));
    }

    pub fn pop_scissor_rectangle(&mut self) {
        self.flush_batch();
        self.scissor_stack.pop();
        self.commands.push(DrawCommand::SetScissor(
            self.scissor_stack
                .last()
                .cloned()
                .unwrap_or(ScissorRectangle::NO_SCISSOR),
        ));
    }

    pub fn inc_stencil_reference(&mut self) {
        self.flush_batch();
        self.stencil_reference += 1;
        self.commands
            .push(DrawCommand::SetStencilReference(self.stencil_reference));
    }

    pub fn dec_stencil_reference(&mut self) {
        self.flush_batch();
        self.stencil_reference -= 1;
        self.commands
            .push(DrawCommand::SetStencilReference(self.stencil_reference));
    }

    pub fn build(mut self) -> Vec<DrawCommand> {
        if let Some(last_batch) = self.current_batch {
            let mesh_id = self.mesh_manager.register_mesh(last_batch.0);
            self.commands.push(DrawCommand::BindShader(last_batch.1));
            self.commands.push(DrawCommand::DrawMesh(mesh_id));
        }

        self.commands
    }
}
