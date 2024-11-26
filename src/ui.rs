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

#[derive(Debug, Clone, Copy)]
pub enum Extent {
    FillParent,
    Px(f32),
}

impl Extent {
    fn resolve(&self, parent_extent: f32) -> f32 {
        match self {
            Extent::FillParent => parent_extent,
            Extent::Px(px) => px.round(),
        }
    }
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

#[derive(Debug, Default)]
pub struct RectangleProps {
    pub width: Extent,
    pub height: Extent,
    pub fill_color: Color,
    pub border_color: Color,
    pub border_radius: BorderRadius,
    pub border_thickness: BorderThickness,
    pub alignment: Alignment,
}

#[derive(Debug, Clone)]
pub enum UiNode {
    Box(BoxProps),
    Column(ColumnProps),
}

impl UiNode {
    pub fn to_draw_data(&self, parent_pos: Vec2, parent_size: Vec2, draw_data: &mut Vec<Rectangle>) {
        assert_eq!(parent_pos, parent_pos.round());
        assert_eq!(parent_size, parent_size.round());

        match self {
            UiNode::Box(props) => Self::process_box(props, parent_pos, parent_size, draw_data),
            UiNode::Column(props) => Self::process_column(props, parent_pos, parent_size, draw_data),
        }
    }

    fn process_box(
        BoxProps { modifiers, children }: &BoxProps,
        parent_pos: Vec2,
        parent_size: Vec2,
        draw_data: &mut Vec<Rectangle>,
    ) {
        let margin = modifiers.margin.to_vec4().round();
        let parent_pos: Vec2 = parent_pos + margin.xw();
        let parent_size = parent_size - margin.xw() - margin.yz();

        let (computed_pos, computed_size) = Self::emit_rectangle(modifiers, parent_pos, parent_size, draw_data);

        let padding = modifiers.padding.to_vec4().round();

        let mut inner_pos = computed_pos;
        inner_pos += Vec2::new(modifiers.border_thickness.left, modifiers.border_thickness.bottom).round();
        inner_pos += padding.xw();
        // Don't allow inner_pos to go past the parent's center.
        inner_pos = inner_pos.min((parent_pos + parent_size * 0.5).round());

        let inner_size = (Vec2::new(
            computed_size.x - modifiers.border_thickness.left.round() - modifiers.border_thickness.right.round(),
            computed_size.y - modifiers.border_thickness.bottom.round() - modifiers.border_thickness.right.round(),
        ) - padding.xw()
            - padding.yz())
        .max(Vec2::ZERO);

        for c in children {
            c.to_draw_data(inner_pos, inner_size, draw_data);
        }
    }

    fn process_column(
        ColumnProps { modifiers, children }: &ColumnProps,
        parent_pos: Vec2,
        parent_size: Vec2,
        draw_data: &mut Vec<Rectangle>,
    ) {
        let (computed_pos, computed_size) = Self::emit_rectangle(modifiers, parent_pos, parent_size, draw_data);

        let mut vertical_offset = 0.0;

        for c in children {
            let child_size = c.measure(computed_size);
            vertical_offset += child_size.y;
            let child_pos = Vec2::new(computed_pos.x, computed_pos.y + computed_size.y - vertical_offset);
            c.to_draw_data(child_pos, child_size, draw_data);
        }
    }

    fn emit_rectangle(
        modifiers: &Modifiers,
        boundary_pos: Vec2,
        boundary_size: Vec2,
        out: &mut Vec<Rectangle>,
    ) -> (Vec2, Vec2) {
        let computed_width = match modifiers.width {
            Extent::FillParent => boundary_size.x,
            Extent::Px(px) => px,
        };

        let computed_height = match modifiers.height {
            Extent::FillParent => boundary_size.y,
            Extent::Px(px) => px,
        };

        let boundary_center = boundary_pos + (boundary_size * 0.5);
        let computed_size = Vec2::new(computed_width, computed_height).round();

        let computed_pos = match modifiers.self_alignment {
            Alignment::Center => (boundary_center - 0.5 * computed_size).round(),
            Alignment::Right => Vec2::new(
                boundary_pos.x + boundary_size.x - computed_size.x,
                boundary_center.y - 0.5 * computed_size.y,
            )
            .round(),
            Alignment::TopRight => (boundary_pos + boundary_size - computed_size).round(),
            Alignment::Top => Vec2::new(
                boundary_center.x - 0.5 * computed_size.x,
                boundary_pos.y + boundary_size.y - computed_size.y,
            )
            .round(),
            Alignment::TopLeft => Vec2::new(boundary_pos.x, boundary_pos.y + boundary_size.y - computed_size.y).round(),
            Alignment::Left => Vec2::new(boundary_pos.x, boundary_center.y - 0.5 * computed_size.y).round(),
            Alignment::BottomLeft => boundary_pos.round(),
            Alignment::Bottom => Vec2::new(boundary_center.x - 0.5 * computed_size.x, boundary_pos.y).round(),
            Alignment::BottomRight => {
                Vec2::new(boundary_pos.x + boundary_size.x - computed_size.x, boundary_pos.y).round()
            }
        };

        let rectangle = Rectangle {
            position: computed_pos,
            size: computed_size,
            fill_color: modifiers.fill_color,
            border_color: modifiers.border_color,
            border_radius: modifiers.border_radius.to_vec4(),
            border_width: modifiers.border_thickness.to_vec4(),
        };

        out.push(rectangle);

        (computed_pos, computed_size)
    }

    fn measure(&self, boundary_size: Vec2) -> Vec2 {
        let modifiers = match self {
            UiNode::Box(props) => &props.modifiers,
            UiNode::Column(props) => &props.modifiers,
        };

        let margin = modifiers.margin.to_vec4().round();
        let boundary_size = boundary_size - margin.xw() - margin.yz();

        return Vec2::new(
            modifiers.width.resolve(boundary_size.x),
            modifiers.height.resolve(boundary_size.y),
        ) + margin.xw()
            + margin.yz();
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
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::TopLeft)
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
                        .width(Extent::Px(128.0))
                        .height(Extent::Px(64.0))
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
        test_converter(
            32.0,
            32.0,
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new().padding(Padding::all(24.0)),
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
}
