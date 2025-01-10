pub mod border_radius;
pub mod border_thickness;
pub mod color_mesh_builder;
pub mod draw_element;
pub mod immediate;
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

use crate::{font::font_engine::FontEngine, rectangle::Rectangle, vertex::Color};

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

pub struct UiNodeLayout {
    layout: Layout,
    children: Vec<UiNodeLayout>,
}

#[derive(Debug, Clone)]
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

#[allow(unused)]
pub fn example_ui() -> UiNode {
    return UiNode::new(
        BlockProps,
        Modifiers::new()
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .fill_color(Color::new(0.1, 0.1, 1.0, 1.0))
            .margin(Margin::all(8.0))
            .padding(Padding::all(16.0))
            .fill_color(Color::new(1.0, 1.0, 0.1, 0.25))
            .border_radius(BorderRadius::all(16.0))
            .padding(Padding::all(16.0))
            .clone(),
        vec![UiNode::new(
            BlockProps,
            Modifiers::new()
                .width(Extent::FillParent)
                .height(Extent::FillParent)
                .padding(Padding::all(0.0))
                .fill_color(Color::new(1.0, 0.1, 0.1, 0.25))
                .border_color(Color::new(1.0, 0.1, 0.1, 0.9))
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
                        .fill_color(Color::new(1.0, 1.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                        .fill_color(Color::new(1.0, 0.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                        .fill_color(Color::new(1.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                        .border_color(Color::new(1.0, 1.0, 1.0, 0.4))
                        .border_thickness(BorderThickness::all(2.0))
                        .clone(),
                    vec![
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .padding(Padding::all(8.0))
                                .border_color(Color::new(1.0, 0.0, 0.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0))
                                .clone(),
                            vec![UiNode::new(
                                BlockProps,
                                Modifiers::new().fill_color(Color::new(0.0, 1.0, 0.0, 0.4)).clone(),
                                vec![],
                            )],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .border_color(Color::new(0.0, 1.0, 0.0, 0.4))
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
                                .border_color(Color::new(0.0, 0.0, 1.0, 0.4))
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
                        .fill_color(Color::new(0.0, 0.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                        .fill_color(Color::new(0.0, 0.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                        .fill_color(Color::new(1.0, 0.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                        .border_color(Color::new(1.0, 1.0, 1.0, 1.0))
                        .clone(),
                    vec![
                        UiNode::new(
                            ColumnProps,
                            Modifiers::new()
                                .width(Extent::Px(256.0))
                                .height(Extent::FitContent)
                                .padding(Padding::all(16.0))
                                .border_thickness(BorderThickness::all(4.0))
                                .border_color(Color::new(1.0, 1.0, 1.0, 1.0))
                                .clone(),
                            vec![
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .height(Extent::Px(24.0))
                                        .fill_color(Color::new(0.0, 1.0, 1.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .height(Extent::FillParent)
                                        .fill_color(Color::new(1.0, 0.0, 1.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .height(Extent::Px(32.0))
                                        .fill_color(Color::new(1.0, 0.0, 0.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                                        .fill_color(Color::new(1.0, 1.0, 0.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(4.0))
                                        .border_radius(BorderRadius::all(8.0))
                                        .clone(),
                                    vec![UiNode::new(
                                        BlockProps,
                                        Modifiers::new()
                                            .width(Extent::FillParent)
                                            .height(Extent::FillParent)
                                            .fill_color(Color::new(1.0, 1.0, 1.0, 0.5))
                                            .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
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
                                .fill_color(Color::new(0.25, 0.25, 1.0, 0.5))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.25, 0.25, 1.0, 0.25))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.25, 1.0, 0.25, 0.5))
                                .clone(),
                            vec![UiNode::new(
                                TextProps {
                                    text: "ÓThis is a text!\nÓWith 😊👍😭three lines\nÓThis is the last lineeeeeeeeee."
                                        .to_string(),
                                    text_color: Color::ONE,
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
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.4))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(2.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.4))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 0.0, 1.0, 0.4))
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
                                        .fill_color(Color::new(1.0, 0.0, 0.0, 0.4))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(2.0)
                                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.4))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(1.0)
                                        .fill_color(Color::new(0.0, 0.0, 1.0, 0.4))
                                        .clone(),
                                    vec![UiNode::new(
                                        TextProps {
                                            text: "HellÓowjdoqi12931289😊👍😭3u!\nYegh".to_string(),
                                            text_color: Color::ONE,
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
                                            .fill_color(Color::new(0.0, 1.0, 0.0, 0.5))
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
                                        .fill_color(Color::new(1.0, 1.0, 0.0, 0.4))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(1.0)
                                        .fill_color(Color::new(0.0, 1.0, 1.0, 0.4))
                                        .clone(),
                                    vec![],
                                ),
                                UiNode::new(
                                    BlockProps,
                                    Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(3.0)
                                        .fill_color(Color::new(1.0, 0.0, 1.0, 0.4))
                                        .clone(),
                                    vec![UiNode::new(
                                        TextProps {
                                            text: "HellÓowjdoqi129312893u!\nYegh".to_string(),
                                            text_color: Color::ONE,
                                            font_family: "times new roman".to_string(),
                                            font_size: 17.0,
                                            line_height: 17.0,
                                            cursor_position: None,
                                        },
                                        Modifiers::new()
                                            .fill_color(Color::new(0.0, 0.0, 0.0, 0.5))
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
                                    text_color: Color::new(
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
                                        Color::new(1.0, 0.0, 0.0, 0.5)
                                    } else {
                                        Color::new(0.0, 1.0, 0.0, 0.5)
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

#[cfg(test)]
mod tests {
    use crate::font::mock_font_engine::MockFontEngine;

    use super::*;
    use glam::Vec4;
    use pretty_assertions::assert_eq;

    fn convert_to_draw_data(position: Vec2, size: Vec2, ui: UiNode) -> Vec<DrawElement> {
        let root_ui_nodes = vec![ui];
        let root_hash_nodes = create_mock_hash_tree_rec(&root_ui_nodes);

        let mut font_engine: Box<dyn FontEngine> = Box::new(MockFontEngine::new());
        let mut draw_data = vec![];
        let mut bounding_boxes = vec![];

        to_draw_data(
            root_ui_nodes,
            root_hash_nodes,
            position,
            size,
            &mut font_engine,
            &mut draw_data,
            &mut bounding_boxes,
        );
        draw_data
    }

    fn test_converter(width: f32, height: f32, ui: UiNode, expected: &[DrawElement]) {
        let draw_data = convert_to_draw_data(Vec2::ZERO, Vec2::new(width, height), ui);
        assert_eq!(expected, &draw_data);
    }

    #[test]
    fn default_block() {
        test_converter(
            32.0,
            32.0,
            UiNode::new(BlockProps, Modifiers::new(), vec![]),
            &[DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::ZERO,
                border_width: Vec4::ZERO,
            }],
        )
    }

    #[test]
    fn padding() {
        test_converter(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new().padding(Padding::all(8.0)).clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)).clone(),
                    vec![],
                )],
            ),
            &[
                DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
                DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(16.0, 16.0)),
                    fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
            ],
        )
    }

    #[test]
    fn margin() {
        test_converter(
            32.0,
            32.0,
            UiNode::new(BlockProps, Modifiers::new().margin(Margin::all(8.0)).clone(), vec![]),
            &[DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(16.0, 16.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::ZERO,
                border_width: Vec4::ZERO,
            }],
        )
    }

    #[test]
    fn full_padding() {
        test_converter(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new().padding(Padding::all(16.0)).clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)).clone(),
                    vec![],
                )],
            ),
            &[
                DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
                DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(Vec2::new(16.0, 16.0), Vec2::new(0.0, 0.0)),
                    fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
            ],
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
        //             Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)).clone(),
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
        fn test_self_alignment_basic(alignment: Alignment, expected_position: Vec2) {
            test_converter(
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
                &[DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(expected_position, Vec2::new(8.0, 8.0)),
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                }],
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
        let ui: UiNode = UiNode::new(
            BlockProps,
            Modifiers::new().width(Extent::Px(8.0)).height(Extent::Px(8.0)).clone(),
            vec![],
        );

        test_converter(
            15.0,
            15.0,
            ui,
            &[DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(8.0, 8.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::splat(0.0),
                border_width: Vec4::splat(0.0),
            }],
        );

        // Duplicate ui (.clone() not available)
        let ui: UiNode = UiNode::new(
            BlockProps,
            Modifiers::new().width(Extent::Px(8.0)).height(Extent::Px(8.0)).clone(),
            vec![],
        );

        test_converter(
            17.0,
            17.0,
            ui,
            &[DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(5.0, 5.0), Vec2::new(8.0, 8.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::splat(0.0),
                border_width: Vec4::splat(0.0),
            }],
        );
    }

    #[test]
    fn self_alignment_with_parent_border_thickness() {
        fn test_self_alignment_basic(alignment: Alignment, expected_position: Vec2) {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::ZERO, Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::splat(4.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(expected_position, Vec2::new(8.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
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

    mod blockes {
        use super::*;

        #[test]
        fn block_different_padding_values() {
            test_converter(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new().padding(Padding::new(1.0, 2.0, 4.0, 8.0)).clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)).clone(),
                        vec![],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 1.0), Vec2::new(32.0 - 10.0, 32.0 - 5.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn block_different_border_thickness_values() {
            test_converter(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .border_thickness(BorderThickness::new(1.0, 2.0, 4.0, 8.0))
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)).clone(),
                        vec![],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::new(1.0, 2.0, 4.0, 8.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0, 2.0), Vec2::new(32.0 - 5.0, 32.0 - 10.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn block_fit_content_with_fill_parent_child() {
            test_converter(
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
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(8.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 128.0 - 64.0 - 8.0), Vec2::new(96.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn block_fit_content_with_fill_parent_child_all_children_with_borders() {
            /* In this case, children having border should not affect in any way the parent size. */
            test_converter(
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
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(8.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 128.0 - 64.0 - 8.0), Vec2::new(96.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                ],
            );
        }

        #[test]
        fn block_fit_content_with_fill_parent_child_parent_with_border() {
            test_converter(
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
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0 + 16.0, 64.0 + 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(8.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0 + 64.0 - 8.0), Vec2::new(96.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn block_fit_content_with_fill_parent_child_parent_with_padding() {
            /* Should be functionally almost equivalent to block_fit_content_with_fill_parent_child_parent_with_border */
            test_converter(
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
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0 + 16.0, 64.0 + 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(8.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0 + 64.0 - 8.0), Vec2::new(96.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }
    }

    mod columns {
        use super::*;

        #[test]
        fn basic_column() {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 128.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 128.0 - 24.0), Vec2::new(32.0, 24.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                            Vec2::new(32.0, 48.0),
                        ),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn basic_column_child_with_padding() {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 128.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 128.0 - 24.0), Vec2::new(32.0, 24.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0 + 2.0, 128.0 - 24.0 + 2.0),
                            Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                        ),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                            Vec2::new(32.0, 48.0),
                        ),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn basic_column_single_child_with_margin() {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 128.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0 + 2.0, 128.0 - 16.0 - 2.0),
                            Vec2::new(32.0 - 4.0, 16.0),
                        ),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_both_fit_content_with_fill_parent_child() {
            test_converter(
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
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.0))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.0))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 32.0 + 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 32.0), Vec2::new(64.0, 32.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 32.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_fit_content_padding() {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(256.0, 32.0 + 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(256.0 - 16.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_fit_content_single_child_fill_parent() {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_fit_content_single_child_fill_parent_with_margin() {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(0.0, 0.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_fit_content_hor_child_fill_parent() {
            test_converter(
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
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.0))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.0))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 48.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 48.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, -48.0), Vec2::new(64.0, 48.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }
    }

    mod rows {
        use super::*;

        #[test]
        fn row_both_fit_content_with_fill_parent_child() {
            test_converter(
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
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.0))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.0))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0 + 64.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 32.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(64.0, 0.0), Vec2::new(64.0, 32.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }
    }

    mod weight {
        use super::*;

        #[test]
        fn column_weight() {
            test_converter(
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
                            Modifiers::new()
                                .height(Extent::Px(20.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(40.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 100.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0, 100.0 - 40.0),
                            Vec2::new(50.0, 20.0 + 20.0),
                        ),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 40.0 + 20.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn row_weight() {
            test_converter(
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
                            Modifiers::new()
                                .width(Extent::Px(20.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::Px(40.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(20.0 + 20.0, 50.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(40.0, 0.0), Vec2::new(40.0 + 20.0, 50.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_rounding() {
            test_converter(
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
                            Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0))
                                .clone(),
                            vec![],
                        ),
                        UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 0.0, 1.0, 1.0))
                                .clone(),
                            vec![],
                        ),
                    ],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 100.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 100.0 - 34.0), Vec2::new(50.0, 34.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0, 100.0 - 34.0 - 33.0),
                            Vec2::new(50.0, 33.0),
                        ),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 33.0)),
                        fill_color: Color::new(0.0, 0.0, 1.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_respects_margin() {
            test_converter(
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
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
                            .clone(),
                        vec![],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(50.0 - 8.0, 0.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_respects_border_thickness() {
            test_converter(
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
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
                            .clone(),
                        vec![UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0))
                                .clone(),
                            vec![],
                        )],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::splat(3.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(3.0, 3.0), Vec2::new(50.0 - 6.0, 2.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_respects_padding() {
            test_converter(
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
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
                            .clone(),
                        vec![UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0))
                                .clone(),
                            vec![],
                        )],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(3.0, 3.0), Vec2::new(50.0 - 6.0, 2.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_transfers_to_nested_children_correctly() {
            test_converter(
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
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
                            .clone(),
                        vec![UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0))
                                .clone(),
                            vec![UiNode::new(
                                BlockProps,
                                Modifiers::new()
                                    .width(Extent::FillParent)
                                    .height(Extent::FillParent)
                                    .fill_color(Color::new(0.0, 0.0, 1.0, 1.0))
                                    .clone(),
                                vec![],
                            )],
                        )],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(0.0, 0.0, 1.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }
    }

    mod rounding {
        use super::*;

        #[test]
        fn border_thickness_rounding() {
            test_converter(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new().border_thickness(BorderThickness::all(3.5)).clone(), // Should all be rounded to 4.0
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)).clone(),
                        vec![],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::splat(4.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(32.0 - 8.0, 32.0 - 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn padding_rounding() {
            test_converter(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new().padding(Padding::all(3.5)).clone(), // Should all be rounded to 4.0
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)).clone(),
                        vec![],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(32.0 - 8.0, 32.0 - 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn margin_rounding() {
            test_converter(
                32.0,
                32.0,
                UiNode::new(
                    BlockProps,
                    Modifiers::new().margin(Margin::all(3.5)).clone(), // Should all be rounded to 4.0
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)).clone(),
                        vec![],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(32.0 - 8.0, 32.0 - 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(32.0 - 8.0, 32.0 - 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn root_rounding() {
            /* This test checks that the position and the size passed to ui.to_draw_data() get correctly rounded. */
            let root_position = Vec2::splat(0.5); // Should be rounded to (1.0, 1.0)
            let root_size = Vec2::splat(31.5); // Should be rounded to (32.0, 32.0)

            let ui = UiNode::new(BlockProps, Modifiers::new(), vec![]);

            let draw_data = convert_to_draw_data(root_position, root_size, ui);

            let expected = vec![DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(1.0, 1.0), Vec2::new(32.0, 32.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::ZERO,
                border_width: Vec4::ZERO,
            }];

            pretty_assertions::assert_eq!(&expected, &draw_data);
        }

        #[test]
        fn column_horizontal_rounding() {
            test_converter(
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
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0))
                            .clone(),
                        vec![],
                    )],
                ),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(9.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0, 0.0), Vec2::new(8.0, 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }
    }

    mod text {
        use glam::{Vec2, Vec4};

        use crate::{font::font_face::GlyphPixelMode, image::image_manager::ImageId};

        use super::{test_converter, Alignment, Color, DrawElement, Extent, Modifiers, Rectangle, TextProps, UiNode};

        #[test]
        fn text_fit_content() {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(6.0 * 13.0, 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(2.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(3.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(5.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                ],
            )
        }

        #[test]
        fn text_fit_content_with_max_width() {
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 3.0 * 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0 * 13.0, 32.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0 * 13.0, 32.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0 * 13.0, 16.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0 * 13.0, 16.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                ],
            )
        }
    }

    mod row_advanced {
        use glam::{Vec2, Vec4};

        use crate::{font::font_face::GlyphPixelMode, image::image_manager::ImageId};

        use super::{
            test_converter, Alignment, Color, DrawElement, Extent, Modifiers, Rectangle, RowProps, TextProps, UiNode,
        };

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
            test_converter(
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
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 3.0 * 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 3.0 * 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0 * 13.0, 32.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0 * 13.0, 32.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0 * 13.0, 16.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0 * 13.0, 16.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                    DrawElement::TextGlyph {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0 * 13.0, 0.0), Vec2::new(13.0, 13.0)),
                        uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                        text_color: Color::ONE,
                        image_id: ImageId::NULL,
                        pixel_mode: GlyphPixelMode::Grayscale,
                    },
                ],
            )
        }
    }
}
