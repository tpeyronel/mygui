use std::time::Instant;

use crate::{
    font::font_engine::TextPosition,
    input::{TextCommand, TextEvent},
    ui::{border_radius::BorderRadius, border_thickness::BorderThickness, node::text::TextProps, Extent, Modifiers},
    vertex::Color,
};

use super::{context::NodeInputEvent, ui::Ui};

const CURSOR_PERIOD_MILLIS: u64 = 1000;

pub trait BaseTextField {
    fn base_text_field(
        &mut self,
        text: impl Into<String>,
        on_text_change: impl FnOnce(String),
        f: impl FnOnce(&mut Ui, &mut TextProps, &mut Modifiers),
    );
}

impl BaseTextField for Ui<'_> {
    fn base_text_field(
        &mut self,
        text: impl Into<String>,
        on_text_change: impl FnOnce(String),
        f: impl FnOnce(&mut Ui, &mut TextProps, &mut Modifiers),
    ) {
        let ui = self;

        ui.text("", |ui, props, modifiers| {
            let input = ui.use_input();

            let internal_state = ui.use_ref(|| BaseTextFieldInternalState {
                cursor_blink_start: Instant::now(),
                cursor_index: 0,
            });
            let mut internal_state = internal_state.borrow_mut();

            props.text = text.into();
            props.font_family = "times new roman".into();
            if input.is_focused() && internal_state.is_cursor_visible() {
                props.cursor_position = Some(TextPosition::Index(internal_state.cursor_index));
            }

            modifiers
                .min_width(Extent::Px(64.0))
                .fill_color(if input.is_pressed() {
                    Color::new(1.0, 1.0, 1.0, 0.5)
                } else if input.is_hovered() {
                    Color::new(1.0, 1.0, 1.0, 0.25)
                } else {
                    Color::new(0.0, 0.0, 0.0, 0.5)
                })
                .border_radius(BorderRadius::all(8.0))
                .border_thickness(BorderThickness::all(2.0));

            if input.is_focused() {
                modifiers.border_color(Color::new(1.0, 0.0, 0.0, 1.0));
            }

            if input.on_release() {
                internal_state.cursor_blink_start = Instant::now();
            }

            process_input_events(&props.text, on_text_change, &mut internal_state, input.events());

            f(ui, props, modifiers);
        });
    }
}

fn process_input_events(
    text: &str,
    on_text_change: impl FnOnce(String),
    internal_state: &mut BaseTextFieldInternalState,
    events: &[NodeInputEvent],
) {
    let old_text = text;
    let mut text = text.to_string();
    let mut cursor_index = internal_state.cursor_index;

    let text_events_iter = events.iter().filter_map(|e| match e {
        NodeInputEvent::TextEvent(text_event) => Some(text_event),
        _ => None,
    });

    for text_event in text_events_iter {
        match text_event {
            TextEvent::TextInput(text_input) => {
                let (left, right) = text.split_at(cursor_index as usize);
                text = left.to_string() + text_input + right;
                cursor_index += text_input.len() as u32;
            }
            TextEvent::TextCommand(command) => match command {
                TextCommand::ArrowRight => {
                    let Some(char_at_position) = text[cursor_index as usize..].chars().next() else {
                        continue;
                    };

                    cursor_index += char_at_position.len_utf8() as u32;
                }
                TextCommand::ArrowUp => {}
                TextCommand::ArrowLeft => {
                    let (left, _) = text.split_at(cursor_index as usize);

                    cursor_index -= left.chars().next_back().map(|c| c.len_utf8() as u32).unwrap_or(0);
                }
                TextCommand::ArrowDown => {
                    // let (_, right) = new_text.split_at(new_cursor_position as usize);
                    // let original_text_coords =
                    //     TextPosition::Index(new_cursor_position).to_coords(&text);
                    // let target_text_coords = TextPositionCoords {
                    //     line: original_text_coords.line + 1,
                    //     column: original_text_coords.column,
                    // };
                    // let mut text_coords =
                    //     TextPosition::Index(new_cursor_position).to_coords(&text);

                    // for c in right.chars() {
                    //     if text_coords.line > target_text_coords.line
                    //         || (text_coords.line == target_text_coords.line
                    //             && text_coords.column >= target_text_coords.column)
                    //     {
                    //         break;
                    //     }
                    // }
                }
                TextCommand::Backspace => {
                    let (left, right) = text.split_at(cursor_index as usize);
                    let Some((last_left_char_index, last_left_char)) = left.char_indices().next_back() else {
                        continue;
                    };

                    text = left[..last_left_char_index].to_string() + right;
                    cursor_index -= last_left_char.len_utf8() as u32;
                }
            },
        }
    }

    if text != old_text {
        on_text_change(text);
    }

    // TODO: text position may change without requiring index to change if the text changes.
    if internal_state.cursor_index != cursor_index {
        internal_state.reset_cursor_blink();
        internal_state.cursor_index = cursor_index;
    }
}

struct BaseTextFieldInternalState {
    cursor_blink_start: Instant,
    cursor_index: u32,
}

impl BaseTextFieldInternalState {
    fn reset_cursor_blink(&mut self) {
        self.cursor_blink_start = Instant::now();
    }

    fn is_cursor_visible(&self) -> bool {
        (self.cursor_blink_start.elapsed().as_millis() as u64 % CURSOR_PERIOD_MILLIS) < (CURSOR_PERIOD_MILLIS / 2)
    }
}
