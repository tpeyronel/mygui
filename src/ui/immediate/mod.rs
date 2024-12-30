pub mod context;
mod set_state;
pub mod ui;

use bitflags::bitflags;

use super::UiNode;

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

#[macro_export]
macro_rules! use_state {
    ($ui:expr, $init:expr) => {
        $ui.use_state(concat!(std::file!(), std::line!(), std::column!()), $init)
    };
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        ui::{immediate::ui::mock_ui, BlockProps, ColumnProps, Extent, Modifiers, RowProps, TextProps, UiNode},
        vertex::Color,
    };

    use super::{ui::Ui, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT};

    fn immediate_test(f: impl FnOnce(&mut Ui), expected: &[UiNode]) {
        let ui_node = mock_ui(f);

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
