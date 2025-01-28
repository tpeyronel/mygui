use crate::{
    font::mock_font_engine::MockFontEngine,
    mesh::mesh_manager::MeshManager,
    ui::{create_mock_hash_tree_rec, Alignment},
};

use super::*;
use glam::Vec2;
use pretty_assertions::assert_eq;

fn compute_layout_nodes(position: Vec2, size: Vec2, ui: UiNode) -> Vec<LayoutNode> {
    let root_ui_nodes = vec![ui];
    let root_hash_nodes = create_mock_hash_tree_rec(&root_ui_nodes);

    let scale_factor = 1.0;
    let mut mesh_manager = MeshManager::new();
    let mut font_engine: Box<dyn FontEngine> = Box::new(MockFontEngine::new());
    let mut draw_data = vec![];
    let mut bounding_boxes = vec![];

    let root_layout_node = UiNodeProcessor::compute_layout_tree(
        root_ui_nodes,
        root_hash_nodes,
        position,
        size,
        scale_factor,
        &mut mesh_manager,
        &mut font_engine,
        &mut draw_data,
        &mut bounding_boxes,
    );

    root_layout_node.children
}

fn test_layout(width: f32, height: f32, ui: UiNode, expected: LayoutNode) {
    let layout_nodes = compute_layout_nodes(Vec2::ZERO, Vec2::new(width, height), ui);
    assert_eq!(&[expected], &layout_nodes.as_slice());
}

#[test]
fn default_block() {
    test_layout(
        32.0,
        32.0,
        UiNode::new(BlockProps, Modifiers::new(), vec![]),
        LayoutNode {
            layout: Layout {
                margin_position: Vec2::new(0.0, 0.0),
                margin_size: Vec2::new(32.0, 32.0),
                children_boundary_size: Vec2::new(32.0, 32.0),
                ..Default::default()
            },
            children: vec![],
        },
    )
}

#[test]
fn padding() {
    test_layout(
        32.0,
        32.0,
        UiNode::new(
            BlockProps,
            Modifiers::new().padding(Padding::all(8.px())).clone(),
            vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
        ),
        LayoutNode {
            layout: Layout {
                margin_position: Vec2::new(0.0, 0.0),
                margin_size: Vec2::new(32.0, 32.0),
                children_boundary_size: Vec2::new(16.0, 16.0),
                padding: Inset::all(8.0),
                ..Default::default()
            },
            children: vec![LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(8.0, 8.0),
                    margin_size: Vec2::new(16.0, 16.0),
                    children_boundary_size: Vec2::new(16.0, 16.0),
                    ..Default::default()
                },
                children: vec![],
            }],
        },
    )
}

#[test]
fn margin() {
    test_layout(
        32.0,
        32.0,
        UiNode::new(BlockProps, Modifiers::new().margin(Margin::all(8.px())).clone(), vec![]),
        LayoutNode {
            layout: Layout {
                margin_position: Vec2::new(0.0, 0.0),
                margin_size: Vec2::new(32.0, 32.0),
                children_boundary_size: Vec2::new(16.0, 16.0),
                margin: Inset::all(8.0),
                ..Default::default()
            },
            children: vec![],
        },
    )
}

#[test]
fn full_padding() {
    test_layout(
        32.0,
        32.0,
        UiNode::new(
            BlockProps,
            Modifiers::new().padding(Padding::all(16.px())).clone(),
            vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
        ),
        LayoutNode {
            layout: Layout {
                margin_position: Vec2::new(0.0, 0.0),
                margin_size: Vec2::new(32.0, 32.0),
                children_boundary_size: Vec2::new(0.0, 0.0),
                padding: Inset::all(16.0),
                ..Default::default()
            },
            children: vec![LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(16.0, 16.0),
                    margin_size: Vec2::new(0.0, 0.0),
                    children_boundary_size: Vec2::new(0.0, 0.0),
                    ..Default::default()
                },
                children: vec![],
            }],
        },
    )
}

