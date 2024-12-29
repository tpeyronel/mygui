use glam::Vec2;
use winit::event::ElementState;

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
pub enum InputState {
    Pressed,
    Released,
}

impl InputState {
    pub fn from_winit(element_state: ElementState) -> Self {
        match element_state {
            ElementState::Pressed => InputState::Pressed,
            ElementState::Released => InputState::Released,
        }
    }
}

pub enum InputEvent {
    CursorMoved { position: Vec2 },
    MouseInput { button: MouseButton, state: InputState },
}
