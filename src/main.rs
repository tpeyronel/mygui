use std::{sync::Arc, time::Instant};

use font::{
    font_engine::{FontEngine, TextPosition},
    freetype_font_engine::FreetypeFontEngine,
};
use glam::Vec2;
use image::image_manager::{ImageManager, ImageManagerEvent};
use input::{ElementState, InputEvent, MouseButton};
use renderer::renderer::Renderer;
use ui::{
    border_radius::BorderRadius,
    border_thickness::BorderThickness,
    draw_element::DrawElement,
    immediate::{
        context::{NodeInputEvent, UiContext},
        ui::Ui,
    },
    margin::Margin,
    padding::Padding,
    Alignment, Extent,
};
use vertex::Color;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

mod config;
mod font;
mod image;
mod input;
mod is_integer;
mod rectangle;
mod renderer;
mod ui;
mod vertex;

struct App {
    state: Option<AppState>,
}

struct AppState {
    #[allow(dead_code)]
    window: Arc<Window>,
    draw_data: Vec<DrawElement>,
    image_manager: ImageManager,
    font_engine: Box<dyn FontEngine>,
    ui_context: UiContext,
    renderer: Renderer,
}

impl App {
    fn new() -> Self {
        return Self { state: None };
    }

    fn process_image_manager_events(state: &mut AppState) {
        let image_manager_events = state.image_manager.collect_events();

        for e in image_manager_events {
            match e {
                ImageManagerEvent::ImageCreated(image_id) => {
                    state.renderer.register_image(&state.image_manager, image_id);
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_inner_size(winit::dpi::Size::Physical(
                    winit::dpi::PhysicalSize {
                        width: 1280,
                        height: 720,
                    },
                )))
                .unwrap(),
        );

        let mut image_manager = ImageManager::new();
        let font_engine = Box::new(FreetypeFontEngine::new(
            "./assets/fonts/",
            "./cache/fonts/",
            &mut image_manager,
        ));
        let ui_context = UiContext::new();
        let renderer = Renderer::new(Arc::clone(&window));

        self.state = Some(AppState {
            window,
            draw_data: vec![],
            image_manager,
            font_engine,
            ui_context,
            renderer,
        })
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(new_size) => {
                let state = self.state.as_mut().unwrap();
                state.renderer.on_resize(new_size.width, new_size.height);
            }
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                // self.window.as_ref().unwrap().request_redraw();

                let state = self.state.as_mut().unwrap();

                state.font_engine.update(&mut state.image_manager);

                Self::process_image_manager_events(state);

                let window_size = state.window.inner_size();
                let window_size = Vec2::new(window_size.width as f32, window_size.height as f32);
                state.draw_data = state.ui_context.build_ui(window_size, &mut state.font_engine, |ui| {
                    example_ui(ui);
                });
                state.renderer.update_draw_data(&state.draw_data);
                state.renderer.render();

                state.window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                device_id: _device_id,
                event,
                is_synthetic: _is_synthetic,
            } => {
                let state = self.state.as_mut().unwrap();

                match event.physical_key {
                    PhysicalKey::Code(winit::keyboard::KeyCode::Escape)
                        if event.state == winit::event::ElementState::Released =>
                    {
                        event_loop.exit();
                        return;
                    }
                    _ => {}
                };

                if event.state != winit::event::ElementState::Pressed {
                    return;
                }

                if let Some(text) = event.text {
                    let text = if text.as_str() == "\r" { "\n" } else { text.as_str() };
                    let event = InputEvent::TextInput { text: text.to_string() };

                    state.ui_context.process_input_event(event);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let state = self.state.as_mut().unwrap();

                let window_height = state.window.inner_size().height as f32;
                let cursor_position = Vec2::new(position.x as f32, window_height - position.y as f32);

                state.draw_data.clear();
                state.ui_context.process_input_event(InputEvent::CursorMoved {
                    position: cursor_position,
                });
                state.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                let state = self.state.as_mut().unwrap();

                let Some(button) = MouseButton::from_winit(button) else {
                    return;
                };

                let button_state = ElementState::from_winit(button_state);

                state.ui_context.process_input_event(InputEvent::MouseInput {
                    button,
                    state: button_state,
                });

                state.window.request_redraw();
            }
            _ => (),
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        println!("exiting...");
        let state = self.state.as_mut().unwrap();
        state.font_engine.on_exit(&state.image_manager);
    }
}

#[allow(unused)]
fn simple_ui(ui: &mut Ui<'_>) {
    ui.column(|ui, _| {
        (0..3).for_each(|_| {
            ui.row(|ui, attr| {
                attr.height(Extent::FitContent);

                ui.block(|ui, attr| {
                    let input = ui.use_input();

                    attr.width(Extent::Px(0.0))
                        .height(Extent::Px(48.0))
                        .weight(1.0)
                        .fill_color(if input.is_hovered() {
                            Color::new(1.0, 1.0, 0.0, 1.0)
                        } else {
                            Color::new(1.0, 0.0, 0.0, 1.0)
                        });

                    ui.block(|_, attr| {
                        attr.margin(Margin::all(16.0))
                            .fill_color(Color::new(0.0, 1.0, 0.0, 0.5));
                    });
                });

                ui.text(
                    "Yeahhhdqwdqwdqwdqwdqwdqwddqwdh\nqiwdhqqwdqwdqwdqwdw",
                    |ui, props, modifiers| {
                        let input = ui.use_input();

                        props.font_family = "Jetbrains Mono".into();
                        props.font_size = 64.0;
                        props.line_height = 64.0;

                        modifiers
                            .width(Extent::Px(0.0))
                            .weight(1.0)
                            .height(Extent::FitContent)
                            .fill_color(if input.is_hovered() {
                                Color::new(1.0, 0.0, 0.5, 0.5)
                            } else {
                                Color::new(0.5, 0.5, 0.5, 0.5)
                            })
                            .self_alignment(ui::Alignment::Bottom);
                    },
                );

                ui.block(|_, attr| {
                    attr.width(Extent::Px(0.0))
                        .weight(1.0)
                        .fill_color(Color::new(0.0, 0.0, 1.0, 1.0));
                });
            });
        });
    });
}

fn example_ui(ui: &mut Ui<'_>) {
    ui.block(|ui, modifiers| {
        modifiers
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .margin(Margin::all(8.0))
            .padding(Padding::all(16.0))
            .fill_color(Color::new(1.0, 1.0, 0.1, 0.25))
            .border_radius(BorderRadius::all(16.0));

        ui.block(|ui, modifiers| {
            modifiers
                .width(Extent::FillParent)
                .height(Extent::FillParent)
                .padding(Padding::all(0.0))
                .fill_color(Color::new(1.0, 0.1, 0.1, 0.25))
                .border_color(Color::new(1.0, 0.1, 0.1, 0.9))
                .border_thickness(BorderThickness::all(4.0))
                .border_radius(BorderRadius::all(8.0));

            ui.block(|_, modifiers| {
                modifiers
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .self_alignment(Alignment::Center)
                    .fill_color(Color::new(1.0, 1.0, 1.0, 0.25))
                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                    .border_thickness(BorderThickness::all(1.0))
                    .border_radius(BorderRadius::all(4.0));
            });

            ui.block(|_, modifiers| {
                modifiers
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .margin(Margin::all(4.0))
                    .self_alignment(Alignment::Right)
                    .fill_color(Color::new(1.0, 0.0, 0.0, 0.25))
                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                    .border_thickness(BorderThickness::all(1.0))
                    .border_radius(BorderRadius::all(4.0));
            });

            ui.block(|_, modifiers| {
                modifiers
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .self_alignment(Alignment::TopRight)
                    .fill_color(Color::new(1.0, 1.0, 0.0, 0.25))
                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                    .border_thickness(BorderThickness::all(1.0))
                    .border_radius(BorderRadius::new(0.0, 8.0, 16.0, 24.0));
            });

            ui.block(|_, modifiers| {
                modifiers
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .self_alignment(Alignment::Top)
                    .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                    .border_thickness(BorderThickness::new(4.0, 8.0, 12.0, 16.0));
            });

            ui.block(|ui, modifiers| {
                modifiers
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::TopLeft)
                    .border_color(Color::new(1.0, 1.0, 1.0, 0.4))
                    .border_thickness(BorderThickness::all(2.0));

                ui.block(|ui, modifiers| {
                    modifiers
                        .width(Extent::FillParent)
                        .height(Extent::FillParent)
                        .padding(Padding::all(8.0))
                        .border_color(Color::new(1.0, 0.0, 0.0, 0.4))
                        .border_thickness(BorderThickness::all(2.0));

                    ui.block(|_, modifiers| {
                        modifiers.fill_color(Color::new(0.0, 1.0, 0.0, 0.4));
                    });
                });

                ui.block(|_, modifiers| {
                    modifiers
                        .width(Extent::Px(8.0))
                        .height(Extent::Px(64.0))
                        .border_color(Color::new(0.0, 1.0, 0.0, 0.4))
                        .border_thickness(BorderThickness::all(2.0))
                        .self_alignment(Alignment::BottomLeft);
                });

                ui.block(|_, modifiers| {
                    modifiers
                        .width(Extent::Px(96.0))
                        .height(Extent::Px(8.0))
                        .border_color(Color::new(0.0, 0.0, 1.0, 0.4))
                        .border_thickness(BorderThickness::all(2.0))
                        .self_alignment(Alignment::TopRight);
                });
            });

            ui.block(|_, modifiers| {
                modifiers
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .self_alignment(Alignment::Left)
                    .fill_color(Color::new(0.0, 0.0, 0.0, 0.25))
                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                    .border_thickness(BorderThickness::all(1.0))
                    .border_radius(BorderRadius::all(4.0));
            });

            ui.block(|_, modifiers| {
                modifiers
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .self_alignment(Alignment::BottomLeft)
                    .fill_color(Color::new(0.0, 0.0, 1.0, 0.25))
                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                    .border_thickness(BorderThickness::all(1.0))
                    .border_radius(BorderRadius::all(4.0));
            });

            ui.block(|_, modifiers| {
                modifiers
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .self_alignment(Alignment::Bottom)
                    .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                    .border_thickness(BorderThickness::all(1.0))
                    .border_radius(BorderRadius::all(4.0));
            });

            ui.block(|_, modifiers| {
                modifiers
                    .width(Extent::Px(80.0))
                    .height(Extent::Px(80.0))
                    .self_alignment(Alignment::BottomRight)
                    .fill_color(Color::new(1.0, 0.0, 1.0, 0.25))
                    .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                    .border_thickness(BorderThickness::all(1.0))
                    .border_radius(BorderRadius::all(4.0));
            });

            ui.row(|ui, modifiers| {
                modifiers
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .padding(Padding::all(8.0))
                    .border_thickness(BorderThickness::all(4.0))
                    .border_color(Color::new(1.0, 1.0, 1.0, 1.0));

                ui.column(|ui, modifiers| {
                    modifiers
                        .width(Extent::Px(256.0))
                        .height(Extent::FitContent)
                        .padding(Padding::all(16.0))
                        .border_thickness(BorderThickness::all(4.0))
                        .border_color(Color::new(1.0, 1.0, 1.0, 1.0));

                    ui.block(|_, modifiers| {
                        modifiers
                            .height(Extent::Px(24.0))
                            .fill_color(Color::new(0.0, 1.0, 1.0, 0.5))
                            .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                            .border_thickness(BorderThickness::all(1.0))
                            .border_radius(BorderRadius::all(8.0));
                    });

                    ui.block(|_, modifiers| {
                        modifiers
                            .height(Extent::FillParent)
                            .fill_color(Color::new(1.0, 0.0, 1.0, 0.5))
                            .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                            .border_thickness(BorderThickness::all(1.0))
                            .border_radius(BorderRadius::all(8.0));
                    });

                    ui.block(|_, modifiers| {
                        modifiers
                            .height(Extent::Px(32.0))
                            .fill_color(Color::new(1.0, 0.0, 0.0, 0.5))
                            .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                            .border_thickness(BorderThickness::all(1.0))
                            .border_radius(BorderRadius::all(8.0));
                    });

                    ui.block(|ui, modifiers| {
                        modifiers
                            .width(Extent::Px(96.0))
                            .height(Extent::Px(64.0))
                            .margin(Margin::all(8.0))
                            .padding(Padding::all(8.0))
                            .self_alignment(Alignment::Center)
                            .fill_color(Color::new(1.0, 1.0, 0.0, 0.5))
                            .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                            .border_thickness(BorderThickness::all(4.0))
                            .border_radius(BorderRadius::all(8.0));

                        ui.block(|_, modifiers| {
                            modifiers
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 1.0, 1.0, 0.5))
                                .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                .border_thickness(BorderThickness::all(1.0))
                                .border_radius(BorderRadius::all(8.0));
                        });
                    });

                    ui.block(|_, modifiers| {
                        modifiers
                            .height(Extent::Px(32.0))
                            .fill_color(Color::new(0.0, 1.0, 0.0, 0.5))
                            .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                            .border_thickness(BorderThickness::all(1.0))
                            .border_radius(BorderRadius::all(8.0));
                    });
                });

                ui.block(|_, modifiers| {
                    modifiers
                        .width(Extent::Px(64.0))
                        .height(Extent::FillParent)
                        .fill_color(Color::new(0.25, 0.25, 1.0, 0.5));
                });

                ui.block(|_, modifiers| {
                    modifiers
                        .width(Extent::Px(64.0))
                        .height(Extent::Px(32.0))
                        .fill_color(Color::new(0.25, 0.25, 1.0, 0.25));
                });

                ui.block(|ui, modifiers| {
                    modifiers
                        .width(Extent::FillParent)
                        .height(Extent::FillParent)
                        .fill_color(Color::new(0.25, 1.0, 0.25, 0.5));

                    ui.text(
                        "ÓThis is a text!\nÓWith 😊👍😭three lines\nÓThis is the last lineeeeeeeeee.",
                        |_, props, modifiers| {
                            props.text_color = Color::ONE;
                            props.font_family = "jetbrains mono".to_string();
                            props.font_size = 24.0;
                            props.line_height = 24.0 * 1.5;
                            modifiers.self_alignment(Alignment::TopLeft);
                        },
                    );
                });
            });

            ui.column(|ui, modifiers| {
                modifiers.width(Extent::Px(256.0)).self_alignment(Alignment::Left);

                ui.block(|_, modifiers| {
                    modifiers
                        .height(Extent::Px(0.0))
                        .weight(1.0)
                        .fill_color(Color::new(1.0, 0.0, 0.0, 0.4));
                });

                ui.block(|_, modifiers| {
                    modifiers
                        .height(Extent::Px(0.0))
                        .weight(2.0)
                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.4));
                });

                ui.block(|_, modifiers| {
                    modifiers
                        .height(Extent::Px(0.0))
                        .weight(1.0)
                        .fill_color(Color::new(0.0, 0.0, 1.0, 0.4));
                });
            });

