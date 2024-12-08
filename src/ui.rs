mod border_radius;
mod border_thickness;
mod margin;
mod padding;

use border_radius::BorderRadius;
use border_thickness::BorderThickness;
use glam::{Vec2, Vec4};
use margin::Margin;
use padding::Padding;

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
    weight: f32,
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

    pub fn weight(self, weight: f32) -> Self {
        Self { weight, ..self }
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

pub type RowProps = ColumnProps;

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
    Row(RowProps),
}

impl UiNode {
    pub fn to_draw_data(self, boundary_pos: Vec2, boundary_size: Vec2, out: &mut Vec<Rectangle>) {
        let boundary_pos = boundary_pos.round();
        let boundary_size = boundary_size.round();

        let root_node = UiNode::Box(BoxProps {
            modifiers: Modifiers::new()
                .width(Extent::Px(boundary_size.x))
                .height(Extent::Px(boundary_size.y)),
            children: vec![self],
        });

        let root_layout = Layout {
            margin_position: boundary_pos,
            margin_size: boundary_size,
            children_boundary_size: boundary_size,
            margin: Margin::all(0.0),
            border_thickness: BorderThickness::all(0.0),
            padding: Padding::all(0.0),
        };

        let root_layout_node = root_node.compute_layout_rec(root_layout);
        root_node.to_draw_data_rec(&root_layout_node, out);
        out.remove(0); // TODO: remove. This is mainly done to simplify testing (the first rectangle is completely transparent).
    }

