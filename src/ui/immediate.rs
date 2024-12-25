use bitflags::bitflags;

use crate::vertex::Color;

use super::{BlockProps, ColumnProps, Extent, Modifiers, RowProps, TextProps, UiNode};

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_LINE_HEIGHT: f32 = DEFAULT_FONT_SIZE;

#[derive(Debug)]
pub struct UiNodeData {
    flags: UiNodeDataFlags,
}

bitflags! {
    #[derive(Debug)]
    pub struct UiNodeDataFlags: u32 {
        const ON_HOVER = 1 << 0;
        const HOVERED = 1 << 1;
        const ON_PRESS = 1 << 2;
        const PRESSED = 1 << 3;
    }
}

pub struct Ui {
    children: Vec<UiNode>,
}

impl Ui {
    pub fn block(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        let mut ui = Ui { children: vec![] };
        let mut modifiers = Modifiers::new();

        f(
            &mut ui,
            &mut modifiers,
            UiNodeData {
                flags: UiNodeDataFlags::empty(),
            },
        );

        self.children.push(UiNode::Block(BlockProps {
            modifiers,
            children: ui.children,
        }));
    }

    pub fn column(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        let mut ui = Ui { children: vec![] };
        let mut modifiers = Modifiers::new();

        f(
            &mut ui,
            &mut modifiers,
            UiNodeData {
                flags: UiNodeDataFlags::empty(),
            },
        );

        self.children.push(UiNode::Column(ColumnProps {
            modifiers,
            children: ui.children,
        }));
    }

    pub fn row(&mut self, f: impl FnOnce(&mut Ui, &mut Modifiers, UiNodeData)) {
        let mut ui = Ui { children: vec![] };
        let mut modifiers = Modifiers::new();

        f(
            &mut ui,
            &mut modifiers,
            UiNodeData {
                flags: UiNodeDataFlags::empty(),
            },
        );

        self.children.push(UiNode::Row(RowProps {
            modifiers,
            children: ui.children,
        }));
    }

    pub fn text(&mut self, text: impl Into<String>, f: impl FnOnce(&mut TextProps, UiNodeData)) {
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

        f(
            &mut props,
            UiNodeData {
                flags: UiNodeDataFlags::empty(),
            },
        );

        self.children.push(UiNode::Text(props));
    }
}

pub fn ui(f: impl FnOnce(&mut Ui)) -> UiNode {
    let mut root = Ui { children: vec![] };

    f(&mut root);

    UiNode::Block(BlockProps {
        modifiers: Modifiers::new()
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .clone(),
        children: root.children,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        ui::{BlockProps, ColumnProps, Extent, Modifiers, RowProps, TextProps, UiNode},
        vertex::Color,
    };

    use super::{ui, Ui, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT};

    fn immediate_test(f: impl FnOnce(&mut Ui), expected: &[UiNode]) {
        let ui = ui(f);

        assert_eq!(
            ui,
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
