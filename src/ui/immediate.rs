use bitflags::bitflags;

use super::{BlockProps, Extent, Modifiers, UiNode};

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
}

pub fn ui(f: impl FnOnce(&mut Ui)) -> UiNode {
    let mut root = Ui { children: vec![] };

    f(&mut root);

    UiNode::Block(BlockProps {
        modifiers: Modifiers::new().width(Extent::FillParent).height(Extent::FillParent),
        children: root.children,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        ui::{BlockProps, Extent, Modifiers, UiNode},
        vertex::Color,
    };

    use super::{ui, Ui};

    fn immediate_test(f: impl FnOnce(&mut Ui), expected: &[UiNode]) {
        let ui = ui(f);

        assert_eq!(
            ui,
            UiNode::Block(BlockProps {
                modifiers: Modifiers::new().width(Extent::FillParent).height(Extent::FillParent),
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
                    *attr = attr.clone().border_color(Color::new(1.0, 0.0, 0.0, 0.0));
                });
                ui.block(|_, _, _| {});
            },
            &[
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new(),
                    children: vec![],
                }),
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new().border_color(Color::new(1.0, 0.0, 0.0, 0.0)),
                    children: vec![],
                }),
                UiNode::Block(BlockProps {
                    modifiers: Modifiers::new(),
                    children: vec![],
                }),
            ],
        );
    }
}
