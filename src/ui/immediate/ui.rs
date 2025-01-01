use std::{
    any::TypeId,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::{
    ui::{
        node::{block::BlockProps, column::ColumnProps, row::RowProps, text::TextProps, UiNodeProps},
        Extent, HashNode, Modifiers, UiNode,
    },
    vertex::Color,
};

use super::{
    set_state::{set_state_channel, SetStateSender},
    UiNodeData, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UiNodeType(TypeId);

impl UiNode {
    fn node_type(&self) -> UiNodeType {
        UiNodeType((*self.props).type_id())
    }
}

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

    fn node<P: UiNodeProps>(&mut self, make_props: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData) -> P) {
        let node_type = UiNodeType(TypeId::of::<P>());
        let child_path_hasher = self.compute_child_path_hasher(node_type);
        let child_path_hash = child_path_hasher.finish();

        let node_data = self.get_node_data_for_path_or_default(child_path_hash);

        let mut ui = Ui {
            node_data_map: self.node_data_map,
            state_map: self.state_map,
            set_state_tx: self.set_state_tx,
            path_hasher: child_path_hasher,
            children: vec![],
            children_path_hash_nodes: vec![],
        };

        let mut modifiers = Modifiers::new();

        let props = make_props(&mut ui, &mut modifiers, node_data);

        let ui_node = UiNode {
            props: Box::new(props),
            modifiers,
            children: ui.children,
        };

        let path_hash_node = HashNode {
            hash: child_path_hash,
            children: ui.children_path_hash_nodes,
        };

        assert_eq!(ui_node.node_type(), node_type);

        self.children.push(ui_node);
        self.children_path_hash_nodes.push(path_hash_node);
    }

    fn leaf_node<P: UiNodeProps>(&mut self, make_props: impl FnOnce(&mut Modifiers, UiNodeData) -> P) {
        self.node(|_, modifiers, node_data| make_props(modifiers, node_data));
    }

    pub fn block(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node(|ui, modifiers, node_data| {
            f(ui, modifiers, node_data);

            BlockProps
        });
    }

    pub fn column(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node(|ui, modifiers, node_data| {
            f(ui, modifiers, node_data);

            ColumnProps
        });
    }

    pub fn row(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node(|ui, modifiers, node_data| {
            f(ui, modifiers, node_data);

            RowProps
        });
    }

    pub fn text(&mut self, text: impl Into<String>, f: impl FnOnce(&mut TextProps, &mut Modifiers, UiNodeData)) {
        self.leaf_node(|modifiers, node_data| {
            modifiers
                .width(Extent::FitContent)
                .max_width(Extent::FillParent)
                .height(Extent::FitContent);

            let mut props = TextProps {
                text: text.into(),
                text_color: Color::new(1.0, 1.0, 1.0, 1.0),
                font_family: String::new(),
                font_size: DEFAULT_FONT_SIZE,
                line_height: DEFAULT_LINE_HEIGHT,
            };

            f(&mut props, modifiers, node_data);

            props
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

    let ui_node = UiNode {
        props: Box::new(BlockProps),
        modifiers: Modifiers::new()
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .clone(),
        children,
    };

    ui_node
}
