use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::{
    ui::{BlockProps, ColumnProps, Extent, HashNode, Modifiers, RowProps, TextProps, UiNode},
    vertex::Color,
};

use super::{
    set_state::{set_state_channel, SetStateSender},
    UiNodeData, UiNodeType, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT,
};

pub struct Ui<'a> {
    node_data_map: &'a HashMap<u64, UiNodeData>,
    state_map: &'a HashMap<u64, Vec<u8>>,
    set_state_tx: &'a SetStateSender,
    path_hasher: DefaultHasher,
    children: Vec<UiNode>,
    children_path_hash_nodes: Vec<HashNode>,
}

impl<'a> Ui<'a> {
    pub fn new(
        node_data_map: &'a HashMap<u64, UiNodeData>,
        state_map: &'a HashMap<u64, Vec<u8>>,
        set_state_tx: &'a SetStateSender,
    ) -> Self {
        Self {
            node_data_map,
            state_map,
            set_state_tx,
            path_hasher: DefaultHasher::new(),
            children: vec![],
            children_path_hash_nodes: vec![],
        }
    }

    pub fn finish(self) -> (Vec<UiNode>, Vec<HashNode>) {
        (self.children, self.children_path_hash_nodes)
    }

    fn node(
        &mut self,
        node_type: UiNodeType,
        make_node: impl FnOnce(DefaultHasher, UiNodeData) -> (UiNode, Vec<HashNode>),
    ) {
        let child_path_hasher = self.compute_child_path_hasher(node_type);
        let child_path_hash = child_path_hasher.finish();

        let node_data = self.get_node_data_for_path_or_default(child_path_hash);

        let (ui_node, children_path_hash_nodes) = make_node(child_path_hasher, node_data);

        let path_hash_node = HashNode {
            hash: child_path_hash,
            children: children_path_hash_nodes,
        };

        assert_eq!(ui_node.node_type(), node_type);

        self.children.push(ui_node);
        self.children_path_hash_nodes.push(path_hash_node);
    }

    fn node_with_children(
        &mut self,
        node_type: UiNodeType,
        make_node: impl FnOnce(Ui, UiNodeData) -> (UiNode, Vec<HashNode>),
    ) {
        self.node(node_type, |child_path_hasher, node_data| {
            let ui = Ui {
                node_data_map: self.node_data_map,
                state_map: self.state_map,
                set_state_tx: self.set_state_tx,
                path_hasher: child_path_hasher,
                children: vec![],
                children_path_hash_nodes: vec![],
            };

            let (ui_node, children_path_hash_nodes) = make_node(ui, node_data);

            (ui_node, children_path_hash_nodes)
        });
    }

    fn node_without_children(
        &mut self,
        node_type: UiNodeType,
        make_node: impl FnOnce(DefaultHasher, UiNodeData) -> UiNode,
    ) {
        self.node(node_type, |child_path_hasher, node_data| {
            (make_node(child_path_hasher, node_data), vec![])
        });
    }

    pub fn block(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node_with_children(UiNodeType::Block, |mut ui, node_data| {
            let mut modifiers = Modifiers::new();
            f(&mut ui, &mut modifiers, node_data);

            let ui_node = UiNode::Block(BlockProps {
                modifiers,
                children: ui.children,
            });

            (ui_node, ui.children_path_hash_nodes)
        });
    }

    pub fn column(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node_with_children(UiNodeType::Column, |mut ui, node_data| {
            let mut modifiers = Modifiers::new();
            f(&mut ui, &mut modifiers, node_data);

            let ui_node = UiNode::Column(ColumnProps {
                modifiers,
                children: ui.children,
            });

            (ui_node, ui.children_path_hash_nodes)
        });
    }

    pub fn row(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node_with_children(UiNodeType::Row, |mut ui, node_data| {
            let mut modifiers = Modifiers::new();
            f(&mut ui, &mut modifiers, node_data);

            let ui_node = UiNode::Row(RowProps {
                modifiers,
                children: ui.children,
            });

            (ui_node, ui.children_path_hash_nodes)
        });
    }

    pub fn text(&mut self, text: impl Into<String>, f: impl FnOnce(&mut TextProps, UiNodeData)) {
        self.node_without_children(UiNodeType::Text, |_, node_data| {
            let mut props = TextProps {
                text: text.into(),
                text_color: Color::new(1.0, 1.0, 1.0, 1.0),
                font_family: String::new(),
                font_size: DEFAULT_FONT_SIZE,
                line_height: DEFAULT_LINE_HEIGHT,
                modifiers: Modifiers::new()
                    .width(Extent::FitContent)
                    .max_width(Extent::FillParent)
                    .height(Extent::FitContent)
                    .clone(),
            };

            f(&mut props, node_data);

            UiNode::Text(props)
        });
    }

    pub fn use_state<T>(&mut self, key: &str, initial_value: impl FnOnce() -> T) -> (T, Box<dyn Fn(&T)>)
    where
        T: Serialize + Deserialize<'a>,
    {
        let mut state_path_hasher = self.path_hasher.clone();
        key.hash(&mut state_path_hasher);
        let state_path_hash = state_path_hasher.finish();

        let set_state_tx = self.set_state_tx.clone();
        let set_state = Box::new(move |s: &T| {
            let bytes = match bincode::serialize(s) {
                Ok(bytes) => bytes,
                Err(err) => {
                    log::error!(
                        "set_state: failed to serialize value of type {}",
                        std::any::type_name::<T>()
                    );
                    log::error!("{}", err);
                    return;
                }
            };

            set_state_tx.send(state_path_hash, bytes);
        });

        let state = match self.state_map.get(&state_path_hash) {
            Some(state_bytes) => match bincode::deserialize(state_bytes) {
                Ok(state) => state,
                Err(err) => {
                    log::error!(
                        "set_state: failed to deserialize value of type {}. Returning initial value.",
                        std::any::type_name::<T>()
                    );
                    log::error!("{}", err);
                    initial_value()
                }
            },
            None => {
                let state = initial_value();
                set_state(&state);
                state
            }
        };

        (state, set_state)
    }

    fn compute_child_path_hasher(&self, child_node_type: UiNodeType) -> DefaultHasher {
        let mut child_path_hasher = self.path_hasher.clone();
        // TODO: this os O(n^2). Should keep track of counts.
        let position = self
            .children
            .iter()
            .filter(|c| c.node_type() == child_node_type)
            .count() as u32;
        child_path_hasher.write_u32(position);
        child_node_type.hash(&mut child_path_hasher);
        child_path_hasher
    }

    fn get_node_data_for_path_or_default(&self, path_hash: u64) -> UiNodeData {
        self.node_data_map.get(&path_hash).cloned().unwrap_or_default()
    }
}

#[allow(unused)]
pub fn mock_ui(f: impl FnOnce(&mut Ui<'_>)) -> UiNode {
    let node_data_map = HashMap::new();
    let state_map = HashMap::new();
    let (set_state_tx, _) = set_state_channel();

    let mut ui = Ui::new(&node_data_map, &state_map, &set_state_tx);
    f(&mut ui);
    let (children, _) = ui.finish();

    let ui_node = UiNode::Block(BlockProps {
        modifiers: Modifiers::new()
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .clone(),
        children,
    });

    ui_node
}
