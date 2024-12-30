use std::collections::HashMap;

use glam::Vec2;

use crate::{
    font::font_engine::FontEngine,
    input::{InputEvent, InputState, MouseButton},
    rectangle::Rectangle,
    ui::{draw_element::DrawElement, to_draw_data},
};

use super::{set_state::set_state_channel, ui::Ui, UiNodeData, UiNodeDataFlags};

pub struct UiContext {
    nodes_data: HashMap<u64, UiNodeData>,
    states: HashMap<u64, Vec<u8>>,
    bounding_boxes: Vec<(u64, Rectangle)>,
    cursor_position: Vec2,
    redraw_required: bool,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            nodes_data: HashMap::new(),
            states: HashMap::new(),
            bounding_boxes: Vec::new(),
            cursor_position: Vec2::ZERO,
            redraw_required: false,
        }
    }

    pub fn process_input_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::CursorMoved { position } => {
                self.cursor_position = position;
                self.on_cursor_moved();
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

    pub fn build_ui<F: FontEngine>(
        &mut self,
        window_size: Vec2,
        font_engine: &mut F,
        f: impl FnOnce(&mut Ui),
    ) -> Vec<DrawElement> {
        let (set_state_tx, set_state_rx) = set_state_channel();

        let mut root_ui = Ui::new(&self.nodes_data, &self.states, &set_state_tx);
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
            self.redraw_required = true;
        });

        self.nodes_data.iter_mut().for_each(|(_, d)| {
            d.flags
                .remove(UiNodeDataFlags::ON_PRESS | UiNodeDataFlags::ON_HOVER | UiNodeDataFlags::ON_RELEASE)
        });
        self.on_cursor_moved();

        draw_data
    }

    pub fn redraw_required(&self) -> bool {
        self.redraw_required
    }

    fn on_cursor_moved(&mut self) {
        let old_nodes_data = self.nodes_data.clone();
        self.nodes_data
            .iter_mut()
            .for_each(|(_, d)| d.flags.remove(UiNodeDataFlags::HOVERED | UiNodeDataFlags::PRESSED));

        for (hash, bbox) in &self.bounding_boxes {
            if !bbox.contains(self.cursor_position) {
                continue;
            }

            let node_data = self.nodes_data.entry(*hash).or_default();
            node_data.flags.insert(UiNodeDataFlags::HOVERED);

            let old_node_data = old_nodes_data.get(hash);
            if old_node_data.is_none_or(|d| !d.flags.contains(UiNodeDataFlags::HOVERED)) {
                node_data.flags.insert(UiNodeDataFlags::ON_HOVER);
            }

            if old_node_data.is_some_and(|d| d.flags.contains(UiNodeDataFlags::PRESSED)) {
                // Maintain PRESSED state when moving cursor inside element.
                node_data.flags.insert(UiNodeDataFlags::PRESSED);
            }
        }

        for (hash, old_node_data) in &old_nodes_data {
            let Some(node_data) = self.nodes_data.get_mut(hash) else {
                continue;
            };

            if old_node_data.flags.contains(UiNodeDataFlags::PRESSED)
                && !node_data.flags.contains(UiNodeDataFlags::PRESSED)
            {
                node_data.flags.insert(UiNodeDataFlags::ON_RELEASE);
            }
        }
    }

    fn on_lmb_state_changed(&mut self, state: InputState) {
        match state {
            InputState::Pressed => {
                for (hash, bbox) in &self.bounding_boxes {
                    if !bbox.contains(self.cursor_position) {
                        continue;
                    }

                    let node_data = self.nodes_data.entry(*hash).or_default();

                    if !node_data.flags.contains(UiNodeDataFlags::PRESSED) {
                        node_data.flags.insert(UiNodeDataFlags::PRESSED);
                        node_data.flags.insert(UiNodeDataFlags::ON_PRESS);
                    }
                }
            }
            InputState::Released => {
                self.nodes_data.iter_mut().for_each(|(_, node_data)| {
                    if node_data.flags.contains(UiNodeDataFlags::PRESSED) {
                        node_data.flags.remove(UiNodeDataFlags::PRESSED);
                        node_data.flags.insert(UiNodeDataFlags::ON_RELEASE);
                    }
                });
            }
        }
    }
}
