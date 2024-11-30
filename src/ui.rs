use glam::{Vec2, Vec4, Vec4Swizzles};

use crate::{rectangle::Rectangle, vertex::Color};

#[derive(Default, Debug, Clone)]
pub struct Modifiers {
    width: Extent,
    height: Extent,
    margin: Margin,
    padding: Padding,
    fill_color: Color,
    border_color: Color,
    border_thickness: BorderThickness,
    border_radius: BorderRadius,
    self_alignment: Alignment,
}

impl Modifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(self, width: Extent) -> Self {
        Self { width, ..self }
    }

    pub fn height(self, height: Extent) -> Self {
        Self { height, ..self }
    }

    pub fn margin(self, margin: Margin) -> Self {
        Self { margin, ..self }
    }

    pub fn padding(self, padding: Padding) -> Self {
        Self { padding, ..self }
    }

    pub fn fill_color(self, fill_color: Color) -> Self {
        Self { fill_color, ..self }
    }

    pub fn border_color(self, border_color: Color) -> Self {
        Self { border_color, ..self }
    }

    pub fn border_thickness(self, border_thickness: BorderThickness) -> Self {
        Self {
            border_thickness,
            ..self
        }
    }

    pub fn border_radius(self, border_radius: BorderRadius) -> Self {
        Self { border_radius, ..self }
    }

    pub fn self_alignment(self, self_alignment: Alignment) -> Self {
        Self { self_alignment, ..self }
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

#[derive(Debug, Clone, Copy)]
pub struct BorderThickness {
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub left: f32,
}

impl BorderThickness {
    fn all(x: f32) -> Self {
        Self {
            bottom: x,
            right: x,
            top: x,
            left: x,
        }
    }

    fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.bottom, self.right, self.top, self.left)
    }

    fn delta_size(&self) -> Vec2 {
        Vec2::new(self.left + self.right, self.bottom + self.right).round()
    }

    fn delta_position(&self) -> Vec2 {
        Vec2::new(self.left, self.bottom).round()
    }
}

impl Default for BorderThickness {
    fn default() -> Self {
        Self {
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
            left: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BorderRadius {
    pub bottom_left: f32,
    pub bottom_right: f32,
    pub top_right: f32,
    pub top_left: f32,
}

impl BorderRadius {
    fn all(x: f32) -> Self {
        Self {
            bottom_left: x,
            bottom_right: x,
            top_right: x,
            top_left: x,
        }
    }

    fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.bottom_left, self.bottom_right, self.top_right, self.top_left)
    }
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self {
            bottom_left: 0.0,
            bottom_right: 0.0,
            top_right: 0.0,
            top_left: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Padding {
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub left: f32,
}

impl Padding {
    fn all(x: f32) -> Self {
        Self {
            bottom: x,
            right: x,
            top: x,
            left: x,
        }
    }

    fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.bottom, self.right, self.top, self.left)
    }

    fn delta_size(&self) -> Vec2 {
        Vec2::new(self.left + self.right, self.bottom + self.right).round()
    }

    fn delta_position(&self) -> Vec2 {
        Vec2::new(self.left, self.bottom).round()
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
            left: 0.0,
        }
    }
}

type Margin = Padding;

#[derive(Debug, Clone)]
pub struct BoxProps {
    modifiers: Modifiers,
    children: Vec<UiNode>,
}

