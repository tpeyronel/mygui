use std::time::Instant;

use crate::{
    input::{TextCommand, TextEvent},
    text::text_position::TextPosition,
    ui::{node::text::TextProps, Modifiers},
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
            if input.is_focused() && internal_state.is_cursor_visible() {
                props.cursor_position = Some(TextPosition::from_text_index(&props.text, internal_state.cursor_index));
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

    let mut shuold_reset_cursor_blink = false;

    for text_event in text_events_iter {
        shuold_reset_cursor_blink = true;

        match text_event {
            TextEvent::TextInput(text_input) => {
                let (left, right) = text.split_at(cursor_index);
                text = left.to_string() + text_input + right;
                cursor_index += text_input.len();
            }
            TextEvent::TextCommand(command) => match command {
                TextCommand::ArrowRight => {
                    let Some(char_at_index) = text[cursor_index..].chars().next() else {
                        continue;
                    };

                    cursor_index += char_at_index.len_utf8();
                }
                TextCommand::ArrowLeft => {
                    let Some(char_to_the_left) = text[..cursor_index].chars().next_back() else {
                        continue;
                    };

                    cursor_index -= char_to_the_left.len_utf8();
                }
                TextCommand::Backspace => {
                    let (left, right) = text.split_at(cursor_index);
                    let Some((char_to_the_left_index, char_to_the_left)) = left.char_indices().next_back() else {
                        continue;
                    };

                    text = left[..char_to_the_left_index].to_string() + right;
                    cursor_index -= char_to_the_left.len_utf8();
                }
                TextCommand::Delete => {
                    let (left, right) = text.split_at(cursor_index);
                    let Some(char_at_index) = right.chars().next() else {
                        continue;
                    };

                    text = left.to_string() + &right[char_at_index.len_utf8()..];
                }
                _ => {}
            },
        }
    }

    if text != old_text {
        on_text_change(text);
    }

    internal_state.cursor_index = cursor_index;

    if shuold_reset_cursor_blink {
        internal_state.reset_cursor_blink();
    }
}

struct BaseTextFieldInternalState {
    cursor_blink_start: Instant,
    cursor_index: usize,
}

impl BaseTextFieldInternalState {
    fn reset_cursor_blink(&mut self) {
        self.cursor_blink_start = Instant::now();
    }

    fn is_cursor_visible(&self) -> bool {
        (self.cursor_blink_start.elapsed().as_millis() as u64 % CURSOR_PERIOD_MILLIS) < (CURSOR_PERIOD_MILLIS / 2)
    }
}
