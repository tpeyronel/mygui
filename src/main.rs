use std::{borrow::Cow, sync::Arc};

use futures::executor;
use glam::{Vec2, Vec4};
use vertex::{Vertex, INDICES};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Device, Queue, RenderPipeline, Surface,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform {
    viewport_width: f32,
    viewport_height: f32,
}

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
    global_uniform: GlobalUniform,
    global_uniform_buffer: wgpu::Buffer,
    global_uniform_bind_group: wgpu::BindGroup,
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

        let global_uniform = GlobalUniform {
            viewport_width: size.width as f32,
            viewport_height: size.height as f32,
        };

        let global_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("global uniform buffer"),
            contents: bytemuck::cast_slice(&[global_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("global uniform group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let global_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("global uniform group"),
            layout: &global_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&global_uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(swapchain_capabilities.formats[0]);

        let bbox = Vec4::new(BOX_X, BOX_Y, BOX_X + BOX_WIDTH, BOX_Y + BOX_HEIGHT);
        let border_radius = Vec4::new(64.0, 48.0, 32.0, 16.0);
        let border_width = Vec4::new(2.0, 4.0, 8.0, 16.0);

        let vertices = [
            Vertex {
                pos: Vec2::new(BOX_X, BOX_Y),
                color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                bbox,
                border_radius,
                border_width,
            },
            Vertex {
                pos: Vec2::new(BOX_X + BOX_WIDTH, BOX_Y),
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                bbox,
                border_radius,
                border_width,
            },
            Vertex {
                pos: Vec2::new(BOX_X + BOX_WIDTH, BOX_Y + BOX_HEIGHT),
                color: Vec4::new(0.0, 0.0, 1.0, 1.0),
                bbox,
                border_radius,
                border_width,
            },
            Vertex {
                pos: Vec2::new(BOX_X, BOX_Y + BOX_HEIGHT),
                color: Vec4::new(1.0, 1.0, 0.0, 1.0),
                bbox,
                border_radius,
                border_width,
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
                buffers: &[Vertex::vertex_buffer_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            global_uniform,
            global_uniform_bind_group,
            global_uniform_buffer,
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
                state.global_uniform.viewport_width = state.config.width as f32;
                state.global_uniform.viewport_height = state.config.height as f32;
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

                state.queue.write_buffer(
                    &state.global_uniform_buffer,
                    0,
                    bytemuck::cast_slice(&[state.global_uniform]),
                );

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
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.8,
                                    g: 0.2,
                                    b: 0.2,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rpass.set_pipeline(&state.render_pipeline);
                    rpass.set_bind_group(0, &state.global_uniform_bind_group, &[]);
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
