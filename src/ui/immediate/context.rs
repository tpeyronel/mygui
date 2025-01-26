use std::{any::Any, collections::HashMap, rc::Rc};

use glam::Vec2;

use crate::{
    font::font_engine::FontEngine,
    input::{ElementState, InputEvent, MouseButton, TextEvent},
    mesh::mesh_manager::MeshManager,
    rectangle::Rectangle,
    ui::{draw_command::DrawCommand, node::block::BlockProps, processor::UiNodeProcessor},
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
            InputEvent::TextEvent(text_event) => self.handle_text_event(text_event),
        }
    }

    pub fn build_ui(
        &mut self,
        window_size: Vec2,
        mesh_manager: &mut MeshManager,
        font_engine: &mut Box<dyn FontEngine>,
        f: impl FnOnce(&mut Ui<BlockProps>),
    ) -> Vec<DrawCommand> {
        let (set_state_tx, set_state_rx) = set_state_channel();

        let mut root_ui = Ui::new(
            BlockProps,
            &self.input_state,
            &self.states,
            &set_state_tx,
            &mut self.refs,
        );
        f(&mut root_ui);
        let (children, children_path_hash_nodes) = root_ui.finish();

        let mut draw_data = vec![];
        self.bounding_boxes.clear();
        UiNodeProcessor::process_ui(
            children,
            children_path_hash_nodes,
            Vec2::ZERO,
            window_size,
            mesh_manager,
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

        let new_hovered_node_hash = cursor_position
            .map(|p| self.find_node_hash_by_cursor_position(p))
            .flatten();

        match (self.input_state.hovered_node_hash, new_hovered_node_hash) {
            (None, None) => (),
            (None, Some(new_hovered)) => {
                self.input_state.emit(
                    new_hovered,
                    NodeInputEvent::MouseEvent(MouseEvent::MouseHoverEvent { hovered: true }),
                );
            }
            (Some(old_hovered), None) => {
                self.input_state.emit(
                    old_hovered,
                    NodeInputEvent::MouseEvent(MouseEvent::MouseHoverEvent { hovered: false }),
                );
            }
            (Some(old_hovered), Some(new_hovered)) => {
                if old_hovered != new_hovered {
                    self.input_state.emit(
                        old_hovered,
                        NodeInputEvent::MouseEvent(MouseEvent::MouseHoverEvent { hovered: false }),
                    );
                    self.input_state.emit(
                        new_hovered,
                        NodeInputEvent::MouseEvent(MouseEvent::MouseHoverEvent { hovered: true }),
                    );
                }
            }
        }

        self.input_state.hovered_node_hash = new_hovered_node_hash;
    }

    fn on_lmb_state_changed(&mut self, state: ElementState) {
        match state {
            ElementState::Pressed => {
                let Some(hovered_node_hash) = self.input_state.hovered_node_hash else {
                    return;
                };

                self.input_state.emit(
                    hovered_node_hash,
                    NodeInputEvent::MouseEvent(MouseEvent::MouseButtonEvent {
                        button: MouseButton::Left,
                        state: ElementState::Pressed,
                    }),
                );
                self.input_state.pressed_node_hash = Some(hovered_node_hash);
                self.input_state.focused_node_hash = Some(hovered_node_hash);
            }
            ElementState::Released => {
                let Some(hovered_node_hash) = self.input_state.hovered_node_hash else {
                    return;
                };

                // Only emit Released event if element is also pressed.
                if self.input_state.pressed_node_hash == Some(hovered_node_hash) {
                    self.input_state.emit(
                        hovered_node_hash,
                        NodeInputEvent::MouseEvent(MouseEvent::MouseButtonEvent {
                            button: MouseButton::Left,
                            state: ElementState::Released,
                        }),
                    );
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

    fn handle_text_event(&mut self, e: TextEvent) {
        let Some(focused_node_hash) = self.input_state.focused_node_hash else {
            return;
        };

        self.input_state.emit(focused_node_hash, NodeInputEvent::TextEvent(e));
    }
}

pub struct InputState {
    cursor_position: Option<Vec2>,
    hovered_node_hash: Option<u64>,
    pressed_node_hash: Option<u64>,
    focused_node_hash: Option<u64>,
    events: HashMap<u64, Vec<NodeInputEvent>>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            cursor_position: None,
            hovered_node_hash: None,
            pressed_node_hash: None,
            focused_node_hash: None,
            events: HashMap::new(),
        }
    }

    pub fn get_node_input_state(&self, node_hash: u64) -> NodeInputState {
        NodeInputState {
            is_hovered: self.hovered_node_hash == Some(node_hash),
            is_pressed: self.pressed_node_hash == Some(node_hash),
            is_focused: self.focused_node_hash == Some(node_hash),
            events: self.events.get(&node_hash).cloned().unwrap_or_else(|| Vec::new()),
        }
    }

    fn emit(&mut self, node_hash: u64, event: NodeInputEvent) {
        self.events.entry(node_hash).or_insert_with(|| Vec::new()).push(event);
    }
}

#[derive(Debug, Clone)]
pub struct NodeInputState {
    is_hovered: bool,
    is_pressed: bool,
    is_focused: bool,
    events: Vec<NodeInputEvent>,
}

impl NodeInputState {
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    pub fn on_hover(&self) -> bool {
        self.has_event(NodeInputEvent::MouseEvent(MouseEvent::MouseHoverEvent {
            hovered: true,
        }))
    }

    pub fn on_unhover(&self) -> bool {
        self.has_event(NodeInputEvent::MouseEvent(MouseEvent::MouseHoverEvent {
            hovered: false,
        }))
    }

    pub fn on_press(&self) -> bool {
        self.has_event(NodeInputEvent::MouseEvent(MouseEvent::MouseButtonEvent {
            button: MouseButton::Left,
            state: ElementState::Pressed,
        }))
    }

    pub fn on_release(&self) -> bool {
        self.has_event(NodeInputEvent::MouseEvent(MouseEvent::MouseButtonEvent {
            button: MouseButton::Left,
            state: ElementState::Released,
        }))
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn events(&self) -> &[NodeInputEvent] {
        &self.events
    }

    fn has_event(&self, event: NodeInputEvent) -> bool {
        self.events.iter().find(|e| **e == event).is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeInputEvent {
    MouseEvent(MouseEvent),
    TextEvent(TextEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    MouseButtonEvent { button: MouseButton, state: ElementState },
    MouseHoverEvent { hovered: bool },
}
