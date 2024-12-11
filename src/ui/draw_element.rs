use glam::Vec4;

use crate::{image::image_manager::ImageId, rectangle::Rectangle, vertex::Color};

pub enum DrawElement {
    Rectangle {
        bounds: Rectangle,
        fill_color: Color,
        border_color: Color,
        border_radius: Vec4,
        border_width: Vec4,
    },
    Texture {
        bounds: Rectangle,
        uv_rectangle: Rectangle,
        image_id: ImageId,
    },
}
