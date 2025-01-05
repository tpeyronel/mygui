use std::{any::Any, collections::HashMap, rc::Rc};

use glam::Vec2;

use crate::{
    font::font_engine::FontEngine,
    input::{ElementState, InputEvent, MouseButton},
    rectangle::Rectangle,
    ui::{draw_element::DrawElement, to_draw_data},
};

use super::{set_state::set_state_channel, ui::Ui};

pub struct UiContext {
    input_state: InputState,
    states: HashMap<u64, Vec<u8>>,
    refs: HashMap<u64, Rc<dyn Any>>,
    bounding_boxes: Vec<(u64, Rectangle)>,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            input_state: InputState::new(),
            states: HashMap::new(),
            refs: HashMap::new(),
            bounding_boxes: Vec::new(),
        }
    }

    pub fn process_input_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::CursorMoved { position } => {
                self.on_cursor_moved(Some(position));
            }
            InputEvent::MouseInput { button, state } => match button {
                MouseButton::Left => self.on_lmb_state_changed(state),
                MouseButton::Right => {}
                MouseButton::Middle => {}
                MouseButton::Back => {}
                MouseButton::Forward => {}
            },
        }
    }

    pub fn build_ui(
        &mut self,
        window_size: Vec2,
        font_engine: &mut Box<dyn FontEngine>,
        f: impl FnOnce(&mut Ui),
    ) -> Vec<DrawElement> {
        let (set_state_tx, set_state_rx) = set_state_channel();

        let mut root_ui = Ui::new(&self.input_state, &self.states, &set_state_tx, &mut self.refs);
        f(&mut root_ui);
        let (children, children_path_hash_nodes) = root_ui.finish();

        let mut draw_data = vec![];
        self.bounding_boxes.clear();
        to_draw_data(
            children,
            children_path_hash_nodes,
            Vec2::ZERO,
            window_size,
            font_engine,
            &mut draw_data,
            &mut self.bounding_boxes,
        );

        set_state_rx.drain(|hash, bytes| {
            self.states.insert(hash, bytes);
        });

        self.input_state.events.clear();

        draw_data
    }

    fn on_cursor_moved(&mut self, cursor_position: Option<Vec2>) {
        self.input_state.cursor_position = cursor_position;

        let Some(cursor_position) = cursor_position else {
            self.input_state.hovered_node_hash = None;
            return;
        };

        self.input_state.hovered_node_hash = self.find_node_hash_by_cursor_position(cursor_position);
    }

    fn on_lmb_state_changed(&mut self, state: ElementState) {
        let Some(cursor_position) = self.input_state.cursor_position else {
            return;
        };

        match state {
            ElementState::Pressed => {
                // TODO: maybe use input_state.hovered_node_hash?
                let Some(pressed_node_hash) = self.find_node_hash_by_cursor_position(cursor_position) else {
                    return;
                };

                self.input_state
                    .events
                    .entry(pressed_node_hash)
                    .or_insert_with(|| Vec::new())
                    .push(NodeInputEvent::MouseEvent {
                        button: MouseButton::Left,
                        state: ElementState::Pressed,
                    });
                self.input_state.pressed_node_hash = Some(pressed_node_hash);
            }
            ElementState::Released => {
                let Some(released_node_hash) = self.find_node_hash_by_cursor_position(cursor_position) else {
                    return;
                };

                // Only emit Released event if element is both pressed and hovered.
                if self.input_state.pressed_node_hash == Some(released_node_hash)
                    && self.input_state.hovered_node_hash == Some(released_node_hash)
                {
                    self.input_state
                        .events
                        .entry(released_node_hash)
                        .or_insert_with(|| Vec::new())
                        .push(NodeInputEvent::MouseEvent {
                            button: MouseButton::Left,
                            state: ElementState::Released,
                        });
                }
                self.input_state.pressed_node_hash = None;
            }
        }
    }

    fn find_node_hash_by_cursor_position(&self, cursor_position: Vec2) -> Option<u64> {
        self.bounding_boxes
            .iter()
            .rev()
            .find(|(_, bbox)| bbox.contains(cursor_position))
            .map(|(h, _)| *h)
    }
}

pub struct InputState {
    cursor_position: Option<Vec2>,
    hovered_node_hash: Option<u64>,
    pressed_node_hash: Option<u64>,
    events: HashMap<u64, Vec<NodeInputEvent>>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            cursor_position: None,
            hovered_node_hash: None,
            pressed_node_hash: None,
            events: HashMap::new(),
        }
    }

    pub fn get_node_input_state(&self, node_hash: u64) -> NodeInputState {
        NodeInputState {
            is_hovered: self.hovered_node_hash == Some(node_hash),
            is_pressed: self.pressed_node_hash == Some(node_hash),
            events: self.events.get(&node_hash).cloned().unwrap_or_else(|| Vec::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeInputState {
    is_hovered: bool,
    is_pressed: bool,
    events: Vec<NodeInputEvent>,
}

impl NodeInputState {
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    pub fn on_press(&self) -> bool {
        self.events
            .iter()
            .find(|e| {
                **e == NodeInputEvent::MouseEvent {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                }
            })
            .is_some()
    }

    pub fn on_release(&self) -> bool {
        self.events
            .iter()
            .find(|e| {
                **e == NodeInputEvent::MouseEvent {
                    button: MouseButton::Left,
                    state: ElementState::Released,
                }
            })
            .is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeInputEvent {
    MouseEvent { button: MouseButton, state: ElementState },
}
