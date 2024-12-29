use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use bitflags::bitflags;
use glam::Vec2;

use crate::{font::font_engine::FontEngine, input::InputEvent, rectangle::Rectangle, vertex::Color};

use super::{
    draw_element::DrawElement, to_draw_data, BlockProps, ColumnProps, Extent, HashNode, Modifiers, RowProps, TextProps,
    UiNode,
};

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_LINE_HEIGHT: f32 = DEFAULT_FONT_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiNodeData {
    flags: UiNodeDataFlags,
}

impl Default for UiNodeData {
    fn default() -> Self {
        Self {
            flags: UiNodeDataFlags::empty(),
        }
    }
}

impl UiNodeData {
    pub fn hovered(&self) -> bool {
        self.flags.contains(UiNodeDataFlags::HOVERED)
    }

    pub fn on_hover(&self) -> bool {
        self.flags.contains(UiNodeDataFlags::ON_HOVER)
    }

    pub fn pressed(&self) -> bool {
        self.flags.contains(UiNodeDataFlags::PRESSED)
    }

    pub fn on_press(&self) -> bool {
        self.flags.contains(UiNodeDataFlags::ON_PRESS)
    }

    pub fn on_release(&self) -> bool {
        self.flags.contains(UiNodeDataFlags::ON_RELEASE)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UiNodeDataFlags: u32 {
        const ON_HOVER = 1 << 0;
        const HOVERED = 1 << 1;
        const PRESSED = 1 << 2;
        const ON_PRESS = 1 << 3;
        const ON_RELEASE = 1 << 4;
    }
}

pub struct UiContext {
    previous_nodes_data: HashMap<u64, UiNodeData>,
    nodes_data: HashMap<u64, UiNodeData>,
    bounding_boxes: Vec<(u64, Rectangle)>,
    cursor_position: Vec2,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            previous_nodes_data: HashMap::new(),
            nodes_data: HashMap::new(),
            bounding_boxes: Vec::new(),
            cursor_position: Vec2::ZERO,
        }
    }

    pub fn process_input_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::CursorMoved { position } => {
                self.cursor_position = position;
                self.on_cursor_moved();
            }
            InputEvent::MouseInput { button, state } => {}
        }
    }

    pub fn build_ui<F: FontEngine>(
        &mut self,
        window_size: Vec2,
        font_engine: &mut F,
        f: impl FnOnce(&mut Ui),
    ) -> Vec<DrawElement> {
        let root_hasher = DefaultHasher::new();

        let mut root_ui = Ui {
            node_data_map: &self.nodes_data,
            path_hasher: root_hasher,
            children: vec![],
            children_path_hash_nodes: vec![],
        };

        f(&mut root_ui);

        let mut draw_data = vec![];
        self.bounding_boxes.clear();
        to_draw_data(
            root_ui.children,
            root_ui.children_path_hash_nodes,
            Vec2::ZERO,
            window_size,
            font_engine,
            &mut draw_data,
            &mut self.bounding_boxes,
        );

        self.previous_nodes_data = self.nodes_data.clone();
        self.on_cursor_moved();

        draw_data
    }

    fn on_cursor_moved(&mut self) {
        self.nodes_data
            .iter_mut()
            .for_each(|(_, d)| d.flags = UiNodeDataFlags::empty());

        for (hash, bbox) in &self.bounding_boxes {
            if bbox.contains(self.cursor_position) {
                let node_data = self.nodes_data.entry(*hash).or_default();

                node_data.flags.insert(UiNodeDataFlags::HOVERED);

                if let Some(old_node_data) = self.previous_nodes_data.get(hash) {
                    if !old_node_data.flags.contains(UiNodeDataFlags::HOVERED) {
                        node_data.flags.insert(UiNodeDataFlags::ON_HOVER);
                    }
                }
            }
        }
    }
}

pub struct Ui<'a> {
    node_data_map: &'a HashMap<u64, UiNodeData>,
    path_hasher: DefaultHasher,
    children: Vec<UiNode>,
    children_path_hash_nodes: Vec<HashNode>,
}