impl Default for BoxProps {
    fn default() -> Self {
        Self {
            modifiers: Modifiers::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnProps {
    modifiers: Modifiers,
    children: Vec<UiNode>,
}

impl Default for ColumnProps {
    fn default() -> Self {
        Self {
            modifiers: Modifiers::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub enum UiNode {
    Box(BoxProps),
    Column(ColumnProps),
}

impl UiNode {
    pub fn to_draw_data(self, boundary_pos: Vec2, boundary_size: Vec2, out: &mut Vec<Rectangle>) {
        let root_node = UiNode::Box(BoxProps {
            modifiers: Modifiers::new()
                .width(Extent::Px(boundary_size.x))
                .height(Extent::Px(boundary_size.y)),
            children: vec![self],
        });

        let root_measurements = Measurements {
            margin_size: boundary_size,
            border_size: boundary_size,
            content_size: boundary_size,
            children_boundary_size: boundary_size,
        };

        let root_layout = Layout {
            measurements: root_measurements,
            margin_position: boundary_pos,
            border_position: boundary_pos,
            content_position: boundary_pos,
        };

        let root_layout_node = root_node.compute_layout_rec(root_layout);
        root_node.to_draw_data_rec(&root_layout_node, out);
        out.remove(0); // TODO: remove. This is mainly done to simplify testing (the first rectangle is completely transparent).
    }

    fn to_draw_data_rec(&self, layout_node: &UiNodeLayout, draw_data: &mut Vec<Rectangle>) {
        let (modifiers, children) = match self {
            UiNode::Box(props) => (&props.modifiers, &props.children),
            UiNode::Column(props) => (&props.modifiers, &props.children),
        };

        let layout = &layout_node.layout;
        let measurements = &layout.measurements;

        Self::emit_rectangle(
            layout.border_position,
            measurements.border_size,
            modifiers.fill_color,
            modifiers.border_color,
            modifiers.border_thickness,
            modifiers.border_radius,
            draw_data,
        );

        for (i, c) in children.iter().enumerate() {
            c.to_draw_data_rec(&layout_node.children[i], draw_data);
        }
    }

    fn compute_layout_rec(&self, layout: Layout) -> UiNodeLayout {
        let children_layouts = Self::compute_children_layouts(&self, &layout);

        UiNodeLayout {
            layout,
            children: children_layouts,
        }
    }

    fn compute_children_layouts(&self, layout: &Layout) -> Vec<UiNodeLayout> {
        let content_size = layout.measurements.content_size;
        let content_position = layout.content_position;
        let content_center = content_position + (content_size * 0.5);

        match self {
            UiNode::Box(props) => props
                .children
                .iter()
                .map(|c| {
                    let child_measurements = c.measure(layout.measurements.children_boundary_size);
                    let child_margin_size = child_measurements.margin_size;
                    let child_modifiers = match c {
                        UiNode::Box(props) => &props.modifiers,
                        UiNode::Column(props) => &props.modifiers,
                    };
                    let child_margin_position = match child_modifiers.self_alignment {
                        Alignment::Center => (content_center - 0.5 * child_margin_size).round(),
                        Alignment::Right => Vec2::new(
                            content_position.x + content_size.x - child_margin_size.x,
                            content_center.y - 0.5 * child_margin_size.y,
                        )
                        .round(),
                        Alignment::TopRight => (content_position + content_size - child_margin_size).round(),
                        Alignment::Top => Vec2::new(
                            content_center.x - 0.5 * child_margin_size.x,
                            content_position.y + content_size.y - child_margin_size.y,
                        )
                        .round(),
                        Alignment::TopLeft => Vec2::new(
                            content_position.x,
                            content_position.y + content_size.y - child_margin_size.y,
                        )
                        .round(),
                        Alignment::Left => {
                            Vec2::new(content_position.x, content_center.y - 0.5 * child_margin_size.y).round()
                        }
                        Alignment::BottomLeft => content_position.round(),
                        Alignment::Bottom => {
                            Vec2::new(content_center.x - 0.5 * child_margin_size.x, content_position.y).round()
                        }
                        Alignment::BottomRight => Vec2::new(
                            content_position.x + content_size.x - child_margin_size.x,
                            content_position.y,
                        )
                        .round(),
                    };

                    let child_border_position = child_margin_position + child_modifiers.margin.delta_position();
                    let child_padding_position =
                        child_border_position + child_modifiers.border_thickness.delta_position();
                    let child_content_position = child_padding_position + child_modifiers.padding.delta_position(); // TODO: clamp

                    let child_layout = Layout {
                        measurements: child_measurements,
                        margin_position: child_margin_position,
                        border_position: child_border_position,
                        content_position: child_content_position,
                    };

                    c.compute_layout_rec(child_layout)
                })
                .collect(),
            UiNode::Column(props) => {
                let mut vertical_offset = 0.0;
                let column_top = content_position.y + content_size.y;
                props
                    .children
                    .iter()
                    .map(|c| {
                        let child_measurements = c.measure(layout.measurements.children_boundary_size);
                        let child_margin_size = child_measurements.margin_size;
                        vertical_offset += child_margin_size.y;

                        let child_modifiers = match c {
                            UiNode::Box(props) => &props.modifiers,
                            UiNode::Column(props) => &props.modifiers,
                        };

                        let child_margin_position = match child_modifiers.self_alignment {
                            Alignment::TopLeft | Alignment::Left | Alignment::BottomLeft => {
                                Vec2::new(content_position.x, column_top - vertical_offset)
                            }
                            Alignment::Top | Alignment::Center | Alignment::Bottom => Vec2::new(
                                content_center.x - 0.5 * child_margin_size.x,
                                column_top - vertical_offset,
                            ),
                            Alignment::TopRight | Alignment::Right | Alignment::BottomRight => Vec2::new(
                                content_position.x + content_size.x - child_margin_size.x,
                                column_top - vertical_offset,
                            ),
                        };

                        let child_border_position = child_margin_position + child_modifiers.margin.delta_position();
                        let child_padding_position =
                            child_border_position + child_modifiers.border_thickness.delta_position();
                        let child_content_position = child_padding_position + child_modifiers.padding.delta_position(); // TODO: clamp

                        let child_layout = Layout {
                            measurements: child_measurements,
                            margin_position: child_margin_position,
                            border_position: child_border_position,
                            content_position: child_content_position,
                        };

                        c.compute_layout_rec(child_layout)
                    })
                    .collect()
            }
        }
    }

    fn measure(&self, boundary_size: Vec2) -> Measurements {
        let modifiers = match self {
            UiNode::Box(props) => &props.modifiers,
            UiNode::Column(props) => &props.modifiers,
        };

        let children = match self {
            UiNode::Box(props) => &props.children,
            UiNode::Column(props) => &props.children,
        };
        // We subtract it here as the only special case is Extent::FillParent.
        let boundary_size = (boundary_size - modifiers.margin.delta_size()).max(Vec2::ZERO);

        let preliminar_children_boundary_size = Vec2::new(
            match modifiers.width {
                Extent::FitContent => 0.0,
                Extent::FillParent => boundary_size.x,
                Extent::Px(px) => px.round(),
            },
            match modifiers.height {
                Extent::FitContent => 0.0,
                Extent::FillParent => boundary_size.y,
                Extent::Px(px) => px.round(),
            },
        );
        let preliminar_children_boundary_size =
            (preliminar_children_boundary_size - modifiers.border_thickness.delta_size() - modifiers.padding.delta_size())
                .max(Vec2::ZERO);
        // TODO: only compute when necessary
        let min_intrinsic_children_sizes: Vec<Measurements> = Self::measure_children(preliminar_children_boundary_size, children);

        let mut children_boundary_size = preliminar_children_boundary_size;
        let computed_width = match modifiers.width {
            Extent::FillParent => boundary_size.x,
            Extent::FitContent => match self {
                UiNode::Box(_) | UiNode::Column(_) => {
                    let max_child_width = min_intrinsic_children_sizes
                        .iter()
                        .map(|cs| cs.margin_size.x)
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.0);
                    children_boundary_size.x = max_child_width;
                    max_child_width + modifiers.padding.delta_size().x + modifiers.border_thickness.delta_size().x
                }
            },
            Extent::Px(px) => px.round(),
        };

        let computed_height = match modifiers.height {
            Extent::FillParent => boundary_size.y,
            Extent::FitContent => match self {
                UiNode::Box(_) => {
                    let max_child_height = min_intrinsic_children_sizes
                        .iter()
                        .map(|cs| cs.margin_size.y)
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.0);
                    children_boundary_size.y = max_child_height;
                    max_child_height + modifiers.padding.delta_size().y + modifiers.border_thickness.delta_size().y
                }
                UiNode::Column(_) => {
                    let min_intrinsic_height = min_intrinsic_children_sizes.iter().map(|cs| cs.margin_size.y).sum();
                    children_boundary_size.y = min_intrinsic_height;
                    let children_sizes = Self::measure_children(children_boundary_size, children);
                    children_sizes.iter().map(|cs| cs.margin_size.y).sum::<f32>()
                        + modifiers.padding.delta_size().y
                        + modifiers.border_thickness.delta_size().y
                }
            },
            Extent::Px(px) => px.round(),
        };

        let border_size = Vec2::new(computed_width, computed_height);
        let margin_size = border_size + modifiers.margin.delta_size();
        let padding_size = (border_size - modifiers.border_thickness.delta_size()).max(Vec2::ZERO);
        let content_size = (padding_size - modifiers.padding.delta_size()).max(Vec2::ZERO);

        Measurements {
            margin_size,
            border_size,
            content_size,
            children_boundary_size,
        }
    }

    fn measure_children(parent_size: Vec2, children: &[UiNode]) -> Vec<Measurements> {
        return children.iter().map(|c| c.measure(parent_size)).collect();
    }

    fn emit_rectangle(
        position: Vec2,
        size: Vec2,
        fill_color: Color,
        border_color: Color,
        border_thickness: BorderThickness,
        border_radius: BorderRadius,
        out: &mut Vec<Rectangle>,
    ) {
        let rectangle = Rectangle {
            position,
            size,
            fill_color,
            border_color,
            border_radius: border_radius.to_vec4(),
            border_width: border_thickness.to_vec4(),
        };

        out.push(rectangle);
    }
}

pub struct UiNodeLayout {
    layout: Layout,
    children: Vec<UiNodeLayout>,
}

struct Layout {
    measurements: Measurements,
    margin_position: Vec2,
    border_position: Vec2,
    content_position: Vec2,
}

#[derive(Debug)]
struct Measurements {
    margin_size: Vec2,
    border_size: Vec2,
    // padding_size: Vec2,
    content_size: Vec2,
    children_boundary_size: Vec2, // This is the size that was used to measure the children of the node.
}

pub fn example_ui() -> UiNode {
    return UiNode::Box(BoxProps {
        modifiers: Modifiers::new()
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .fill_color(Color::new(0.1, 0.1, 1.0, 1.0))
            .margin(Margin::all(8.0))
            .padding(Padding::all(16.0))
            .fill_color(Color::new(1.0, 1.0, 0.1, 0.25))
            .border_radius(BorderRadius::all(16.0))
            .padding(Padding::all(16.0)),
        children: vec![UiNode::Box(BoxProps {
            modifiers: Modifiers::new()
                .width(Extent::FillParent)
                .height(Extent::FillParent)
                .padding(Padding::all(0.0))
                .fill_color(Color::new(1.0, 0.1, 0.1, 0.25))
                .border_color(Color::new(1.0, 0.1, 0.1, 0.9))
                .border_thickness(BorderThickness::all(4.0))
                .border_radius(BorderRadius::all(8.0)),
            children: vec![
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Center)
                        .fill_color(Color::new(1.0, 1.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .margin(Margin::all(4.0))
                        .self_alignment(Alignment::Right)
                        .fill_color(Color::new(1.0, 0.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::TopRight)
                        .fill_color(Color::new(1.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Top)
                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::TopLeft)
                        .border_color(Color::new(1.0, 1.0, 1.0, 0.4))
                        .border_thickness(BorderThickness::all(2.0)),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .border_color(Color::new(1.0, 0.0, 0.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .border_color(Color::new(0.0, 1.0, 0.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .border_color(Color::new(0.0, 0.0, 1.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Left)
                        .fill_color(Color::new(0.0, 0.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::BottomLeft)
                        .fill_color(Color::new(0.0, 0.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Bottom)
                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::BottomRight)
                        .fill_color(Color::new(1.0, 0.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(256.0))
                        .height(Extent::FitContent)
                        .padding(Padding::all(16.0))
                        .border_thickness(BorderThickness::all(4.0))
                        .border_color(Color::new(1.0, 1.0, 1.0, 1.0))
                        .self_alignment(Alignment::Center),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(24.0))
                                .fill_color(Color::new(0.0, 1.0, 1.0, 0.5))
                                .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                .border_thickness(BorderThickness::all(1.0))
                                .border_radius(BorderRadius::all(8.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 1.0, 0.5))
                                .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                .border_thickness(BorderThickness::all(1.0))
                                .border_radius(BorderRadius::all(8.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.5))
                                .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                .border_thickness(BorderThickness::all(1.0))
                                .border_radius(BorderRadius::all(8.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(64.0))
                                .margin(Margin::all(8.0))
                                .padding(Padding::all(8.0))
                                .self_alignment(Alignment::Center)
                                .fill_color(Color::new(1.0, 1.0, 0.0, 0.5))
                                .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                .border_thickness(BorderThickness::all(4.0))
                                .border_radius(BorderRadius::all(8.0)),
                            children: vec![UiNode::Box(BoxProps {
                                modifiers: Modifiers::new()
                                    .width(Extent::FillParent)
                                    .height(Extent::FillParent)
                                    .fill_color(Color::new(1.0, 1.0, 1.0, 0.5))
                                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                    .border_thickness(BorderThickness::all(1.0))
                                    .border_radius(BorderRadius::all(8.0)),
                                children: vec![],
                            })],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.5))
                                .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                .border_thickness(BorderThickness::all(1.0))
                                .border_radius(BorderRadius::all(8.0)),
                            children: vec![],
                        }),
                    ],
                }),
            ],
        })],
        ..Default::default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn test_converter(width: f32, height: f32, ui: UiNode, expected: &[Rectangle]) {
        let mut draw_data = vec![];
        ui.to_draw_data(Vec2::ZERO, Vec2::new(width, height), &mut draw_data);
        assert_eq!(expected, &draw_data);
    }

    #[test]
    fn default_box() {
        test_converter(
            32.0,
            32.0,
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new(),
                children: vec![],
            }),
            &[Rectangle {
                position: Vec2::new(0.0, 0.0),
                size: Vec2::new(32.0, 32.0),
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
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new().padding(Padding::all(8.0)),
                children: vec![UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                    children: vec![],
                })],
            }),
            &[
                Rectangle {
                    position: Vec2::new(0.0, 0.0),
                    size: Vec2::new(32.0, 32.0),
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
                Rectangle {
                    position: Vec2::new(8.0, 8.0),
                    size: Vec2::new(16.0, 16.0),
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
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new().margin(Margin::all(8.0)),
                children: vec![],
            }),
            &[Rectangle {
                position: Vec2::new(8.0, 8.0),
                size: Vec2::new(16.0, 16.0),
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
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new().padding(Padding::all(16.0)),
                children: vec![UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                    children: vec![],
                })],
            }),
            &[
                Rectangle {
                    position: Vec2::new(0.0, 0.0),
                    size: Vec2::new(32.0, 32.0),
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
                Rectangle {
                    position: Vec2::new(16.0, 16.0),
                    size: Vec2::new(0.0, 0.0),
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
        //     UiNode::Box(BoxProps {
        //         modifiers: Modifiers::new().padding(Padding::all(24.0)),
        //         children: vec![UiNode::Box(BoxProps {
        //             modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
        //             children: vec![],
        //         })],
        //     }),
        //     &[
        //         Rectangle {
        //             position: Vec2::new(0.0, 0.0),
        //             size: Vec2::new(32.0, 32.0),
        //             fill_color: Color::ZERO,
        //             border_color: Color::ZERO,
        //             border_radius: Vec4::ZERO,
        //             border_width: Vec4::ZERO,
        //         },
        //         Rectangle {
        //             position: Vec2::new(16.0, 16.0),
        //             size: Vec2::new(0.0, 0.0),
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
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(8.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(alignment),
                    children: vec![],
                }),
                &[Rectangle {
                    position: expected_position,
                    size: Vec2::new(8.0, 8.0),
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
        let ui: UiNode = UiNode::Box(BoxProps {
            modifiers: Modifiers::new().width(Extent::Px(8.0)).height(Extent::Px(8.0)),
            children: vec![],
        });

        test_converter(
            15.0,
            15.0,
            ui.clone(),
            &[Rectangle {
                position: Vec2::new(4.0, 4.0),
                size: Vec2::new(8.0, 8.0),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::splat(0.0),
                border_width: Vec4::splat(0.0),
            }],
        );

        test_converter(
            17.0,
            17.0,
            ui,
            &[Rectangle {
                position: Vec2::new(5.0, 5.0),
                size: Vec2::new(8.0, 8.0),
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
                UiNode::Box(BoxProps {
                    // Border thickness of 4.0 makes the parent container equivalent to a 24.0 size container.
                    modifiers: Modifiers::new().border_thickness(BorderThickness::all(4.0)),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .width(Extent::Px(8.0))
                            .height(Extent::Px(8.0))
                            .self_alignment(alignment),
                        children: vec![],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::ZERO,
                        size: Vec2::new(32.0, 32.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::splat(4.0),
                    },
                    Rectangle {
                        position: expected_position,
                        size: Vec2::new(8.0, 8.0),
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

    mod boxes {
        use super::*;

        #[test]
        fn box_fit_content_with_fill_parent_child() {
            test_converter(
                128.0,
                128.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(96.0, 64.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(96.0, 64.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(8.0, 64.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 128.0 - 64.0 - 8.0),
                        size: Vec2::new(96.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn box_fit_content_with_fill_parent_child_all_children_with_borders() {
            /* In this case, children having border should not affect in any way the parent size. */
            test_converter(
                128.0,
                128.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .border_thickness(BorderThickness::all(8.0))
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .border_thickness(BorderThickness::all(8.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .border_thickness(BorderThickness::all(8.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(96.0, 64.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(96.0, 64.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(8.0, 64.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 128.0 - 64.0 - 8.0),
                        size: Vec2::new(96.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                ],
            );
        }

        #[test]
        fn box_fit_content_with_fill_parent_child_parent_with_border() {
            test_converter(
                128.0,
                128.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .border_thickness(BorderThickness::all(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(96.0 + 16.0, 64.0 + 16.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    Rectangle {
                        position: Vec2::new(8.0, 8.0),
                        size: Vec2::new(96.0, 64.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(8.0, 8.0),
                        size: Vec2::new(8.0, 64.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(8.0, 8.0 + 64.0 - 8.0),
                        size: Vec2::new(96.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn box_fit_content_with_fill_parent_child_parent_with_padding() {
            /* Should be functionally almost equivalent to box_fit_content_with_fill_parent_child_parent_with_border */
            test_converter(
                128.0,
                128.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(96.0 + 16.0, 64.0 + 16.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(8.0, 8.0),
                        size: Vec2::new(96.0, 64.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(8.0, 8.0),
                        size: Vec2::new(8.0, 64.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(8.0, 8.0 + 64.0 - 8.0),
                        size: Vec2::new(96.0, 8.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new(),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new().height(Extent::Px(24.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new().height(Extent::Px(48.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(32.0, 128.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 128.0 - 24.0),
                        size: Vec2::new(32.0, 24.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                        size: Vec2::new(32.0, 48.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new(),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new().height(Extent::Px(24.0)).padding(Padding::all(2.0)),
                            children: vec![UiNode::Box(BoxProps {
                                modifiers: Modifiers::new(),
                                children: vec![],
                            })],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new().height(Extent::Px(48.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(32.0, 128.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 128.0 - 24.0),
                        size: Vec2::new(32.0, 24.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0 + 2.0, 128.0 - 24.0 + 2.0),
                        size: Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                        size: Vec2::new(32.0, 48.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new(),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().height(Extent::Px(16.0)).margin(Margin::all(2.0)),
                        children: vec![],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(32.0, 128.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0 + 2.0, 128.0 - 16.0 - 2.0),
                        size: Vec2::new(32.0 - 4.0, 16.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(64.0, 32.0 + 32.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 32.0),
                        size: Vec2::new(64.0, 32.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(64.0, 32.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FillParent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().height(Extent::Px(32.0)),
                        children: vec![],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(256.0, 32.0 + 16.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(8.0, 8.0),
                        size: Vec2::new(256.0 - 16.0, 32.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::FitContent),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().width(Extent::FillParent).height(Extent::FillParent),
                        children: vec![],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(0.0, 0.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(0.0, 0.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::FitContent),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .width(Extent::FillParent)
                            .height(Extent::FillParent)
                            .margin(Margin::all(4.0)),
                        children: vec![],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(8.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(4.0, 4.0),
                        size: Vec2::new(0.0, 0.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::Px(48.0)),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(64.0, 48.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(64.0, 48.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, -48.0),
                        size: Vec2::new(64.0, 48.0),
                        fill_color: Color::new(0.0, 1.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }
    }
}
