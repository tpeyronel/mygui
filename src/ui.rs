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
        Vec4::new(
            self.bottom_left,
            self.bottom_right,
            self.top_right,
            self.top_left,
        )
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

pub enum UiNode {
    Box(BoxProps),
}

impl UiNode {
    pub fn to_draw_data(
        &self,
        mut parent_pos: Vec2,
        mut parent_size: Vec2,
        draw_data: &mut Vec<Rectangle>,
    ) {
        match self {
            UiNode::Box(BoxProps {
                modifiers,
                children,
            }) => {
                let mut width = Extent::default();
                let mut height = Extent::default();
                let mut fill_color = Color::default();
                let mut border_color = Color::default();
                let mut border_thickness = BorderThickness::default();
                let mut border_radius = BorderRadius::default();

                for m in &modifiers.0 {
                    match *m {
                        Modifier::Width(w) => width = w,
                        Modifier::Height(h) => height = h,
                        Modifier::Padding(padding) => {
                            let computed_width = match width {
                                Extent::FillParent => parent_size.x,
                                Extent::Px(px) => px,
                            };

                            let computed_height = match height {
                                Extent::FillParent => parent_size.y,
                                Extent::Px(px) => px,
                            };

                            let computed_size = Vec2::new(computed_width, computed_height).round();

                            let parent_center = parent_pos + (parent_size * 0.5);
                            let computed_pos = (parent_center - (computed_size * 0.5)).round();

                            let rectangle = Rectangle {
                                position: computed_pos,
                                size: computed_size,
                                fill_color,
                                border_color,
                                border_radius: border_radius.to_vec4(),
                                border_width: border_thickness.to_vec4(),
                            };

                            draw_data.push(rectangle);

                            width = Default::default();
                            height = Default::default();
                            fill_color = Default::default();
                            border_color = Default::default();
                            border_thickness = Default::default();
                            border_radius = Default::default();
                            parent_pos = computed_pos + padding.xw().round();
                            parent_size =
                                computed_size - padding.xw().round() - padding.yz().round();
                        }
                        Modifier::FillColor(c) => fill_color = c,
                        Modifier::BorderColor(c) => border_color = c,
                        Modifier::BorderThickness(t) => border_thickness = t,
                        Modifier::BorderRadius(r) => border_radius = r,
                    }
                }

                let computed_width = match width {
                    Extent::FillParent => parent_size.x,
                    Extent::Px(px) => px,
                };

                let computed_height = match height {
                    Extent::FillParent => parent_size.y,
                    Extent::Px(px) => px,
                };

                let computed_size = Vec2::new(computed_width, computed_height).round();

                let parent_center = parent_pos + (parent_size * 0.5);
                let computed_pos = (parent_center - (computed_size * 0.5)).round();

                let rectangle = Rectangle {
                    position: computed_pos,
                    size: computed_size,
                    fill_color,
                    border_color,
                    border_radius: border_radius.to_vec4(),
                    border_width: border_thickness.to_vec4(),
                };

                draw_data.push(rectangle);
                for c in children {
                    c.to_draw_data(computed_pos, computed_size, draw_data);
                }
            }
        }
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
            children: vec![UiNode::Box(BoxProps {
                modifiers: Modifiers::new()
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .padding(Vec4::ZERO)
                    .fill_color(Color::new(0.1, 1.0, 0.1, 0.25))
                    .border_color(Color::new(0.1, 1.0, 0.1, 0.9))
                    .border_thickness(BorderThickness::all(1.0))
                    .border_radius(BorderRadius::all(4.0)),
                children: vec![],
            })],
        })],
        ..Default::default()
    });
}
