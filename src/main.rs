use std::{borrow::Cow, sync::Arc};

use futures::executor;
use glam::{Vec2, Vec3, Vec4};
use vertex::{Vertex, INDICES};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Device, Queue, RenderPipeline, Surface, VertexAttribute, VertexBufferLayout,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

mod vertex;

pub const BOX_WIDTH: f32 = 128.0 * 4.0;
pub const BOX_HEIGHT: f32 = 236.0;

pub const BOX_X: f32 = 64.0;
pub const BOX_Y: f32 = 64.0;

struct App {
    state: Option<AppState>,
}

struct AppState {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    config: wgpu::SurfaceConfiguration,
}

impl App {
    fn new() -> Self {
        return Self { state: None };
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        ))
        .expect("Failed to create device");

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(swapchain_capabilities.formats[0]);

        let bbox_bottom_left = Vec2::new(BOX_X, BOX_Y);
        let bbox_top_right = Vec2::new(BOX_X + BOX_WIDTH, BOX_Y + BOX_HEIGHT);
        let border_radius = Vec2::splat(32.0);

        let vertices = [
            Vertex {
                pos: Vec2::new(BOX_X, BOX_Y),
                color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                bbox_bottom_left,
                bbox_top_right,
                border_radius,
            },
            Vertex {
                pos: Vec2::new(BOX_X + BOX_WIDTH, BOX_Y),
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                bbox_bottom_left,
                bbox_top_right,
                border_radius,
            },
            Vertex {
                pos: Vec2::new(BOX_X + BOX_WIDTH, BOX_Y + BOX_HEIGHT),
                color: Vec4::new(0.0, 0.0, 1.0, 1.0),
                bbox_bottom_left,
                bbox_top_right,
                border_radius,
            },
            Vertex {
                pos: Vec2::new(BOX_X, BOX_Y + BOX_HEIGHT),
                color: Vec4::new(1.0, 1.0, 0.0, 1.0),
                bbox_bottom_left,
                bbox_top_right,
                border_radius,
            },
        ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: std::mem::size_of::<Vec4>() as u64,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: (std::mem::size_of::<Vec4>() + std::mem::size_of::<Vec2>())
                                as u64,
                            shader_location: 2,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: (std::mem::size_of::<Vec4>() + 2 * std::mem::size_of::<Vec2>())
                                as u64,
                            shader_location: 3,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: (std::mem::size_of::<Vec4>() + 3 * std::mem::size_of::<Vec2>())
                                as u64,
                            shader_location: 4,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: swapchain_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        self.state = Some(AppState {
            window,
            surface,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            config,
        })
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                let state = self.state.as_mut().unwrap();
                state.config.width = new_size.width.max(1);
                state.config.height = new_size.height.max(1);
                state.surface.configure(&state.device, &state.config);
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

                let state = self.state.as_ref().unwrap();

                let frame = state
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rpass.set_pipeline(&state.render_pipeline);
                    rpass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
                    rpass.set_index_buffer(state.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
                }

                state.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => match event.physical_key {
                PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => event_loop.exit(),
                _ => {}
            },
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
