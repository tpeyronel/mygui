use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

impl MouseButton {
    pub fn from_winit(button: winit::event::MouseButton) -> Option<Self> {
        match button {
            winit::event::MouseButton::Left => Some(MouseButton::Left),
            winit::event::MouseButton::Right => Some(MouseButton::Right),
            winit::event::MouseButton::Middle => Some(MouseButton::Middle),
            winit::event::MouseButton::Back => Some(MouseButton::Back),
            winit::event::MouseButton::Forward => Some(MouseButton::Forward),
            winit::event::MouseButton::Other(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementState {
    Pressed,
    Released,
}

impl ElementState {
    pub fn from_winit(element_state: winit::event::ElementState) -> Self {
        match element_state {
            winit::event::ElementState::Pressed => ElementState::Pressed,
            winit::event::ElementState::Released => ElementState::Released,
        }
    }
}

pub enum InputEvent {
    CursorMoved { position: Vec2 },
    MouseInput { button: MouseButton, state: ElementState },
}