#[test]
fn too_much_padding() {
    // TODO: restore this test.
    // test_converter(
    //     32.0,
    //     32.0,
    //     UiNode::new(
    //         BlockProps,
    //         Modifiers::new().padding(Padding::all(24.0)).clone(),
    //         vec![UiNode::new(
    //             BlockProps,
    //             Modifiers::new(),
    //             vec![],
    //         )],
    //     ),
    //     &[
    //         DrawElement::Rectangle {
    //             bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
    //             fill_color: Color::ZERO,
    //             border_color: Color::ZERO,
    //             corner_radius: Vec4::ZERO,
    //             border_width: Vec4::ZERO,
    //         },
    //         DrawElement::Rectangle {
    //             bounds: Rectangle::from_position_size(Vec2::new(16.0, 16.0), Vec2::new(0.0, 0.0)),
    //             fill_color: Color::rgba(1.0, 0.0, 0.0, 1.0),
    //             border_color: Color::ZERO,
    //             corner_radius: Vec4::ZERO,
    //             border_width: Vec4::ZERO,
    //         },
    //     ],
    // )
}

#[test]
fn self_alignment_basic() {
    fn test_self_alignment_basic(alignment: Alignment, expected_margin_position: Vec2) {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(8.px())
                    .height(8.px())
                    .self_alignment(alignment)
                    .clone(),
                vec![],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: expected_margin_position,
                    margin_size: Vec2::new(8.0, 8.0),
                    children_boundary_size: Vec2::new(8.0, 8.0),
                    ..Default::default()
                },
                children: vec![],
            },
        )
    }

    test_self_alignment_basic(Alignment::Center, Vec2::new(12.0, 12.0));
    test_self_alignment_basic(Alignment::Right, Vec2::new(24.0, 12.0));
    test_self_alignment_basic(Alignment::TopRight, Vec2::new(24.0, 24.0));
    test_self_alignment_basic(Alignment::Top, Vec2::new(12.0, 24.0));
    test_self_alignment_basic(Alignment::TopLeft, Vec2::new(0.0, 24.0));
    test_self_alignment_basic(Alignment::Left, Vec2::new(0.0, 12.0));
    test_self_alignment_basic(Alignment::BottomLeft, Vec2::new(0.0, 0.0));
    test_self_alignment_basic(Alignment::Bottom, Vec2::new(12.0, 0.0));
    test_self_alignment_basic(Alignment::BottomRight, Vec2::new(24.0, 0.0));
}

#[test]
fn subpixel_alignment() {
    test_layout(
        15.0,
        15.0,
        UiNode::new(
            BlockProps,
            Modifiers::new().width(8.px()).height(8.px()).clone(),
            vec![],
        ),
        LayoutNode {
            layout: Layout {
                margin_position: Vec2::new(4.0, 4.0),
                margin_size: Vec2::new(8.0, 8.0),
                children_boundary_size: Vec2::new(8.0, 8.0),
                ..Default::default()
            },
            children: vec![],
        },
    );

    test_layout(
        17.0,
        17.0,
        // Duplicate ui (.clone() not available)
        UiNode::new(
            BlockProps,
            Modifiers::new().width(8.px()).height(8.px()).clone(),
            vec![],
        ),
        LayoutNode {
            layout: Layout {
                margin_position: Vec2::new(5.0, 5.0),
                margin_size: Vec2::new(8.0, 8.0),
                children_boundary_size: Vec2::new(8.0, 8.0),
                ..Default::default()
            },
            children: vec![],
        },
    );
}

