use bitflags::bitflags;

use super::{BlockProps, Modifiers, UiNode};

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
    pub fn block(&mut self, f: impl FnOnce(&mut Ui, UiNodeData)) {
        let mut ui = Ui { children: vec![] };

        f(
            &mut ui,
            UiNodeData {
                flags: UiNodeDataFlags::empty(),
            },
        );

        self.children.push(UiNode::Block(BlockProps {
            modifiers: Modifiers::new(),
            children: ui.children,
        }));
    }
}

pub fn ui(f: impl FnOnce(&mut Ui)) -> UiNode {
    let mut root = Ui { children: vec![] };

    f(&mut root);

    UiNode::Block(BlockProps {
        modifiers: Modifiers::new(),
        children: root.children,
    })
}

#[cfg(test)]
mod tests {
    use crate::ui::{BlockProps, Modifiers, UiNode};

    use super::ui;

    #[test]
    fn single_block() {
        let ui = ui(|ui| {
            ui.block(|_, _| {});
        });

        assert_eq!(
            ui,
            UiNode::Block(BlockProps {
                modifiers: Modifiers::new(),
                children: vec![UiNode::Block(BlockProps {
                    modifiers: Modifiers::new(),
                    children: vec![]
                })]
            })
        )
    }

    #[test]
    fn two_blocks() {
        let ui = ui(|ui| {
            ui.block(|_, _| {});
            ui.block(|_, _| {});
        });

        assert_eq!(
            ui,
            UiNode::Block(BlockProps {
                modifiers: Modifiers::new(),
                children: vec![
                    UiNode::Block(BlockProps {
                        modifiers: Modifiers::new(),
                        children: vec![]
                    }),
                    UiNode::Block(BlockProps {
                        modifiers: Modifiers::new(),
                        children: vec![]
                    })
                ]
            })
        )
    }
}
