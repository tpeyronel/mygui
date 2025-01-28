pub mod border_color;
pub mod border_thickness;
pub mod color_mesh_builder;
pub mod corner_radius;
pub mod draw_command;
pub mod extent;
pub mod immediate;
mod inset;
#[cfg(test)]
mod layout_tests;
pub mod margin;
mod measurements_cache;
pub mod mesh_builder;
pub mod node;
pub mod padding;
mod processor;
pub mod shape;
pub mod texture_mesh_builder;

use core::f32;

use border_color::BorderColor;
use border_thickness::BorderThickness;
use corner_radius::CornerRadius;
use draw_command::DrawCommand;
use extent::{Extent, ExtentExt};
use glam::Vec2;
use inset::Inset;
use margin::Margin;
use node::{block::BlockProps, column::ColumnProps, row::RowProps, text::TextProps, UiNode};
use padding::Padding;
use processor::UiNodeProcessor;
use shape::{rounded_rectangle_shape::RoundedRectangleShape, Shape};

use crate::{color::Color, font::font_engine::FontEngine, mesh::mesh_manager::MeshManager};

enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Modifiers {
    width: Extent,
    height: Extent,
    max_width: Option<Extent>,
    max_height: Option<Extent>,
    min_width: Option<Extent>,
    min_height: Option<Extent>,
    margin: Margin,
    padding: Padding,
    fill_color: Color,
    border_color: BorderColor,
    border_thickness: BorderThickness,
    corner_radius: CornerRadius,
    shape: Box<dyn Shape>,
    self_alignment: Alignment,
    weight: f32,
    clip: Clip,
}

#[allow(unused)]
impl Modifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(&mut self, width: impl Into<Extent>) -> &mut Self {
        self.width = width.into();
        self
    }

    pub fn height(&mut self, height: impl Into<Extent>) -> &mut Self {
        self.height = height.into();
        self
    }

    pub fn max_width(&mut self, max_width: impl Into<Option<Extent>>) -> &mut Self {
        self.max_width = max_width.into();
        self
    }

    pub fn max_height(&mut self, max_height: impl Into<Option<Extent>>) -> &mut Self {
        self.max_height = max_height.into();
        self
    }

    pub fn min_width(&mut self, min_width: impl Into<Option<Extent>>) -> &mut Self {
        self.min_width = min_width.into();
        self
    }

    pub fn min_height(&mut self, min_height: impl Into<Option<Extent>>) -> &mut Self {
        self.min_height = min_height.into();
        self
    }

    pub fn margin(&mut self, margin: Margin) -> &mut Self {
        self.margin = margin;
        self
    }

    pub fn padding(&mut self, padding: Padding) -> &mut Self {
        self.padding = padding;
        self
    }

    pub fn fill_color(&mut self, fill_color: Color) -> &mut Self {
        self.fill_color = fill_color;
        self
    }

    pub fn border_color(&mut self, border_color: impl Into<BorderColor>) -> &mut Self {
        self.border_color = border_color.into();
        self
    }

    pub fn border_thickness(&mut self, border_thickness: BorderThickness) -> &mut Self {
        self.border_thickness = border_thickness;
        self
    }

    pub fn corner_radius(&mut self, corner_radius: CornerRadius) -> &mut Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn shape(&mut self, shape: impl Shape) -> &mut Self {
        self.shape = Box::new(shape);
        self
    }

    pub fn self_alignment(&mut self, self_alignment: Alignment) -> &mut Self {
        self.self_alignment = self_alignment;
        self
    }

    pub fn weight(&mut self, weight: f32) -> &mut Self {
        self.weight = weight;
        self
    }

    pub fn clip(&mut self, clip: Clip) -> &mut Self {
        self.clip = clip;
        self
    }

    pub fn overflow_visible(&mut self) -> &mut Self {
        self.clip(Clip::Inherit)
    }

    pub fn overflow_hidden(&mut self) -> &mut Self {
        self.clip(Clip::InheritAndShape)
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            max_width: Default::default(),
            max_height: Default::default(),
            min_width: Default::default(),
            min_height: Default::default(),
            margin: Default::default(),
            padding: Default::default(),
            fill_color: Default::default(),
            border_color: Default::default(),
            border_thickness: Default::default(),
            corner_radius: Default::default(),
            shape: Box::new(RoundedRectangleShape),
            self_alignment: Default::default(),
            weight: Default::default(),
            clip: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum Alignment {
    Center,
    Right,
    TopRight,
    Top,
    TopLeft,
    Left,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Center
    }
}

