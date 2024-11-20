use glam::{Vec2, Vec4, Vec4Swizzles};

use crate::{rectangle::Rectangle, vertex::Color};

#[derive(Debug, Clone, Copy)]
enum Modifier {
    Width(Extent),
    Height(Extent),
    Padding(Vec4),
    FillColor(Color),
    BorderColor(Color),
    BorderThickness(BorderThickness),
    BorderRadius(BorderRadius),
    SelfAlignment(Alignment),
}

pub struct Modifiers(Vec<Modifier>);

impl Modifiers {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn width(self, width: Extent) -> Self {
        self.add(Modifier::Width(width))
    }

    pub fn height(self, height: Extent) -> Self {
        self.add(Modifier::Height(height))
    }

    pub fn padding(self, padding: Vec4) -> Self {
        self.add(Modifier::Padding(padding))
    }

    pub fn fill_color(self, fill_color: Color) -> Self {
        self.add(Modifier::FillColor(fill_color))
    }

    pub fn border_color(self, border_color: Color) -> Self {
        self.add(Modifier::BorderColor(border_color))
    }

    pub fn border_thickness(self, border_thickness: BorderThickness) -> Self {
        self.add(Modifier::BorderThickness(border_thickness))
    }

    pub fn border_radius(self, border_radius: BorderRadius) -> Self {
        self.add(Modifier::BorderRadius(border_radius))
    }

    pub fn self_alignment(self, alignment: Alignment) -> Self {
        self.add(Modifier::SelfAlignment(alignment))
    }

    fn add(mut self, modifier: Modifier) -> Self {
        self.0.push(modifier);
        Self(self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Extent {
    FillParent,
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

pub enum UiNode {
    Box(BoxProps),
}

impl UiNode {
    pub fn to_draw_data(&self, parent_pos: Vec2, parent_size: Vec2, draw_data: &mut Vec<Rectangle>) {
        match self {
            UiNode::Box(props) => Self::process_box(props, parent_pos, parent_size, draw_data),
        }
    }

    fn process_box(
        BoxProps { modifiers, children }: &BoxProps,
        mut parent_pos: Vec2,
        mut parent_size: Vec2,
        draw_data: &mut Vec<Rectangle>,
    ) {
        let mut p = RectangleProps::default();

        for m in &modifiers.0 {
            match *m {
                Modifier::Width(width) => p.width = width,
                Modifier::Height(height) => p.height = height,
                Modifier::FillColor(fill_color) => p.fill_color = fill_color,
                Modifier::BorderColor(border_color) => p.border_color = border_color,
                Modifier::BorderThickness(border_thickness) => p.border_thickness = border_thickness,
                Modifier::BorderRadius(border_radius) => p.border_radius = border_radius,
                Modifier::SelfAlignment(alignment) => p.alignment = alignment,
                Modifier::Padding(padding) => {
                    let (computed_pos, computed_size) = Self::emit_rectangle(&p, parent_pos, parent_size, draw_data);

                    p = Default::default();
                    parent_pos = (computed_pos + padding.xw().round()).min(parent_pos + parent_size * 0.5);
                    parent_size = (computed_size - padding.xw().round() - padding.yz().round()).max(Vec2::ZERO);
                }
            }
        }

        let (computed_pos, computed_size) = Self::emit_rectangle(&p, parent_pos, parent_size, draw_data);

        for c in children {
            c.to_draw_data(computed_pos, computed_size, draw_data);
        }
    }

    fn emit_rectangle(
        p: &RectangleProps,
        parent_pos: Vec2,
        parent_size: Vec2,
        out: &mut Vec<Rectangle>,
    ) -> (Vec2, Vec2) {
        let computed_width = match p.width {
            Extent::FillParent => parent_size.x,
            Extent::Px(px) => px,
        };

        let computed_height = match p.height {
            Extent::FillParent => parent_size.y,
            Extent::Px(px) => px,
        };

        let parent_center = parent_pos + (parent_size * 0.5);
        let computed_size = Vec2::new(computed_width, computed_height).round();

        let computed_pos = match p.alignment {
            Alignment::Center => (parent_center - 0.5 * computed_size).round(),
            Alignment::Right => Vec2::new(
                parent_pos.x + parent_size.x - computed_size.x,
                parent_center.y - 0.5 * computed_size.y,
            )
            .round(),
            Alignment::TopRight => (parent_pos + parent_size - computed_size).round(),
            Alignment::Top => Vec2::new(
                parent_center.x - 0.5 * computed_size.x,
                parent_pos.y + parent_size.y - computed_size.y,
            )
            .round(),
            Alignment::TopLeft => Vec2::new(parent_pos.x, parent_pos.y + parent_size.y - computed_size.y).round(),
            Alignment::Left => Vec2::new(parent_pos.x, parent_center.y - 0.5 * computed_size.y).round(),
            Alignment::BottomLeft => parent_pos.round(),
            Alignment::Bottom => Vec2::new(parent_center.x - 0.5 * computed_size.x, parent_pos.y).round(),
            Alignment::BottomRight => Vec2::new(parent_pos.x + parent_size.x - computed_size.x, parent_pos.y).round(),
        };

        let rectangle = Rectangle {
            position: computed_pos,
            size: computed_size,
            fill_color: p.fill_color,
            border_color: p.border_color,
            border_radius: p.border_radius.to_vec4(),
            border_width: p.border_thickness.to_vec4(),
        };

        out.push(rectangle);

        (computed_pos, computed_size)
    }
}

pub fn example_ui() -> UiNode {
    return UiNode::Box(BoxProps {
        modifiers: Modifiers::new()
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .fill_color(Color::new(0.1, 0.1, 1.0, 1.0))
            .padding(Vec4::splat(16.0))
            .fill_color(Color::new(1.0, 1.0, 0.1, 0.25))
            .border_radius(BorderRadius::all(16.0))
            .padding(Vec4::splat(16.0)),
        children: vec![UiNode::Box(BoxProps {
            modifiers: Modifiers::new()
                .width(Extent::FillParent)
                .height(Extent::FillParent)
                .padding(Vec4::ZERO)
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
                modifiers: Modifiers::new().padding(Vec4::splat(8.0)),
                children: vec![],
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
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
            ],
        )
    }

    #[test]
    fn full_padding() {
        test_converter(
            32.0,
            32.0,
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new().padding(Vec4::splat(16.0)),
                children: vec![],
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
                    fill_color: Color::ZERO,
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
                modifiers: Modifiers::new().padding(Vec4::splat(24.0)),
                children: vec![],
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
                    fill_color: Color::ZERO,
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



}
