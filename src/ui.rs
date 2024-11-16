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

pub enum UiNode {
    Box {
        width: Extent,
        height: Extent,
        padding: Vec4,
        fill_color: Color,
        border_color: Color,
        border_thickness: BorderThickness,
        border_radius: BorderRadius,
        children: Vec<UiNode>,
    },
}

impl UiNode {
    pub fn to_draw_data(
        &self,
        parent_pos: Vec2,
        parent_size: Vec2,
        draw_data: &mut Vec<Rectangle>,
    ) {
        match self {
            UiNode::Box {
                width,
                height,
                padding,
                fill_color,
                border_color,
                border_thickness,
                border_radius,
                children,
            } => {
                let computed_width = match width {
                    Extent::FillParent => parent_size.x,
                    Extent::Px(px) => *px,
                };

                let computed_height = match height {
                    Extent::FillParent => parent_size.y,
                    Extent::Px(px) => *px,
                };

                let computed_size = Vec2::new(computed_width, computed_height);

                let parent_center = parent_pos + (parent_size * 0.5);
                let computed_pos = parent_center - (computed_size * 0.5);

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
                        computed_pos + padding.xw(),
                        computed_size - padding.xw() - padding.yz(),
                        draw_data,
                    );
                }
            }
        }
    }
}

pub fn example_ui() -> UiNode {
    return UiNode::Box {
        width: Extent::FillParent,
        height: Extent::FillParent,
        padding: Vec4::splat(16.0),
        fill_color: Color::ZERO,
        border_color: Color::ZERO,
        border_thickness: BorderThickness::all(0.0),
        border_radius: BorderRadius::all(0.0),
        children: vec![UiNode::Box {
            width: Extent::FillParent,
            height: Extent::FillParent,
            padding: Vec4::ZERO,
            fill_color: Color::new(1.0, 0.1, 0.1, 0.25),
            border_color: Color::new(1.0, 0.1, 0.1, 0.9),
            border_thickness: BorderThickness::all(4.0),
            border_radius: BorderRadius::all(8.0),
            children: vec![UiNode::Box {
                width: Extent::Px(80.0),
                height: Extent::Px(80.0),
                padding: Vec4::ZERO,
                fill_color: Color::new(0.1, 1.0, 0.1, 0.25),
                border_color: Color::new(0.1, 1.0, 0.1, 0.9),
                border_thickness: BorderThickness::all(1.0),
                border_radius: BorderRadius::all(4.0),
                children: vec![],
            }],
        }],
    };
}
