use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use bitflags::bitflags;
use glam::Vec2;

use crate::{font::font_engine::FontEngine, rectangle::Rectangle, vertex::Color};

use super::{
    draw_element::DrawElement, to_draw_data, BlockProps, ColumnProps, Extent, HashNode, Modifiers, RowProps, TextProps,
    UiNode,
};

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_LINE_HEIGHT: f32 = DEFAULT_FONT_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiNodeData {
    pub flags: UiNodeDataFlags,
}

impl Default for UiNodeData {
    fn default() -> Self {
        Self {
            flags: UiNodeDataFlags::empty(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UiNodeDataFlags: u32 {
        const ON_HOVER = 1 << 0;
        const HOVERED = 1 << 1;
        const ON_PRESS = 1 << 2;
        const PRESSED = 1 << 3;
    }
}

pub struct UiContext {
    nodes_data: HashMap<u64, UiNodeData>,
    cursor_position: Vec2,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            nodes_data: HashMap::new(),
            cursor_position: Vec2::ZERO,
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
        let mut bounding_boxes = vec![];
        to_draw_data(
            root_ui.children,
            root_ui.children_path_hash_nodes,
            Vec2::ZERO,
            window_size,
            font_engine,
            &mut draw_data,
            &mut bounding_boxes,
        );

        self.update_nodes_data(&bounding_boxes);

        draw_data
    }

    pub fn set_cursor_position(&mut self, cursor_position: Vec2) {
        self.cursor_position = cursor_position;
    }

    fn update_nodes_data(&mut self, bounding_boxes: &[(u64, Rectangle)]) {
        self.nodes_data.clear();

        for (hash, bbox) in bounding_boxes {
            if bbox.contains(self.cursor_position) {
                self.nodes_data.insert(
                    *hash,
                    UiNodeData {
                        flags: UiNodeDataFlags::HOVERED,
                    },
                );
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
    fn node_with_children(
        &mut self,
        node_type: UiNodeType,
        f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData),
        make_node: impl FnOnce(Vec<UiNode>, Modifiers) -> UiNode,
    ) {
        let child_path_hasher = self.compute_child_path_hasher(node_type);
        let child_path_hash = child_path_hasher.finish();

        let mut ui = Ui {
            node_data_map: self.node_data_map,
            path_hasher: child_path_hasher,
            children: vec![],
            children_path_hash_nodes: vec![],
        };
        let mut modifiers = Modifiers::new();
        let node_data = self.get_node_data_for_path_or_default(child_path_hash);

        f(&mut ui, &mut modifiers, node_data);

        let ui_node = make_node(ui.children, modifiers);

        let path_hash_node = HashNode {
            hash: child_path_hash,
            children: ui.children_path_hash_nodes,
        };

        assert_eq!(ui_node.node_type(), node_type);

        self.children.push(ui_node);
        self.children_path_hash_nodes.push(path_hash_node);
    }

    pub fn block(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node_with_children(UiNodeType::Block, f, |children, modifiers| {
            UiNode::Block(BlockProps { modifiers, children })
        });
    }

    pub fn column(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node_with_children(UiNodeType::Column, f, |children, modifiers| {
            UiNode::Column(ColumnProps { modifiers, children })
        });
    }

    pub fn row(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        self.node_with_children(UiNodeType::Row, f, |children, modifiers| {
            UiNode::Row(RowProps { modifiers, children })
        });
    }

    pub fn text(&mut self, text: impl Into<String>, f: impl FnOnce(&mut TextProps, UiNodeData)) {
        let child_path_hasher = self.compute_child_path_hasher(UiNodeType::Text);
        let child_path_hash = child_path_hasher.finish();

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
        let node_data = self.get_node_data_for_path_or_default(child_path_hash);

        f(&mut props, node_data);

        self.children.push(UiNode::Text(props));
        self.children_path_hash_nodes.push(HashNode {
            hash: child_path_hash,
            children: vec![],
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