#[test]
fn self_alignment_with_parent_border_thickness() {
    fn test_self_alignment_basic(alignment: Alignment, expected_margin_position: Vec2) {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                // Border thickness of 4.0 makes the parent container equivalent to a 24.0 size container.
                Modifiers::new().border_thickness(BorderThickness::all(4.px())).clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(8.px())
                        .height(8.px())
                        .self_alignment(alignment)
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(24.0, 24.0),
                    border_thickness: Inset::all(4.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: expected_margin_position,
                        margin_size: Vec2::new(8.0, 8.0),
                        children_boundary_size: Vec2::new(8.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    test_self_alignment_basic(Alignment::Center, Vec2::new(12.0, 12.0));
    test_self_alignment_basic(Alignment::Right, Vec2::new(20.0, 12.0));
    test_self_alignment_basic(Alignment::TopRight, Vec2::new(20.0, 20.0));
    test_self_alignment_basic(Alignment::Top, Vec2::new(12.0, 20.0));
    test_self_alignment_basic(Alignment::TopLeft, Vec2::new(4.0, 20.0));
    test_self_alignment_basic(Alignment::Left, Vec2::new(4.0, 12.0));
    test_self_alignment_basic(Alignment::BottomLeft, Vec2::new(4.0, 4.0));
    test_self_alignment_basic(Alignment::Bottom, Vec2::new(12.0, 4.0));
    test_self_alignment_basic(Alignment::BottomRight, Vec2::new(20.0, 4.0));
}

mod blocks {
    use super::*;

    #[test]
    fn block_different_padding_values() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .padding(Padding::new(8.px(), 1.px(), 2.px(), 4.px()))
                    .clone(),
                vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(32.0 - 10.0, 32.0 - 5.0),
                    padding: Inset::new(8.0, 1.0, 2.0, 4.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(8.0, 1.0),
                        margin_size: Vec2::new(32.0 - 10.0, 32.0 - 5.0),
                        children_boundary_size: Vec2::new(32.0 - 10.0, 32.0 - 5.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn block_different_border_thickness_values() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .border_thickness(BorderThickness::new(1.px(), 2.px(), 4.px(), 8.px()))
                    .clone(),
                vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(32.0 - 5.0, 32.0 - 10.0),
                    border_thickness: Inset::new(1.0, 2.0, 4.0, 8.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(1.0, 2.0),
                        margin_size: Vec2::new(32.0 - 5.0, 32.0 - 10.0),
                        children_boundary_size: Vec2::new(32.0 - 5.0, 32.0 - 10.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn block_fit_content_with_fill_parent_child() {
        test_layout(
            128.0,
            128.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(8.px())
                            .height(64.px())
                            .self_alignment(Alignment::BottomLeft)
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(96.px())
                            .height(8.px())
                            .self_alignment(Alignment::TopRight)
                            .clone(),
                        vec![],
                    ),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(96.0, 64.0),
                    children_boundary_size: Vec2::new(96.0, 64.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(96.0, 64.0),
                            children_boundary_size: Vec2::new(96.0, 64.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(8.0, 64.0),
                            children_boundary_size: Vec2::new(8.0, 64.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 64.0 - 8.0),
                            margin_size: Vec2::new(96.0, 8.0),
                            children_boundary_size: Vec2::new(96.0, 8.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }

    #[test]
    fn block_fit_content_with_fill_parent_child_all_children_with_borders() {
        /* In this case, children having border should not affect in any way the parent size
        (aside from leaf layouts having different children boundary size and border thickness). */
        test_layout(
            128.0,
            128.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .border_thickness(BorderThickness::all(8.px()))
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(8.px())
                            .height(64.px())
                            .border_thickness(BorderThickness::all(8.px()))
                            .self_alignment(Alignment::BottomLeft)
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(96.px())
                            .height(8.px())
                            .border_thickness(BorderThickness::all(8.px()))
                            .self_alignment(Alignment::TopRight)
                            .clone(),
                        vec![],
                    ),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(96.0, 64.0),
                    children_boundary_size: Vec2::new(96.0, 64.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(96.0, 64.0),
                            children_boundary_size: Vec2::new(96.0 - 16.0, 64.0 - 16.0),
                            border_thickness: Inset::all(8.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(8.0, 64.0),
                            children_boundary_size: Vec2::new(0.0, 64.0 - 16.0),
                            border_thickness: Inset::all(8.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 64.0 - 8.0),
                            margin_size: Vec2::new(96.0, 8.0),
                            children_boundary_size: Vec2::new(96.0 - 16.0, 0.0),
                            border_thickness: Inset::all(8.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }

    #[test]
    fn block_fit_content_with_fill_parent_child_parent_with_border() {
        test_layout(
            128.0,
            128.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .border_thickness(BorderThickness::all(8.px()))
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(8.px())
                            .height(64.px())
                            .self_alignment(Alignment::BottomLeft)
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(96.px())
                            .height(8.px())
                            .self_alignment(Alignment::TopRight)
                            .clone(),
                        vec![],
                    ),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(96.0 + 16.0, 64.0 + 16.0),
                    children_boundary_size: Vec2::new(96.0, 64.0),
                    border_thickness: Inset::all(8.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(8.0, 8.0),
                            margin_size: Vec2::new(96.0, 64.0),
                            children_boundary_size: Vec2::new(96.0, 64.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(8.0, 8.0),
                            margin_size: Vec2::new(8.0, 64.0),
                            children_boundary_size: Vec2::new(8.0, 64.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(8.0, 64.0),
                            margin_size: Vec2::new(96.0, 8.0),
                            children_boundary_size: Vec2::new(96.0, 8.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }

    #[test]
    fn block_fit_content_with_fill_parent_child_parent_with_padding() {
        /* Should be almost functionally equivalent to block_fit_content_with_fill_parent_child_parent_with_border */
        test_layout(
            128.0,
            128.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .padding(Padding::all(8.px()))
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(8.px())
                            .height(64.px())
                            .self_alignment(Alignment::BottomLeft)
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(96.px())
                            .height(8.px())
                            .self_alignment(Alignment::TopRight)
                            .clone(),
                        vec![],
                    ),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(96.0 + 16.0, 64.0 + 16.0),
                    children_boundary_size: Vec2::new(96.0, 64.0),
                    padding: Inset::all(8.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(8.0, 8.0),
                            margin_size: Vec2::new(96.0, 64.0),
                            children_boundary_size: Vec2::new(96.0, 64.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(8.0, 8.0),
                            margin_size: Vec2::new(8.0, 64.0),
                            children_boundary_size: Vec2::new(8.0, 64.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(8.0, 64.0),
                            margin_size: Vec2::new(96.0, 8.0),
                            children_boundary_size: Vec2::new(96.0, 8.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }
}

mod columns {
    use crate::ui::node::column::ColumnProps;

    use super::*;

    #[test]
    fn basic_column() {
        test_layout(
            32.0,
            128.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new(),
                vec![
                    UiNode::new(BlockProps, Modifiers::new().height(24.px()).clone(), vec![]),
                    UiNode::new(BlockProps, Modifiers::new().height(48.px()).clone(), vec![]),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 128.0),
                    children_boundary_size: Vec2::new(32.0, 128.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 128.0 - 24.0),
                            margin_size: Vec2::new(32.0, 24.0),
                            children_boundary_size: Vec2::new(32.0, 24.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                            margin_size: Vec2::new(32.0, 48.0),
                            children_boundary_size: Vec2::new(32.0, 48.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }

    #[test]
    fn basic_column_child_with_padding() {
        test_layout(
            32.0,
            128.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new(),
                vec![
                    UiNode::new(
                        BlockProps,
                        Modifiers::new().height(24.px()).padding(Padding::all(2.px())).clone(),
                        vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
                    ),
                    UiNode::new(BlockProps, Modifiers::new().height(48.px()).clone(), vec![]),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 128.0),
                    children_boundary_size: Vec2::new(32.0, 128.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 128.0 - 24.0),
                            margin_size: Vec2::new(32.0, 24.0),
                            children_boundary_size: Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                            padding: Inset::all(2.0),
                            ..Default::default()
                        },
                        children: vec![LayoutNode {
                            layout: Layout {
                                margin_position: Vec2::new(2.0, 128.0 - 24.0 + 2.0),
                                margin_size: Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                                children_boundary_size: Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                                ..Default::default()
                            },
                            children: vec![],
                        }],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                            margin_size: Vec2::new(32.0, 48.0),
                            children_boundary_size: Vec2::new(32.0, 48.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }

    #[test]
    fn basic_column_single_child_with_margin() {
        test_layout(
            32.0,
            128.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new().height(16.px()).margin(Margin::all(2.px())).clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(32.0, 128.0),
                    children_boundary_size: Vec2::new(32.0, 128.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 128.0 - (16.0 + 4.0)),
                        margin_size: Vec2::new(32.0, 16.0 + 4.0),
                        children_boundary_size: Vec2::new(32.0 - 4.0, 16.0),
                        margin: Inset::all(2.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        );
    }

    #[test]
    fn column_both_fit_content_with_fill_parent_child() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new().width(64.px()).height(32.px()).clone(),
                        vec![],
                    ),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(64.0, 64.0),
                    children_boundary_size: Vec2::new(64.0, 32.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 32.0),
                            margin_size: Vec2::new(64.0, 32.0),
                            children_boundary_size: Vec2::new(64.0, 32.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(64.0, 32.0),
                            children_boundary_size: Vec2::new(64.0, 32.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }

    #[test]
    fn column_fit_content_padding() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(Extent::fill_parent())
                    .height(Extent::FitContent)
                    .padding(Padding::all(8.px()))
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new().height(32.px()).clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::new(0.0, 0.0),
                    margin_size: Vec2::new(256.0, 32.0 + 16.0),
                    children_boundary_size: Vec2::new(256.0 - 16.0, 32.0),
                    padding: Inset::all(8.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(8.0, 8.0),
                        margin_size: Vec2::new(256.0 - 16.0, 32.0),
                        children_boundary_size: Vec2::new(256.0 - 16.0, 32.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        );
    }

    #[test]
    fn column_fit_content_single_child_fill_parent() {
        test_layout(
            32.0,
            128.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .self_alignment(Alignment::BottomLeft)
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::fill_parent())
                        .height(Extent::fill_parent())
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::ZERO,
                    children_boundary_size: Vec2::ZERO,
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::ZERO,
                        children_boundary_size: Vec2::ZERO,
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        );
    }

    #[test]
    fn column_fit_content_single_child_fill_parent_with_margin() {
        test_layout(
            32.0,
            128.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .self_alignment(Alignment::BottomLeft)
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(Extent::fill_parent())
                        .height(Extent::fill_parent())
                        .margin(Margin::all(4.px()))
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::ZERO,
                    children_boundary_size: Vec2::ZERO,
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::ZERO,
                        children_boundary_size: Vec2::ZERO,
                        margin: Inset::all(4.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        );
    }

    #[test]
    fn column_fit_content_hor_child_fill_parent() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .self_alignment(Alignment::BottomLeft)
                    .width(Extent::FitContent)
                    .height(48.px())
                    .clone(),
                vec![
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new().width(64.px()).height(Extent::fill_parent()).clone(),
                        vec![],
                    ),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(64.0, 48.0),
                    children_boundary_size: Vec2::new(64.0, 48.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(64.0, 48.0),
                            children_boundary_size: Vec2::new(64.0, 48.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, -48.0),
                            margin_size: Vec2::new(64.0, 48.0),
                            children_boundary_size: Vec2::new(64.0, 48.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }
}

mod rows {
    use crate::ui::node::row::RowProps;

    use super::*;

    #[test]
    fn row_both_fit_content_with_fill_parent_child() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                RowProps,
                Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![],
                    ),
                    UiNode::new(
                        BlockProps,
                        Modifiers::new().width(64.px()).height(32.px()).clone(),
                        vec![],
                    ),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(128.0, 32.0),
                    children_boundary_size: Vec2::new(64.0, 32.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(64.0, 32.0),
                            children_boundary_size: Vec2::new(64.0, 32.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(64.0, 0.0),
                            margin_size: Vec2::new(64.0, 32.0),
                            children_boundary_size: Vec2::new(64.0, 32.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        );
    }
}

mod weight {
    use crate::ui::node::{column::ColumnProps, row::RowProps};

    use super::*;

    #[test]
    fn column_weight() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(50.px())
                    .height(100.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(BlockProps, Modifiers::new().height(20.px()).weight(1.0).clone(), vec![]),
                    UiNode::new(BlockProps, Modifiers::new().height(40.px()).weight(1.0).clone(), vec![]),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(50.0, 100.0),
                    children_boundary_size: Vec2::new(50.0, 100.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 60.0),
                            margin_size: Vec2::new(50.0, 40.0),
                            children_boundary_size: Vec2::new(50.0, 40.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(50.0, 60.0),
                            children_boundary_size: Vec2::new(50.0, 60.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        )
    }

    #[test]
    fn row_weight() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                RowProps,
                Modifiers::new()
                    .width(100.px())
                    .height(50.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(BlockProps, Modifiers::new().width(20.px()).weight(1.0).clone(), vec![]),
                    UiNode::new(BlockProps, Modifiers::new().width(40.px()).weight(1.0).clone(), vec![]),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(100.0, 50.0),
                    children_boundary_size: Vec2::new(100.0, 50.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 00.0),
                            margin_size: Vec2::new(40.0, 50.0),
                            children_boundary_size: Vec2::new(40.0, 50.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(40.0, 0.0),
                            margin_size: Vec2::new(60.0, 50.0),
                            children_boundary_size: Vec2::new(60.0, 50.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        )
    }

    #[test]
    fn column_weight_rounding() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(50.px())
                    .height(100.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![
                    UiNode::new(BlockProps, Modifiers::new().height(0.px()).weight(1.0).clone(), vec![]),
                    UiNode::new(BlockProps, Modifiers::new().height(0.px()).weight(1.0).clone(), vec![]),
                    UiNode::new(BlockProps, Modifiers::new().height(0.px()).weight(1.0).clone(), vec![]),
                ],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(50.0, 100.0),
                    children_boundary_size: Vec2::new(50.0, 100.0),
                    ..Default::default()
                },
                children: vec![
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 100.0 - 34.0),
                            margin_size: Vec2::new(50.0, 34.0),
                            children_boundary_size: Vec2::new(50.0, 34.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 100.0 - 34.0 - 33.0),
                            margin_size: Vec2::new(50.0, 33.0),
                            children_boundary_size: Vec2::new(50.0, 33.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(0.0, 0.0),
                            margin_size: Vec2::new(50.0, 33.0),
                            children_boundary_size: Vec2::new(50.0, 33.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            },
        )
    }

    #[test]
    fn column_weight_respects_margin() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(50.px())
                    .height(8.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .height(0.px())
                        .weight(1.0)
                        .margin(Margin::all(4.px())) // This margin should only allow for a height of 0.
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(50.0, 8.0),
                    children_boundary_size: Vec2::new(50.0, 8.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(50.0, 8.0),
                        children_boundary_size: Vec2::new(50.0 - 8.0, 0.0),
                        margin: Inset::all(4.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn column_weight_respects_border_thickness() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(50.px())
                    .height(8.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .height(0.px())
                        .weight(1.0)
                        .border_thickness(BorderThickness::all(3.px())) // This border thickness should only allow for a height of the child of 2.0.
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![],
                    )],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(50.0, 8.0),
                    children_boundary_size: Vec2::new(50.0, 8.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(50.0, 8.0),
                        children_boundary_size: Vec2::new(50.0 - 6.0, 8.0 - 6.0),
                        border_thickness: Inset::all(3.0),
                        ..Default::default()
                    },
                    children: vec![LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(3.0, 3.0),
                            margin_size: Vec2::new(44.0, 2.0),
                            children_boundary_size: Vec2::new(44.0, 2.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                }],
            },
        )
    }

    #[test]
    fn column_weight_respects_padding() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(50.px())
                    .height(8.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .height(0.px())
                        .weight(1.0)
                        .padding(Padding::all(3.px())) // This padding should only allow for a height of the child of 2.0.
                        .clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![],
                    )],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(50.0, 8.0),
                    children_boundary_size: Vec2::new(50.0, 8.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(0.0, 0.0),
                        margin_size: Vec2::new(50.0, 8.0),
                        children_boundary_size: Vec2::new(50.0 - 6.0, 8.0 - 6.0),
                        padding: Inset::all(3.0),
                        ..Default::default()
                    },
                    children: vec![LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::new(3.0, 3.0),
                            margin_size: Vec2::new(44.0, 2.0),
                            children_boundary_size: Vec2::new(44.0, 2.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }],
                }],
            },
        )
    }

    #[test]
    fn column_weight_transfers_to_nested_children_correctly() {
        test_layout(
            256.0,
            256.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(50.px())
                    .height(8.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new().height(0.px()).weight(1.0).clone(),
                    vec![UiNode::new(
                        BlockProps,
                        Modifiers::new()
                            .width(Extent::fill_parent())
                            .height(Extent::fill_parent())
                            .clone(),
                        vec![UiNode::new(
                            BlockProps,
                            Modifiers::new()
                                .width(Extent::fill_parent())
                                .height(Extent::fill_parent())
                                .clone(),
                            vec![],
                        )],
                    )],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(50.0, 8.0),
                    children_boundary_size: Vec2::new(50.0, 8.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(50.0, 8.0),
                        children_boundary_size: Vec2::new(50.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![LayoutNode {
                        layout: Layout {
                            margin_position: Vec2::ZERO,
                            margin_size: Vec2::new(50.0, 8.0),
                            children_boundary_size: Vec2::new(50.0, 8.0),
                            ..Default::default()
                        },
                        children: vec![LayoutNode {
                            layout: Layout {
                                margin_position: Vec2::ZERO,
                                margin_size: Vec2::new(50.0, 8.0),
                                children_boundary_size: Vec2::new(50.0, 8.0),
                                ..Default::default()
                            },
                            children: vec![],
                        }],
                    }],
                }],
            },
        )
    }
}

mod rounding {
    use crate::ui::node::column::ColumnProps;

    use super::*;

    #[test]
    fn border_thickness_rounding() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .border_thickness(BorderThickness::all(3.5.px()))
                    .clone(), // Should all be rounded to 4.0
                vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(24.0, 24.0),
                    border_thickness: Inset::all(4.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(4.0, 4.0),
                        margin_size: Vec2::new(24.0, 24.0),
                        children_boundary_size: Vec2::new(24.0, 24.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn padding_rounding() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new().padding(Padding::all(3.5.px())).clone(), // Should all be rounded to 4.0
                vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(24.0, 24.0),
                    padding: Inset::all(4.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(4.0, 4.0),
                        margin_size: Vec2::new(24.0, 24.0),
                        children_boundary_size: Vec2::new(24.0, 24.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn margin_rounding() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new().margin(Margin::all(3.5.px())).clone(), // Should all be rounded to 4.0
                vec![UiNode::new(BlockProps, Modifiers::new(), vec![])],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(32.0, 32.0),
                    children_boundary_size: Vec2::new(24.0, 24.0),
                    margin: Inset::all(4.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(4.0, 4.0),
                        margin_size: Vec2::new(24.0, 24.0),
                        children_boundary_size: Vec2::new(24.0, 24.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn root_rounding() {
        /* This test checks that the position and the size passed to ui.to_draw_data() get correctly rounded. */
        let root_position = Vec2::splat(0.5); // Should be rounded to (1.0, 1.0)
        let root_size = Vec2::splat(31.5); // Should be rounded to (32.0, 32.0)

        let ui = UiNode::new(BlockProps, Modifiers::new(), vec![]);

        let draw_data = compute_layout_nodes(root_position, root_size, ui);

        let expected = vec![LayoutNode {
            layout: Layout {
                margin_position: Vec2::new(1.0, 1.0),
                margin_size: Vec2::new(32.0, 32.0),
                children_boundary_size: Vec2::new(32.0, 32.0),
                ..Default::default()
            },
            children: vec![],
        }];

        pretty_assertions::assert_eq!(&expected, &draw_data);
    }

    #[test]
    fn column_horizontal_rounding() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                ColumnProps,
                Modifiers::new()
                    .width(9.px())
                    .height(8.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new().width(8.px()).self_alignment(Alignment::Center).clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(9.0, 8.0),
                    children_boundary_size: Vec2::new(9.0, 8.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::new(1.0, 0.0),
                        margin_size: Vec2::new(8.0, 8.0),
                        children_boundary_size: Vec2::new(8.0, 8.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }
}

mod text {
    use glam::Vec2;

    use crate::ui::node::text::TextProps;

    use super::{test_layout, Alignment, Color, Extent, Layout, LayoutNode, Modifiers, UiNode};

    #[test]
    fn text_fit_content() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                TextProps {
                    text: "abcdef".into(),
                    text_color: Color::WHITE,
                    font_family: String::new(),
                    font_size: 13.0,
                    line_height: 16.0,
                    cursor_position: None,
                },
                Modifiers::new()
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(13.0 * 6.0, 16.0),
                    children_boundary_size: Vec2::ZERO,
                    ..Default::default()
                },
                children: vec![],
            },
        )
    }

    #[test]
    fn text_fit_content_with_max_width() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                TextProps {
                    text: "abcdef".into(),
                    text_color: Color::WHITE,
                    font_family: String::new(),
                    font_size: 13.0,
                    line_height: 16.0,
                    cursor_position: None,
                },
                Modifiers::new()
                    .width(Extent::FitContent)
                    .max_width(Extent::fill_parent())
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(32.0, 16.0 * 3.0),
                    children_boundary_size: Vec2::ZERO,
                    ..Default::default()
                },
                children: vec![],
            },
        )
    }
}

mod row_advanced {
    use glam::Vec2;

    use crate::ui::node::{row::RowProps, text::TextProps};

    use super::{test_layout, Alignment, Color, Extent, ExtentExt, Layout, LayoutNode, Modifiers, UiNode};

    /// This test checks that if a row child has non-zero weight, then
    /// when weight is applied, the height of the element is recomputed
    /// (and the height of the row itself too, as it is FitContent).
    /// In this case, if this were not the case, then because the initial
    /// size of the text is Px(0.0), the initially computed height of the text
    /// would be very big, as it would try to spread it vertically. But because
    /// we use weight(1.0), it should be equivalent to having specified the size
    /// of the text to be Px(32.0) / fill_parent().
    #[test]
    fn row_with_text_extent_0_weight_1() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                RowProps,
                Modifiers::new()
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    TextProps {
                        text: "abcdef".into(),
                        text_color: Color::WHITE,
                        font_family: String::new(),
                        font_size: 13.0,
                        line_height: 16.0,
                        cursor_position: None,
                    },
                    Modifiers::new()
                        .width(0.px())
                        .weight(1.0)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(32.0, 16.0 * 3.0),
                    children_boundary_size: Vec2::new(32.0, 48.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(32.0, 16.0 * 3.0),
                        children_boundary_size: Vec2::ZERO,
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }
}

mod extent {
    use glam::Vec2;

    use crate::ui::Alignment;

    use super::{test_layout, BlockProps, ExtentExt, Layout, LayoutNode, Modifiers, UiNode};

    #[test]
    fn extent_parent_100_percent() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(100.px())
                    .height(100.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(100.percent())
                        .height(100.percent())
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(100.0, 100.0),
                    children_boundary_size: Vec2::new(100.0, 100.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(100.0, 100.0),
                        children_boundary_size: Vec2::new(100.0, 100.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn extent_parent_50_percent() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(100.px())
                    .height(100.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width(50.percent())
                        .height(50.percent())
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(100.0, 100.0),
                    children_boundary_size: Vec2::new(100.0, 100.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(50.0, 50.0),
                        children_boundary_size: Vec2::new(50.0, 50.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn extent_parent_rounding() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(100.px())
                    .height(100.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width((100.0 / 3.0).percent())
                        .height((100.0 / 3.0).percent())
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(100.0, 100.0),
                    children_boundary_size: Vec2::new(100.0, 100.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(33.0, 33.0),
                        children_boundary_size: Vec2::new(33.0, 33.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn extent_parent_negative() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width(100.px())
                    .height(100.px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![UiNode::new(
                    BlockProps,
                    Modifiers::new()
                        .width((-100.0).percent()) // Should get clamped to 0.0
                        .height((-100.0).percent()) // Should get clamped to 0.0
                        .self_alignment(Alignment::BottomLeft)
                        .clone(),
                    vec![],
                )],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(100.0, 100.0),
                    children_boundary_size: Vec2::new(100.0, 100.0),
                    ..Default::default()
                },
                children: vec![LayoutNode {
                    layout: Layout {
                        margin_position: Vec2::ZERO,
                        margin_size: Vec2::new(0.0, 0.0),
                        children_boundary_size: Vec2::new(0.0, 0.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            },
        )
    }

    #[test]
    fn extent_px_negative() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width((-100).px())
                    .height((-100).px())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(0.0, 0.0),
                    children_boundary_size: Vec2::new(0.0, 0.0),
                    ..Default::default()
                },
                children: vec![],
            },
        )
    }

    #[test]
    fn extent_dp_negative() {
        test_layout(
            32.0,
            32.0,
            UiNode::new(
                BlockProps,
                Modifiers::new()
                    .width((-100).dp())
                    .height((-100).dp())
                    .self_alignment(Alignment::BottomLeft)
                    .clone(),
                vec![],
            ),
            LayoutNode {
                layout: Layout {
                    margin_position: Vec2::ZERO,
                    margin_size: Vec2::new(0.0, 0.0),
                    children_boundary_size: Vec2::new(0.0, 0.0),
                    ..Default::default()
                },
                children: vec![],
            },
        )
    }
}
