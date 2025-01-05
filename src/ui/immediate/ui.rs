use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
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
    context::{InputState, NodeInputState},
    set_state::{set_state_channel, SetStateSender},
    DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UiNodeType(TypeId);

impl UiNode {
    fn node_type(&self) -> UiNodeType {
        UiNodeType((*self.props).type_id())
    }
}

pub struct Ui<'a> {
    input_state: &'a InputState,
    state_map: &'a HashMap<u64, Vec<u8>>,
    set_state_tx: &'a SetStateSender,
    ref_map: &'a mut HashMap<u64, Rc<dyn Any>>,
    path_hash: u64,
    path_hasher: DefaultHasher,
    children: Vec<UiNode>,
    children_path_hash_nodes: Vec<HashNode>,
}

impl<'a> Ui<'a> {
    pub fn new(
        input_state: &'a InputState,
        state_map: &'a HashMap<u64, Vec<u8>>,
        set_state_tx: &'a SetStateSender,
        ref_map: &'a mut HashMap<u64, Rc<dyn Any>>,
    ) -> Self {
        Self {
            input_state,
            state_map,
            set_state_tx,
            ref_map,
            path_hash: DefaultHasher::new().finish(),
            path_hasher: DefaultHasher::new(),
            children: vec![],
            children_path_hash_nodes: vec![],
        }
    }

    pub fn finish(self) -> (Vec<UiNode>, Vec<HashNode>) {
        (self.children, self.children_path_hash_nodes)
    }

    fn node<P: UiNodeProps>(&mut self, make_props: impl FnOnce(&mut Ui, &mut Modifiers) -> P) {
        let node_type = UiNodeType(TypeId::of::<P>());
        let child_path_hasher = self.compute_child_path_hasher(node_type);
        let child_path_hash = child_path_hasher.finish();

        let mut ui = Ui {
            input_state: self.input_state,
            state_map: self.state_map,
            set_state_tx: self.set_state_tx,
            ref_map: self.ref_map,
            path_hash: child_path_hash,
            path_hasher: child_path_hasher,
            children: vec![],
            children_path_hash_nodes: vec![],
        };

        let mut modifiers = Modifiers::new();

        let props = make_props(&mut ui, &mut modifiers);

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

    fn leaf_node<P: UiNodeProps>(&mut self, make_props: impl FnOnce(&mut Ui, &mut Modifiers) -> P) {
        self.node(|ui, modifiers| {
            let props = make_props(ui, modifiers);
            assert!(ui.children.is_empty(), "leaf node may not have children!");
            props
        });
    }

    pub fn block(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers)) {
        self.node(|ui, modifiers| {
            f(ui, modifiers);

            BlockProps
        });
    }

    pub fn column(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers)) {
        self.node(|ui, modifiers| {
            f(ui, modifiers);

            ColumnProps
        });
    }

    pub fn row(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers)) {
        self.node(|ui, modifiers| {
            f(ui, modifiers);

            RowProps
        });
    }

    pub fn text(&mut self, text: impl Into<String>, f: impl FnOnce(&mut Ui, &mut TextProps, &mut Modifiers)) {
        self.leaf_node(|ui, modifiers| {
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
                cursor_position: None,
            };

            f(ui, &mut props, modifiers);

            props
        });
    }

    pub fn use_state<T, F>(&mut self, initial_value: F) -> (T, Box<dyn Fn(&T)>)
    where
        T: Serialize + Deserialize<'a>,
        F: FnOnce() -> T + 'static,
    {
        let mut state_path_hasher = self.path_hasher.clone();
        TypeId::of::<F>().hash(&mut state_path_hasher);
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

    pub fn use_ref<T, F>(&mut self, initial_value: F) -> Rc<RefCell<T>>
    where
        T: Any + 'static,
        F: FnOnce() -> T + 'static,
    {
        let mut state_path_hasher = self.path_hasher.clone();
        TypeId::of::<F>().hash(&mut state_path_hasher);
        let state_path_hash = state_path_hasher.finish();

        let rc = match self.ref_map.entry(state_path_hash) {
            Entry::Occupied(mut occupied_entry) => {
                let rc = occupied_entry.get_mut();

                match rc.clone().downcast::<RefCell<T>>() {
                    Ok(r) => r,
                    Err(_) => {
                        log::error!(
                            "failed to downcast stored use_ref to requested type. Overwriting with initial value."
                        );

                        let new_rc = Rc::new(RefCell::new(initial_value()));
                        *rc = Rc::clone(&new_rc) as Rc<dyn Any>;
                        new_rc
                    }
                }
            }
            Entry::Vacant(vacant_entry) => {
                let new_rc: Rc<RefCell<T>> = Rc::new(RefCell::new(initial_value()));
                vacant_entry.insert(Rc::clone(&new_rc) as Rc<dyn Any>);
                new_rc
            }
        };

        rc
    }

    pub fn use_input(&mut self) -> NodeInputState {
        self.input_state.get_node_input_state(self.path_hash)
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
}

#[allow(unused)]
pub fn mock_ui(f: impl FnOnce(&mut Ui<'_>)) -> UiNode {
    let input_state = InputState::new();
    let state_map = HashMap::new();
    let (set_state_tx, _) = set_state_channel();
    let mut ref_map = HashMap::new();

    let mut ui = Ui::new(&input_state, &state_map, &set_state_tx, &mut ref_map);
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
