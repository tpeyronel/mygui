pub mod base_text_field;
pub mod context;
mod set_state;
pub mod ui;

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_LINE_HEIGHT: f32 = DEFAULT_FONT_SIZE;

// #[macro_export]
// macro_rules! use_state {
//     ($ui:expr, $init:expr) => {
//         $ui.use_state(concat!(std::file!(), std::line!(), std::column!()), $init)
//     };
// }

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        color::Color,
        ui::{
            immediate::ui::mock_ui,
            node::{block::BlockProps, column::ColumnProps, row::RowProps, text::TextProps},
            Extent, Modifiers, UiNode,
        },
    };

    use super::{ui::Ui, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT};

    fn immediate_test(f: impl FnOnce(&mut Ui), expected: Vec<UiNode>) {
        let ui_node = mock_ui(f);

        assert_eq!(
            ui_node,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(Extent::FillParent)
                    .height(Extent::FillParent)
                    .clone(),
                expected,
            )
        )
    }

    #[test]
    fn single_block() {
        immediate_test(
            |ui| {
                ui.block(|_, _| {});
            },
            vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
        );
    }

    #[test]
    fn multiple_blocks() {
        immediate_test(
            |ui| {
                ui.block(|_, _| {});
                ui.block(|_, _| {});
                ui.block(|_, _| {});
            },
            vec![
                UiNode::new(BlockProps, Modifiers::new(), vec![]),
                UiNode::new(BlockProps, Modifiers::new(), vec![]),
                UiNode::new(BlockProps, Modifiers::new(), vec![]),
            ],
        );
    }

    #[test]
    fn multiple_blocks_with_custom_attributes() {
        immediate_test(
            |ui| {
                ui.block(|_, _| {});
                ui.block(|_, attr| {
                    attr.border_color(Color::new(1.0, 0.0, 0.0, 0.0));
                });
                ui.block(|_, _| {});
            },
            vec![
                UiNode::new(BlockProps, Modifiers::new(), vec![]),
                UiNode::new(
                    BlockProps,
                    Modifiers::new().border_color(Color::new(1.0, 0.0, 0.0, 0.0)).clone(),
                    vec![],
                ),
                UiNode::new(BlockProps, Modifiers::new(), vec![]),
            ],
        );
    }

    #[test]
    fn single_column() {
        immediate_test(
            |ui| {
                ui.column(|_, _| {});
            },
            vec![UiNode::new(ColumnProps, Modifiers::new(), vec![])],
        );
    }

    #[test]
    fn single_row() {
        immediate_test(
            |ui| {
                ui.row(|_, _| {});
            },
            vec![UiNode::new(RowProps, Modifiers::new(), vec![])],
        );
    }

    #[test]
    fn nested_block_column_row() {
        immediate_test(
            |ui| {
                ui.block(|ui, _| {
                    ui.column(|ui, _| {
                        ui.row(|_, _| {});
                    });
                });
            },
            vec![UiNode::new(
                BlockProps,
                Modifiers::new(),
                vec![UiNode::new(
                    ColumnProps,
                    Modifiers::new(),
                    vec![UiNode::new(RowProps, Modifiers::new(), vec![])],
                )],
            )],
        );
    }

    #[test]
    fn single_text() {
        immediate_test(
            |ui| {
                ui.text("Hello", |_, _, _| {});
            },
            vec![UiNode::new(
                TextProps {
                    text: "Hello".into(),
                    text_color: Color::new(1.0, 1.0, 1.0, 1.0),
                    font_family: String::new(),
                    font_size: DEFAULT_FONT_SIZE,
                    line_height: DEFAULT_LINE_HEIGHT,
                    cursor_position: None,
                },
                Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .max_width(Extent::FillParent)
                    .clone(),
                vec![],
            )],
        );
    }
}