            ui.column(|ui, modifiers| {
                modifiers.height(Extent::FitContent);

                ui.row(|ui, modifiers| {
                    modifiers.height(Extent::Px(128.0)).self_alignment(Alignment::Top);

                    ui.block(|_, modifiers| {
                        modifiers
                            .width(Extent::Px(0.0))
                            .weight(1.0)
                            .fill_color(Color::new(1.0, 0.0, 0.0, 0.4));
                    });

                    ui.block(|_, modifiers| {
                        modifiers
                            .width(Extent::Px(0.0))
                            .weight(2.0)
                            .fill_color(Color::new(0.0, 1.0, 0.0, 0.4));
                    });

                    ui.block(|ui, modifiers| {
                        modifiers
                            .width(Extent::Px(0.0))
                            .weight(1.0)
                            .fill_color(Color::new(0.0, 0.0, 1.0, 0.4));

                        ui.text("HellÓowjdoqi12931289😊👍😭3u!\nYegh", |_, props, modifiers| {
                            props.text_color = Color::ONE;
                            props.font_family = "Segoe UI Emoji".to_string();
                            props.font_size = 24.0;
                            props.line_height = 24.0;
                            modifiers
                                .width(Extent::FillParent)
                                .max_width(Extent::Px(512.0))
                                .min_width(Extent::Px(256.0))
                                .height(Extent::FitContent)
                                .max_height(Extent::Px(512.0))
                                .padding(Padding::all(64.0))
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.5));
                        });
                    });
                });

