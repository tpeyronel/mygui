use std::sync::Arc;

use font::{font_engine::FontEngine, freetype_font_engine::FreetypeFontEngine};
use glam::Vec2;
use image::image_manager::{ImageManager, ImageManagerEvent};
use renderer::renderer::Renderer;
use ui::{draw_element::DrawElement, example_ui};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

mod config;
mod font;
mod image;
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
    font_engine: FreetypeFontEngine,
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

        let renderer = Renderer::new(Arc::clone(&window));

        let ui = example_ui();

        let image_manager = ImageManager::new();
        let mut font_engine = FreetypeFontEngine::new("./assets/fonts/");

        let mut draw_data = vec![];
        ui.to_draw_data(
            Vec2::ZERO,
            Vec2::new(window.inner_size().width as f32, window.inner_size().height as f32),
            &mut font_engine,
            &mut draw_data,
        );

        self.state = Some(AppState {
            window,
            draw_data,
            image_manager,
            font_engine,
            renderer,
        })
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(new_size) => {
                let state = self.state.as_mut().unwrap();
                state.font_engine.update(&mut state.image_manager);
                state.renderer.on_resize(new_size.width, new_size.height);

                state.draw_data.clear();
                example_ui().to_draw_data(
                    Vec2::ZERO,
                    Vec2::new(new_size.width as f32, new_size.height as f32),
                    &mut state.font_engine,
                    &mut state.draw_data,
                );

                state.renderer.update_draw_data(&state.draw_data);
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

                state.renderer.render();
            }
            WindowEvent::KeyboardInput {
                device_id: _device_id,
                event,
                is_synthetic: _is_synthetic,
            } => match event.physical_key {
                PhysicalKey::Code(winit::keyboard::KeyCode::Escape) if event.state == ElementState::Released => {
                    event_loop.exit()
                }
                _ => {}
            },
            _ => (),
        }
    }
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
