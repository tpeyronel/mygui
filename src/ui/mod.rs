pub mod border_radius;
pub mod border_thickness;
pub mod color_mesh_builder;
pub mod draw_element;
pub mod immediate;
#[cfg(test)]
mod layout_tests;
pub mod margin;
mod measurements_cache;
pub mod mesh;
mod node;
pub mod padding;
mod processor;

use core::f32;

use border_radius::BorderRadius;
use border_thickness::BorderThickness;
use draw_element::DrawElement;
use glam::Vec2;
use margin::Margin;
use node::{block::BlockProps, column::ColumnProps, row::RowProps, text::TextProps, UiNode};
use padding::Padding;
use processor::UiNodeProcessor;

use crate::{color::Color, font::font_engine::FontEngine};

enum Axis {
    X,
    Y,
}

#[derive(Default, Debug, Clone, PartialEq)]
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
    border_color: Color,
    border_thickness: BorderThickness,
    border_radius: BorderRadius,
    self_alignment: Alignment,
    weight: f32,
    mask: Mask,
    children_mask: Mask,
}

#[allow(unused)]
impl Modifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(&mut self, width: Extent) -> &mut Self {
        self.width = width;
        self
    }

    pub fn height(&mut self, height: Extent) -> &mut Self {
        self.height = height;
        self
    }

    pub fn max_width(&mut self, max_width: Extent) -> &mut Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn max_height(&mut self, max_height: Extent) -> &mut Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn no_max_width(&mut self) -> &mut Self {
        self.max_width = None;
        self
    }

    pub fn no_max_height(&mut self) -> &mut Self {
        self.max_height = None;
        self
    }

    pub fn min_width(&mut self, min_width: Extent) -> &mut Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn min_height(&mut self, min_height: Extent) -> &mut Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn no_min_width(&mut self) -> &mut Self {
        self.min_width = None;
        self
    }

    pub fn no_min_height(&mut self) -> &mut Self {
        self.min_height = None;
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

    pub fn border_color(&mut self, border_color: Color) -> &mut Self {
        self.border_color = border_color;
        self
    }

    pub fn border_thickness(&mut self, border_thickness: BorderThickness) -> &mut Self {
        self.border_thickness = border_thickness;
        self
    }

    pub fn border_radius(&mut self, border_radius: BorderRadius) -> &mut Self {
        self.border_radius = border_radius;
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

    pub fn mask(&mut self, mask: Mask) -> &mut Self {
        self.mask = mask;
        self
    }

    pub fn children_mask(&mut self, children_mask: Mask) -> &mut Self {
        self.children_mask = children_mask;
        self
    }

    pub fn overflow_visible(&mut self) -> &mut Self {
        self.children_mask(Mask::Inherit)
    }

    pub fn overflow_hidden(&mut self) -> &mut Self {
        self.children_mask(Mask::InheritOpParent(LogicalOperator::And))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extent {
    FillParent,
    FitContent,
    Px(f32),
}

impl Default for Extent {
    fn default() -> Self {
        Self::FillParent
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
    font_engine: &mut Box<dyn FontEngine>,
    draw_elements: &mut Vec<DrawElement>,
) {
    let hash_nodes = create_mock_hash_tree_rec(&ui_nodes);
    let mut bounding_boxes = vec![];
    UiNodeProcessor::process_ui(
        ui_nodes,
        hash_nodes,
        boundary_pos,
        boundary_size,
        font_engine,
        draw_elements,
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
    margin: Margin,
    border_thickness: BorderThickness,
    padding: Padding,
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
    margin: Margin,
    border_thickness: BorderThickness,
    padding: Padding,
    children_boundary_size: Vec2,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mask {
    Inherit,
    None,
    Parent,
    InheritOpParent(LogicalOperator),
}

impl Default for Mask {
    fn default() -> Self {
        Self::Inherit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOperator {
    Not,
    And,
    Nand,
    Or,
    Nor,
    Eq, // Same as XNOR
    Ne, // Same as XOR
}

#[allow(unused)]
pub fn example_ui() -> UiNode {
    return UiNode::new(
        BlockProps,
        Modifiers::new()
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .fill_color(Color::rgba(0.1, 0.1, 1.0, 1.0))
            .margin(Margin::all(8.0))
            .padding(Padding::all(16.0))
            .fill_color(Color::rgba(1.0, 1.0, 0.1, 0.25))
            .border_radius(BorderRadius::all(16.0))
            .padding(Padding::all(16.0))
            .clone(),
        vec![UiNode::new(
            BlockProps,
            Modifiers::new()
                .width(Extent::FillParent)
                .height(Extent::FillParent)
                .padding(Padding::all(0.0))
                .fill_color(Color::rgba(1.0, 0.1, 0.1, 0.25))
                .border_color(Color::rgba(1.0, 0.1, 0.1, 0.9))
                .border_thickness(BorderThickness::all(4.0))
                .border_radius(BorderRadius::all(8.0))
                .clone(),
            vec![
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Center)
                        .fill_color(Color::rgba(1.0, 1.0, 1.0, 0.25))
                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0))
                        .clone(),
                    vec![],
                ),
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .margin(Margin::all(4.0))
                        .self_alignment(Alignment::Right)
                        .fill_color(Color::rgba(1.0, 0.0, 0.0, 0.25))
                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0))
                        .clone(),
                    vec![],
                ),
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::TopRight)
                        .fill_color(Color::rgba(1.0, 1.0, 0.0, 0.25))
                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::new(0.0, 8.0, 16.0, 24.0))
                        .clone(),
                    vec![],
                ),
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Top)
                        .fill_color(Color::rgba(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::new(4.0, 8.0, 12.0, 16.0))
                        .clone(),
                    vec![],
                ),
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::TopLeft)
                        .border_color(Color::rgba(1.0, 1.0, 1.0, 0.4))
                        .border_thickness(BorderThickness::all(2.0))
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .padding(Padding::all(8.0))
                                .border_color(Color::rgba(1.0, 0.0, 0.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0))
                                .clone(),
                            vec![UiNode::new(
                                BlockProps,
                                Modifiers::new().fill_color(Color::rgba(0.0, 1.0, 0.0, 0.4)).clone(),
                                vec![],
                            )],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .border_color(Color::rgba(0.0, 1.0, 0.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0))
                                .self_alignment(Alignment::BottomLeft)
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .border_color(Color::rgba(0.0, 0.0, 1.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0))
                                .self_alignment(Alignment::TopRight)
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Left)
                        .fill_color(Color::rgba(0.0, 0.0, 0.0, 0.25))
                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0))
                        .clone(),
                    vec![],
                ),
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::BottomLeft)
                        .fill_color(Color::rgba(0.0, 0.0, 1.0, 0.25))
                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0))
                        .clone(),
                    vec![],
                ),
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Bottom)
                        .fill_color(Color::rgba(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0))
                        .clone(),
                    vec![],
                ),
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::BottomRight)
                        .fill_color(Color::rgba(1.0, 0.0, 1.0, 0.25))
                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0))
                        .clone(),
                    vec![],
                ),
                UiNode::new(
                    RowProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .border_thickness(BorderThickness::all(4.0))
                        .border_color(Color::rgba(1.0, 1.0, 1.0, 1.0))
                        .clone(),
                    vec![
                        UiNode::new(
                            ColumnProps,
                            Modifiers::new()
                                .width(Extent::Px(256.0))
                                .height(Extent::FitContent)
                                .padding(Padding::all(16.0))
                                .border_thickness(BorderThickness::all(4.0))
                                .border_color(Color::rgba(1.0, 1.0, 1.0, 1.0))
                                .clone(),
                            vec![
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .height(Extent::Px(24.0))
                                        .fill_color(Color::rgba(0.0, 1.0, 1.0, 0.5))
                                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .height(Extent::FillParent)
                                        .fill_color(Color::rgba(1.0, 0.0, 1.0, 0.5))
                                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .height(Extent::Px(32.0))
                                        .fill_color(Color::rgba(1.0, 0.0, 0.0, 0.5))
                                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(96.0))
                                        .height(Extent::Px(64.0))
                                        .margin(Margin::all(8.0))
                                        .padding(Padding::all(8.0))
                                        .self_alignment(Alignment::Center)
                                        .fill_color(Color::rgba(1.0, 1.0, 0.0, 0.5))
                                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(4.0))
                                        .border_radius(BorderRadius::all(8.0))
                                        .clone(),
                                    vec![UiNode::new(
                                        BlockProps,
                                        Modifiers::new()
                                            .width(Extent::FillParent)
                                            .height(Extent::FillParent)
                                            .fill_color(Color::rgba(1.0, 1.0, 1.0, 0.5))
                                            .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                                            .border_thickness(BorderThickness::all(1.0))
                                            .border_radius(BorderRadius::all(8.0))
                                            .clone(),
                                        vec![],
                                    )],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .height(Extent::Px(32.0))
                                        .fill_color(Color::rgba(0.0, 1.0, 0.0, 0.5))
                                        .border_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0))
                                        .clone(),
                                    vec![],
                                ),
                            ],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::FillParent)
                                .fill_color(Color::rgba(0.25, 0.25, 1.0, 0.5))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::rgba(0.25, 0.25, 1.0, 0.25))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::rgba(0.25, 1.0, 0.25, 0.5))
                                .clone(),
                            vec![UiNode::new(
                                TextProps {
                                    text: "ÓThis is a text!\nÓWith 😊👍😭three lines\nÓThis is the last lineeeeeeeeee."
                                        .to_string(),
                                    text_color: Color::WHITE,
                                    font_family: "jetbrains mono".to_string(),
                                    font_size: 24.0,
                                    line_height: 24.0 * 1.5,
                                    cursor_position: None,
                                },
                                Modifiers::new().self_alignment(Alignment::TopLeft).clone(),
                                vec![],
                            )],
                        ),
                    ],
                ),
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::Px(256.0))
                        .self_alignment(Alignment::Left)
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::rgba(1.0, 0.0, 0.0, 0.4))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(2.0)
                                .fill_color(Color::rgba(0.0, 1.0, 0.0, 0.4))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::rgba(0.0, 0.0, 1.0, 0.4))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                UiNode::new(
                    ColumnProps,
                    Modifiers::new().height(Extent::FitContent).clone(),
                    vec![
                        UiNode::new(
                            RowProps,
                            Modifiers::new()
                                .height(Extent::Px(128.0))
                                .self_alignment(Alignment::Top)
                                .clone(),
                            vec![
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(1.0)
                                        .fill_color(Color::rgba(1.0, 0.0, 0.0, 0.4))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(2.0)
                                        .fill_color(Color::rgba(0.0, 1.0, 0.0, 0.4))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(1.0)
                                        .fill_color(Color::rgba(0.0, 0.0, 1.0, 0.4))
                                        .clone(),
                                    vec![UiNode::new(
                                        TextProps {
                                            text: "HellÓowjdoqi12931289😊👍😭3u!\nYegh".to_string(),
                                            text_color: Color::WHITE,
                                            font_family: "Segoe UI Emoji".to_string(),
                                            font_size: 24.0,
                                            line_height: 24.0,
                                            cursor_position: None,
                                        },
                                        Modifiers::new()
                                            .width(Extent::FillParent)
                                            .max_width(Extent::Px(512.0))
                                            .min_width(Extent::Px(256.0))
                                            .height(Extent::FitContent)
                                            .max_height(Extent::Px(512.0))
                                            .padding(Padding::all(64.0))
                                            .fill_color(Color::rgba(0.0, 1.0, 0.0, 0.5))
                                            .clone(),
                                        vec![],
                                    )],
                                ),
                            ],
                        ),
                        UiNode::new(
                            RowProps,
                            Modifiers::new()
                                .height(Extent::Px(128.0))
                                .self_alignment(Alignment::Top)
                                .clone(),
                            vec![
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(2.0)
                                        .fill_color(Color::rgba(1.0, 1.0, 0.0, 0.4))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(1.0)
                                        .fill_color(Color::rgba(0.0, 1.0, 1.0, 0.4))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(3.0)
                                        .fill_color(Color::rgba(1.0, 0.0, 1.0, 0.4))
                                        .clone(),
                                    vec![UiNode::new(
                                        TextProps {
                                            text: "HellÓowjdoqi129312893u!\nYegh".to_string(),
                                            text_color: Color::WHITE,
                                            font_family: "times new roman".to_string(),
                                            font_size: 17.0,
                                            line_height: 17.0,
                                            cursor_position: None,
                                        },
                                        Modifiers::new()
                                            .fill_color(Color::rgba(0.0, 0.0, 0.0, 0.5))
                                            .border_radius(BorderRadius::all(8.0))
                                            .padding(Padding::all(8.0))
                                            .clone(),
                                        vec![],
                                    )],
                                ),
                            ],
                        ),
                    ],
                ),
                UiNode::new(
                    ColumnProps,
                    Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomRight)
                        .clone(),
                    (5..32)
                        .map(|i| {
                            UiNode::new(
                                TextProps {
                                    text: "aAbBcCdDoOÓgfjpq".to_string(),
                                    text_color: Color::rgba(
                                        0.0 + (i - 5) as f32 / 31.0,
                                        (31 - i) as f32 / (26.0),
                                        1.0,
                                        1.0,
                                    ),
                                    font_family: "tangerine".to_string(),
                                    font_size: i as f32,
                                    line_height: i as f32,
                                    cursor_position: None,
                                },
                                Modifiers::new()
                                    .width(Extent::FitContent)
                                    .height(Extent::FitContent)
                                    .self_alignment(Alignment::Left)
                                    .fill_color(if i % 2 == 0 {
                                        Color::rgba(1.0, 0.0, 0.0, 0.5)
                                    } else {
                                        Color::rgba(0.0, 1.0, 0.0, 0.5)
                                    })
                                    .clone(),
                                vec![],
                            )
                        })
                        .collect(),
                ),
            ],
        )],
    );
}
