use glam::{Vec2, Vec4, Vec4Swizzles};

use crate::{rectangle::Rectangle, vertex::Color};

pub enum Extent {
    FillParent,
    Px(f32),
}

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

impl From<&BorderThickness> for Vec4 {
    fn from(value: &BorderThickness) -> Self {
        Self::new(value.bottom, value.right, value.top, value.left)
    }
}

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

impl From<&BorderRadius> for Vec4 {
    fn from(value: &BorderRadius) -> Self {
        Self::new(
            value.bottom_left,
            value.bottom_right,
            value.top_right,
            value.top_left,
        )
    }
}

pub struct BoxProps {
    width: Extent,
    height: Extent,
    padding: Vec4,
    fill_color: Color,
    border_color: Color,
    border_thickness: BorderThickness,
    border_radius: BorderRadius,
    children: Vec<UiNode>,
}

impl Default for BoxProps {
    fn default() -> Self {
        Self {
            width: Extent::FillParent,
            height: Extent::FillParent,
            padding: Vec4::ZERO,
            fill_color: Color::ZERO,
            border_color: Color::ZERO,
            border_thickness: Default::default(),
            border_radius: Default::default(),
            children: Vec::new(),
        }
    }
}

pub enum UiNode {
    Box(BoxProps),
}

impl UiNode {
    pub fn to_draw_data(
        &self,
        parent_pos: Vec2,
        parent_size: Vec2,
        draw_data: &mut Vec<Rectangle>,
    ) {
        match self {
            UiNode::Box(BoxProps {
                width,
                height,
                padding,
                fill_color,
                border_color,
                border_thickness,
                border_radius,
                children,
            }) => {
                let computed_width = match width {
                    Extent::FillParent => parent_size.x,
                    Extent::Px(px) => *px,
                };

                let computed_height = match height {
                    Extent::FillParent => parent_size.y,
                    Extent::Px(px) => *px,
                };

                let computed_size = Vec2::new(computed_width, computed_height).round();

                let parent_center = parent_pos + (parent_size * 0.5);
                let computed_pos = (parent_center - (computed_size * 0.5)).round();

                let rectangle = Rectangle {
                    position: computed_pos,
                    size: computed_size,
                    fill_color: *fill_color,
                    border_color: *border_color,
                    border_radius: border_radius.into(),
                    border_width: border_thickness.into(),
                };

                draw_data.push(rectangle);
                for c in children {
                    c.to_draw_data(
                        computed_pos + padding.xw().round(),
                        computed_size - padding.xw().round() - padding.yz().round(),
                        draw_data,
                    );
                }
            }
        }
    }
}

pub fn example_ui() -> UiNode {
    return UiNode::Box(BoxProps {
        width: Extent::FillParent,
        height: Extent::FillParent,
        padding: Vec4::splat(16.0),
        children: vec![UiNode::Box(BoxProps {
            width: Extent::FillParent,
            height: Extent::FillParent,
            padding: Vec4::ZERO,
            fill_color: Color::new(1.0, 0.1, 0.1, 0.25),
            border_color: Color::new(1.0, 0.1, 0.1, 0.9),
            border_thickness: BorderThickness::all(4.0),
            border_radius: BorderRadius::all(8.0),
            children: vec![UiNode::Box(BoxProps {
                width: Extent::Px(80.0),
                height: Extent::Px(80.0),
                padding: Vec4::ZERO,
                fill_color: Color::new(0.1, 1.0, 0.1, 0.25),
                border_color: Color::new(0.1, 1.0, 0.1, 0.9),
                border_thickness: BorderThickness::all(1.0),
                border_radius: BorderRadius::all(4.0),
                children: vec![],
            })],
        })],
        ..Default::default()
    });
}