                ui.row(|ui, modifiers| {
                    modifiers.height(Extent::Px(128.0)).self_alignment(Alignment::Top);

                    ui.block(|_, modifiers| {
                        modifiers
                            .width(Extent::Px(0.0))
                            .weight(2.0)
                            .fill_color(Color::new(1.0, 1.0, 0.0, 0.4));
                    });

                    ui.block(|_, modifiers| {
                        modifiers
                            .width(Extent::Px(0.0))
                            .weight(1.0)
                            .fill_color(Color::new(0.0, 1.0, 1.0, 0.4));
                    });

                    ui.block(|ui, modifiers| {
                        modifiers
                            .width(Extent::Px(0.0))
                            .weight(3.0)
                            .fill_color(Color::new(1.0, 0.0, 1.0, 0.4));

                        let (count, set_count) = ui.use_state(|| 0);
                        let cursor_start_ref = ui.use_ref(|| Instant::now());

                        ui.text(
                            format!("{} HellÓowjdoqi129312893u!\nYegh", count),
                            |ui, props, modifiers| {
                                let input = ui.use_input();

                                props.text_color = Color::ONE;
                                props.font_family = "times new roman".to_string();
                                props.font_size = 17.0;
                                props.line_height = 17.0;
                                if cursor_start_ref.borrow().elapsed().as_millis() % 1000 < 500 {
                                    props.cursor_position = Some(TextPosition { line: 0, column: count });
                                }

                                modifiers
                                    .fill_color(if input.is_pressed() {
                                        Color::new(1.0, 1.0, 1.0, 0.5)
                                    } else if input.is_hovered() {
                                        Color::new(1.0, 1.0, 1.0, 0.25)
                                    } else {
                                        Color::new(0.0, 0.0, 0.0, 0.5)
                                    })
                                    .border_radius(BorderRadius::all(8.0))
                                    .padding(Padding::all(8.0));

                                if input.on_release() {
                                    set_count(&(count + 1));
                                    *cursor_start_ref.borrow_mut() = Instant::now();
                                }
                            },
                        );
                    });
                });
            });

            ui.column(|ui, modifiers| {
                modifiers
                    .width(Extent::FitContent)
                    .height(Extent::FitContent)
                    .self_alignment(Alignment::BottomRight);

                let (count, set_count) = ui.use_state(|| 0);

                for i in 5..32 {
                    ui.text(format!("{} aAbBcCdDoOÓgfjpq", count), |ui, props, modifiers| {
                        let input = ui.use_input();

                        props.text_color = Color::new(0.0 + (i - 5) as f32 / 31.0, (31 - i) as f32 / (26.0), 1.0, 1.0);
                        props.font_family = "tangerine".to_string();
                        props.font_size = i as f32;
                        props.line_height = i as f32;

                        modifiers
                            .width(Extent::FitContent)
                            .height(Extent::FitContent)
                            .no_max_width()
                            .self_alignment(Alignment::Left)
                            .fill_color(if input.is_pressed() {
                                Color::new(0.0, 0.0, 1.0, 1.0)
                            } else if input.on_hover() {
                                Color::new(1.0, 1.0, 1.0, 1.0)
                            } else if input.on_unhover() {
                                Color::new(0.0, 0.0, 0.0, 1.0)
                            } else if i % 2 == 0 {
                                Color::new(1.0, 0.0, 0.0, 0.5)
                            } else {
                                Color::new(0.0, 1.0, 0.0, 0.5)
                            });

                        if input.on_press() {
                            set_count(&(count + 1));
                        }
                    });
                }
            });
        });
    });
}

fn main() {
    env_logger::builder()
        .filter_module("mygui", log::LevelFilter::Trace)
        .init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