#[derive(Debug)]
pub struct HashNode {
    pub hash: u64,
    pub children: Vec<HashNode>,
}

#[allow(unused)]
pub fn to_draw_data_no_hashes(
    ui_nodes: Vec<UiNode>,
    boundary_pos: Vec2,
    boundary_size: Vec2,
    mesh_manager: &mut MeshManager,
    font_engine: &mut Box<dyn FontEngine>,
    command_list: &mut Vec<DrawCommand>,
) {
    let hash_nodes = create_mock_hash_tree_rec(&ui_nodes);
    let scale_factor = 1.0;
    let mut bounding_boxes = vec![];
    UiNodeProcessor::process_ui(
        ui_nodes,
        hash_nodes,
        boundary_pos,
        boundary_size,
        scale_factor,
        mesh_manager,
        font_engine,
        command_list,
        &mut bounding_boxes,
    );
}

fn create_mock_hash_tree_rec(ui_nodes: &[UiNode]) -> Vec<HashNode> {
    ui_nodes
        .iter()
        .map(|n| HashNode {
            hash: 0,
            children: create_mock_hash_tree_rec(&n.children),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutNode {
    layout: Layout,
    children: Vec<LayoutNode>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    margin_position: Vec2,
    margin_size: Vec2,
    children_boundary_size: Vec2,
    margin: Inset,
    border_thickness: Inset,
    padding: Inset,
}

impl Layout {
    #[allow(unused)]
    fn margin_position(&self) -> Vec2 {
        self.margin_position
    }

    #[allow(unused)]
    fn border_position(&self) -> Vec2 {
        self.margin_position + self.margin.delta_position()
    }

    #[allow(unused)]
    fn padding_position(&self) -> Vec2 {
        self.margin_position + self.margin.delta_position() + self.border_thickness.delta_position()
    }

    #[allow(unused)]
    fn content_position(&self) -> Vec2 {
        self.margin_position
            + self.margin.delta_position()
            + self.border_thickness.delta_position()
            + self.padding.delta_position() // TODO: clamp
    }

    #[allow(unused)]
    fn margin_size(&self) -> Vec2 {
        self.margin_size
    }

    #[allow(unused)]
    fn border_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size()).max(Vec2::ZERO)
    }

    #[allow(unused)]
    fn padding_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size()).max(Vec2::ZERO)
    }

    #[allow(unused)]
    fn content_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size() - self.padding.delta_size())
            .max(Vec2::ZERO)
    }

    #[allow(unused)]
    fn children_boundary_size(&self) -> Vec2 {
        self.children_boundary_size
    }

    #[allow(unused)]
    fn content_center(&self) -> Vec2 {
        self.content_position() + (self.content_size() * 0.5)
    }
}

#[derive(Debug, Clone)]
pub struct Measurements {
    margin_size: Vec2,
    children_boundary_size: Vec2,
    margin: Inset,
    border_thickness: Inset,
    padding: Inset,
}

impl Measurements {
    fn to_layout(&self, margin_position: Vec2) -> Layout {
        Layout {
            margin_position,
            margin_size: self.margin_size,
            children_boundary_size: self.children_boundary_size,
            margin: self.margin,
            border_thickness: self.border_thickness,
            padding: self.padding,
        }
    }

    #[allow(unused)]
    fn border_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size()).max(Vec2::ZERO)
    }

    #[allow(unused)]
    fn padding_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size()).max(Vec2::ZERO)
    }

    #[allow(unused)]
    fn content_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size() - self.padding.delta_size())
            .max(Vec2::ZERO)
    }

    fn total_delta_size(&self) -> Vec2 {
        self.margin.delta_size() + self.border_thickness.delta_size() + self.padding.delta_size()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clip {
    Inherit,
    None,
    Shape,
    InheritAndShape,
}

impl Default for Clip {
    fn default() -> Self {
        Self::Inherit
    }
}