    fn to_draw_data_rec(&self, layout_node: &UiNodeLayout, draw_data: &mut Vec<Rectangle>) {
        let (modifiers, children) = match self {
            UiNode::Box(props) => (&props.modifiers, &props.children),
            UiNode::Column(props) => (&props.modifiers, &props.children),
            UiNode::Row(props) => (&props.modifiers, &props.children),
        };

        let layout = &layout_node.layout;

        Self::emit_rectangle(
            layout.border_position(),
            layout.border_size(),
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
        match self {
            UiNode::Box(props) => Self::compute_box_children_layouts(props, layout),
            UiNode::Column(props) => Self::compute_column_children_layouts(props, layout),
            UiNode::Row(props) => Self::compute_row_children_layouts(props, layout),
        }
    }

    fn compute_box_children_layouts(BoxProps { children, .. }: &BoxProps, layout: &Layout) -> Vec<UiNodeLayout> {
        children
            .iter()
            .map(|c| {
                let child_measurements = c.measure(layout.children_boundary_size());
                let child_margin_size = child_measurements.margin_size;
                let child_modifiers = match c {
                    UiNode::Box(props) => &props.modifiers,
                    UiNode::Column(props) => &props.modifiers,
                    UiNode::Row(props) => &props.modifiers,
                };
                let child_margin_position = match child_modifiers.self_alignment {
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

                let child_layout = Layout {
                    margin_position: child_margin_position,
                    margin_size: child_measurements.margin_size,
                    children_boundary_size: child_measurements.children_boundary_size,
                    margin: child_modifiers.margin,
                    border_thickness: child_modifiers.border_thickness,
                    padding: child_modifiers.padding,
                };

                c.compute_layout_rec(child_layout)
            })
            .collect()
    }

    fn compute_column_children_layouts(
        ColumnProps { children, .. }: &ColumnProps,
        layout: &Layout,
    ) -> Vec<UiNodeLayout> {
        let total_children_weight: f32 = children
            .iter()
            .map(|c| match c {
                UiNode::Box(props) => props.modifiers.weight,
                UiNode::Column(props) => props.modifiers.weight,
                UiNode::Row(props) => props.modifiers.weight,
            })
            .sum();

        let total_children_height: f32 = children
            .iter()
            .map(|c| c.measure(layout.children_boundary_size()).margin_size.y)
            .sum();

        let extra_column_height = (layout.content_size().y - total_children_height).max(0.0);

        let mut remaining_column_height = extra_column_height;
        let mut remaining_children_weight = total_children_weight;

        let mut vertical_offset = 0.0;
        let column_top = layout.content_position().y + layout.content_size().y;
        children
            .iter()
            .map(|c| {
                let child_modifiers = match c {
                    UiNode::Box(props) => &props.modifiers,
                    UiNode::Column(props) => &props.modifiers,
                    UiNode::Row(props) => &props.modifiers,
                };

                let mut child_measurements = c.measure(layout.children_boundary_size());

                if remaining_children_weight > 0.0 {
                    let child_weight = child_modifiers.weight;
                    let child_extra_height = remaining_column_height * (child_weight / remaining_children_weight);
                    let child_extra_height = child_extra_height.ceil().min(remaining_column_height);
                    remaining_column_height -= child_extra_height;
                    remaining_children_weight -= child_weight;

                    child_measurements.margin_size.y += child_extra_height;
                    child_measurements.children_boundary_size = child_measurements.content_size();
                }

                let child_margin_size = child_measurements.margin_size;
                vertical_offset += child_margin_size.y;

                let child_margin_position = match child_modifiers.self_alignment {
                    Alignment::TopLeft | Alignment::Left | Alignment::BottomLeft => {
                        Vec2::new(layout.content_position().x, column_top - vertical_offset)
                    }
                    Alignment::Top | Alignment::Center | Alignment::Bottom => Vec2::new(
                        layout.content_center().x - 0.5 * child_margin_size.x,
                        column_top - vertical_offset,
                    ),
                    Alignment::TopRight | Alignment::Right | Alignment::BottomRight => Vec2::new(
                        layout.content_position().x + layout.content_size().x - child_margin_size.x,
                        column_top - vertical_offset,
                    ),
                }
                .round();

                let child_layout = Layout {
                    margin_position: child_margin_position,
                    margin_size: child_measurements.margin_size,
                    children_boundary_size: child_measurements.children_boundary_size,
                    margin: child_modifiers.margin,
                    border_thickness: child_modifiers.border_thickness,
                    padding: child_modifiers.padding,
                };

                c.compute_layout_rec(child_layout)
            })
            .collect()
    }

    fn compute_row_children_layouts(RowProps { children, .. }: &RowProps, layout: &Layout) -> Vec<UiNodeLayout> {
        let mut horizontal_offset = 0.0;
        children
            .iter()
            .map(|c| {
                let child_measurements = c.measure(layout.children_boundary_size());
                let child_margin_size = child_measurements.margin_size;

                let child_modifiers = match c {
                    UiNode::Box(props) => &props.modifiers,
                    UiNode::Column(props) => &props.modifiers,
                    UiNode::Row(props) => &props.modifiers,
                };

                let child_margin_position = match child_modifiers.self_alignment {
                    Alignment::BottomLeft | Alignment::Bottom | Alignment::BottomRight => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        layout.content_position().y,
                    ),
                    Alignment::Left | Alignment::Center | Alignment::Right => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        layout.content_center().y - 0.5 * child_margin_size.y,
                    ),
                    Alignment::TopLeft | Alignment::Top | Alignment::TopRight => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        layout.content_position().y + layout.content_size().y - child_margin_size.y,
                    ),
                };

                horizontal_offset += child_margin_size.x;

                let child_layout = Layout {
                    margin_position: child_margin_position,
                    margin_size: child_measurements.margin_size,
                    children_boundary_size: child_measurements.children_boundary_size,
                    margin: child_modifiers.margin,
                    border_thickness: child_modifiers.border_thickness,
                    padding: child_modifiers.padding,
                };

                c.compute_layout_rec(child_layout)
            })
            .collect()
    }

    fn measure(&self, boundary_size: Vec2) -> Measurements {
        let modifiers = match self {
            UiNode::Box(props) => &props.modifiers,
            UiNode::Column(props) => &props.modifiers,
            UiNode::Row(props) => &props.modifiers,
        };

        let children = match self {
            UiNode::Box(props) => &props.children,
            UiNode::Column(props) => &props.children,
            UiNode::Row(props) => &props.children,
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
        let preliminar_children_boundary_size = (preliminar_children_boundary_size
            - modifiers.border_thickness.delta_size()
            - modifiers.padding.delta_size())
        .max(Vec2::ZERO);
        // TODO: only compute when necessary
        let min_intrinsic_children_sizes: Vec<Measurements> =
            Self::measure_children(preliminar_children_boundary_size, children);

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
                UiNode::Row(_) => {
                    let min_intrinsic_width = min_intrinsic_children_sizes.iter().map(|cs| cs.margin_size.x).sum();
                    children_boundary_size.x = min_intrinsic_width;
                    let children_sizes = Self::measure_children(children_boundary_size, children);
                    children_sizes.iter().map(|cs| cs.margin_size.x).sum::<f32>()
                        + modifiers.padding.delta_size().x
                        + modifiers.border_thickness.delta_size().x
                }
            },
            Extent::Px(px) => px.round(),
        };

        let computed_height = match modifiers.height {
            Extent::FillParent => boundary_size.y,
            Extent::FitContent => match self {
                UiNode::Box(_) | UiNode::Row(_) => {
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

        Measurements {
            margin_size,
            margin: modifiers.margin,
            border_thickness: modifiers.border_thickness,
            padding: modifiers.padding,
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
    margin_position: Vec2,
    margin_size: Vec2,
    children_boundary_size: Vec2,
    margin: Margin,
    border_thickness: BorderThickness,
    padding: Padding,
}

impl Layout {
    fn margin_position(&self) -> Vec2 {
        self.margin_position
    }

    fn border_position(&self) -> Vec2 {
        self.margin_position + self.margin.delta_position()
    }

    fn padding_position(&self) -> Vec2 {
        self.margin_position + self.margin.delta_position() + self.border_thickness.delta_position()
    }

    fn content_position(&self) -> Vec2 {
        self.margin_position
            + self.margin.delta_position()
            + self.border_thickness.delta_position()
            + self.padding.delta_position() // TODO: clamp
    }

    fn margin_size(&self) -> Vec2 {
        self.margin_size
    }

    fn border_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size()).max(Vec2::ZERO)
    }

    fn padding_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size()).max(Vec2::ZERO)
    }

    fn content_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size() - self.padding.delta_size())
            .max(Vec2::ZERO)
    }

    fn children_boundary_size(&self) -> Vec2 {
        self.children_boundary_size
    }

    fn content_center(&self) -> Vec2 {
        self.content_position() + (self.content_size() * 0.5)
    }
}