impl<'a> Ui<'a> {
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum UiNodeType {
    Block,
    Column,
    Row,
    Text,
}

impl UiNode {
    fn node_type(&self) -> UiNodeType {
        match self {
            UiNode::Block(_) => UiNodeType::Block,
            UiNode::Column(_) => UiNodeType::Column,
            UiNode::Row(_) => UiNodeType::Row,
            UiNode::Text(_) => UiNodeType::Text,
        }
    }
}

#[allow(unused)]
fn mock_ui(node_data_map: &HashMap<u64, UiNodeData>) -> Ui<'_> {
    Ui {
        node_data_map,
        path_hasher: DefaultHasher::new(),
        children: vec![],
        children_path_hash_nodes: vec![],
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::{
        ui::{immediate::mock_ui, BlockProps, ColumnProps, Extent, Modifiers, RowProps, TextProps, UiNode},
        vertex::Color,
    };

    use super::{Ui, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT};

    fn immediate_test(f: impl FnOnce(&mut Ui), expected: &[UiNode]) {
        let node_data_map = HashMap::new();
        let mut ui = mock_ui(&node_data_map);
        f(&mut ui);
        let ui_node = UiNode::Block(BlockProps {
            modifiers: Modifiers::new()
                .width(Extent::FillParent)
                .height(Extent::FillParent)
                .clone(),
            children: ui.children,
        });

        assert_eq!(
            ui_node,
            UiNode::Block(BlockProps {
                modifiers: Modifiers::new()
                    .width(Extent::FillParent)
                    .height(Extent::FillParent)
                    .clone(),
                children: expected.to_vec(),
            })
        )
    }

    #[test]
    fn single_block() {
        immediate_test(
            |ui| {
                ui.block(|_, _, _| {});
            },
            &[UiNode::Block(BlockProps {
                modifiers: Modifiers::new(),
                children: vec![],
            })],
        );
    }

    #[test]
    fn multiple_blocks() {
        immediate_test(
            |ui| {
                ui.block(|_, _, _| {});
                ui.block(|_, _, _| {});
                ui.block(|_, _, _| {});
            },
            &[
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new(),
                    children: vec![],
                }),
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new(),
                    children: vec![],
                }),
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new(),
                    children: vec![],
                }),
            ],
        );
    }

    #[test]
    fn multiple_blocks_with_custom_attributes() {
        immediate_test(
            |ui| {
                ui.block(|_, _, _| {});
                ui.block(|_, attr, _| {
                    attr.border_color(Color::new(1.0, 0.0, 0.0, 0.0));
                });
                ui.block(|_, _, _| {});
            },
            &[
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new(),
                    children: vec![],
                }),
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new().border_color(Color::new(1.0, 0.0, 0.0, 0.0)).clone(),
                    children: vec![],
                }),
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new(),
                    children: vec![],
                }),
            ],
        );
    }

    #[test]
    fn single_column() {
        immediate_test(
            |ui| {
                ui.column(|_, _, _| {});
            },
            &[UiNode::Column(ColumnProps {
                modifiers: Modifiers::new(),
                children: vec![],
            })],
        );
    }

    #[test]
    fn single_row() {
        immediate_test(
            |ui| {
                ui.row(|_, _, _| {});
            },
            &[UiNode::Row(RowProps {
                modifiers: Modifiers::new(),
                children: vec![],
            })],
        );
    }

    #[test]
    fn nested_block_column_row() {
        immediate_test(
            |ui| {
                ui.block(|ui, _, _| {
                    ui.column(|ui, _, _| {
                        ui.row(|_, _, _| {});
                    });
                });
            },
            &[UiNode::Block(BlockProps {
                modifiers: Modifiers::new(),
                children: vec![UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new(),
                    children: vec![UiNode::Row(RowProps {
                        modifiers: Modifiers::new(),
                        children: vec![],
                    })],
                })],
            })],
        );
    }

    #[test]
    fn single_text() {
        immediate_test(
            |ui| {
                ui.text("Hello", |_, _| {});
            },
            &[UiNode::Text(TextProps {
                text: "Hello".into(),
                text_color: Color::new(1.0, 1.0, 1.0, 1.0),
                font_family: String::new(),
                font_size: DEFAULT_FONT_SIZE,
                line_height: DEFAULT_LINE_HEIGHT,
                modifiers: Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .max_width(Extent::FillParent)
                    .clone(),
            })],
        );
    }
}