#[derive(Debug)]
struct Measurements {
    margin_size: Vec2,
    margin: Margin,
    border_thickness: BorderThickness,
    padding: Padding,
    children_boundary_size: Vec2,
}

impl Measurements {
    fn border_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size()).max(Vec2::ZERO)
    }

    fn padding_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size()).max(Vec2::ZERO)
    }

    fn content_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size() - self.padding.delta_size())
            .max(Vec2::ZERO)
    }
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
                        .border_radius(BorderRadius::new(0.0, 8.0, 16.0, 24.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Top)
                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::new(4.0, 8.0, 12.0, 16.0)),
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
                                .padding(Padding::all(8.0))
                                .border_color(Color::new(1.0, 0.0, 0.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0)),
                            children: vec![UiNode::Box(BoxProps {
                                modifiers: Modifiers::new().fill_color(Color::new(0.0, 1.0, 0.0, 0.4)),
                                children: vec![],
                            })],
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
                UiNode::Row(RowProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .border_thickness(BorderThickness::all(4.0))
                        .border_color(Color::new(1.0, 1.0, 1.0, 1.0)),
                    children: vec![
                        UiNode::Column(ColumnProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(256.0))
                                .height(Extent::FitContent)
                                .padding(Padding::all(16.0))
                                .border_thickness(BorderThickness::all(4.0))
                                .border_color(Color::new(1.0, 1.0, 1.0, 1.0)),
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
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.25, 0.25, 1.0, 0.5)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.25, 0.25, 1.0, 0.25)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.25, 1.0, 0.25, 0.5)),
                            children: vec![],
                        }),
                    ],
                }),
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(256.0))
                        .self_alignment(Alignment::Left),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.4)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(2.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.4)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 0.0, 1.0, 0.4)),
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
        fn box_different_padding_values() {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().padding(Padding::new(1.0, 2.0, 4.0, 8.0)),
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
                        position: Vec2::new(8.0, 1.0),
                        size: Vec2::new(32.0 - 10.0, 32.0 - 5.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn box_different_border_thickness_values() {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().border_thickness(BorderThickness::new(1.0, 2.0, 4.0, 8.0)),
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
                        border_width: Vec4::new(1.0, 2.0, 4.0, 8.0),
                    },
                    Rectangle {
                        position: Vec2::new(1.0, 2.0),
                        size: Vec2::new(32.0 - 5.0, 32.0 - 10.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

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

    mod rows {
        use super::*;

        #[test]
        fn row_both_fit_content_with_fill_parent_child() {
            test_converter(
                256.0,
                256.0,
                UiNode::Row(RowProps {
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
                        size: Vec2::new(64.0 + 64.0, 32.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(64.0, 32.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    Rectangle {
                        position: Vec2::new(64.0, 0.0),
                        size: Vec2::new(64.0, 32.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(100.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(20.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(40.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 100.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 100.0 - 40.0),
                        size: Vec2::new(50.0, 20.0 + 20.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 40.0 + 20.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(100.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 0.0, 1.0, 1.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 100.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 100.0 - 34.0),
                        size: Vec2::new(50.0, 34.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 100.0 - 34.0 - 33.0),
                        size: Vec2::new(50.0, 33.0),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 33.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .margin(Margin::all(4.0)) // This margin should only allow for a height of 0.
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(4.0, 4.0),
                        size: Vec2::new(50.0 - 8.0, 0.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .border_thickness(BorderThickness::all(3.0)) // This border thickness should only allow for a height of the child of 2.0.
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        })],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::splat(3.0),
                    },
                    Rectangle {
                        position: Vec2::new(3.0, 3.0),
                        size: Vec2::new(50.0 - 6.0, 2.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .padding(Padding::all(3.0)) // This padding should only allow for a height of the child of 2.0.
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        })],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(3.0, 3.0),
                        size: Vec2::new(50.0 - 6.0, 2.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![UiNode::Box(BoxProps {
                                modifiers: Modifiers::new()
                                    .width(Extent::FillParent)
                                    .height(Extent::FillParent)
                                    .fill_color(Color::new(0.0, 0.0, 1.0, 1.0)),
                                children: vec![],
                            })],
                        })],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(50.0, 8.0),
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
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().border_thickness(BorderThickness::all(3.5)), // Should all be rounded to 4.0
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
                        border_width: Vec4::splat(4.0),
                    },
                    Rectangle {
                        position: Vec2::new(4.0, 4.0),
                        size: Vec2::new(32.0 - 8.0, 32.0 - 8.0),
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
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().padding(Padding::all(3.5)), // Should all be rounded to 4.0
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
                        position: Vec2::new(4.0, 4.0),
                        size: Vec2::new(32.0 - 8.0, 32.0 - 8.0),
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
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().margin(Margin::all(3.5)), // Should all be rounded to 4.0
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(4.0, 4.0),
                        size: Vec2::new(32.0 - 8.0, 32.0 - 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(4.0, 4.0),
                        size: Vec2::new(32.0 - 8.0, 32.0 - 8.0),
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

            let ui = UiNode::Box(BoxProps {
                modifiers: Modifiers::new(),
                children: vec![],
            });

            let mut draw_data = vec![];
            ui.to_draw_data(root_position, root_size, &mut draw_data);

            let expected = vec![Rectangle {
                position: Vec2::new(1.0, 1.0),
                size: Vec2::new(32.0, 32.0),
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
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(9.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .width(Extent::Px(8.0))
                            .self_alignment(Alignment::Center)
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    Rectangle {
                        position: Vec2::new(0.0, 0.0),
                        size: Vec2::new(9.0, 8.0),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    Rectangle {
                        position: Vec2::new(1.0, 0.0),
                        size: Vec2::new(8.0, 8.0),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }
    }
}
