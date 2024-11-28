use std::{borrow::Cow, sync::Arc};

use futures::executor;
use glam::Vec2;
use rectangle::Rectangle;
use ui::{example_ui, UiNode};
use vertex::Vertex;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Device, Queue, RenderPipeline, Surface,
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

mod rectangle;
mod ui;
mod vertex;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform {
    viewport_width: f32,
    viewport_height: f32,
}

fn rectangles_to_vertices_and_indices(rectangles: &[Rectangle]) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = vec![];
    let mut indices = vec![];

    for r in rectangles {
        let i = vertices.len() as u32;
        indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
        vertices.extend(r.to_vertices());
    }

    (vertices, indices)
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
    ui: UiNode,
    rectangles: Vec<Rectangle>,
    rectangle_data_uniform_buffer: wgpu::Buffer,
    rectangle_data_uniform_bind_group: wgpu::BindGroup,
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
                .create_window(Window::default_attributes().with_inner_size(
                    winit::dpi::Size::Physical(winit::dpi::PhysicalSize {
                        width: 1280,
                        height: 720,
                    }),
                ))
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

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
        let mut required_limits =
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        required_limits.max_storage_buffers_per_shader_stage =
            wgpu::Limits::default().max_storage_buffers_per_shader_stage;
        required_limits.max_storage_buffer_binding_size =
            wgpu::Limits::default().max_storage_buffer_binding_size;

        // Create the logical device and command queue
        let (device, queue) = executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::BUFFER_BINDING_ARRAY
                    | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY, // TODO: apparently not needed?
                required_limits,
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

        let ui = example_ui();

        let mut rectangles = vec![];
        let layout = ui.compute_layout(Vec2::ZERO, Vec2::new(size.width as f32, size.height as f32));
        ui.to_draw_data(&layout, &mut rectangles);

        // let rectangles = vec![
        //     Rectangle {
        //         position: Vec2::new(16.0, 16.0),
        //         size: Vec2::new(64.0, 64.0),
        //         fill_color: Vec4::new(0.1, 1.0, 0.1, 1.0),
        //         border_color: Vec4::new(1.0, 1.0, 1.0, 0.8),
        //         border_radius: Vec4::splat(8.0),
        //         border_width: Vec4::splat(2.0),
        //     },
        //     Rectangle {
        //         position: Vec2::new(16.0, 16.0 + 64.0 + 16.0),
        //         size: Vec2::new(128.0 * 4.0, 236.0),
        //         fill_color: Vec4::new(1.0, 0.1, 0.1, 1.0),
        //         border_color: Vec4::new(1.0, 1.0, 1.0, 0.8),
        //         border_radius: Vec4::new(64.0, 48.0, 32.0, 16.0),
        //         border_width: Vec4::new(-16.0, 4.0, 8.0, 16.0),
        //     },
        //     Rectangle {
        //         position: Vec2::new(16.0, 16.0 + 64.0 + 16.0 + 236.0 + 16.0),
        //         size: Vec2::new(128.0 * 4.0, 236.0),
        //         fill_color: Vec4::new(0.0, 0.0, 0.0, 0.4),
        //         border_color: Vec4::new(1.0, 1.0, 1.0, 0.5),
        //         border_radius: Vec4::new(4.0, 8.0, 16.0, 24.0),
        //         border_width: Vec4::splat(2.0),
        //     },
        //     Rectangle {
        //         position: Vec2::new(16.0 + 512.0 + 16.0, 16.0 + 64.0 + 16.0 + 236.0 + 16.00),
        //         size: Vec2::new(16.0 * 4.0, 236.0),
        //         fill_color: Vec4::new(0.0, 0.0, 0.0, 0.4),
        //         border_color: Vec4::new(1.0, 1.0, 1.0, 0.5),
        //         border_radius: Vec4::new(4.0, 8.0, 16.0, 24.0),
        //         border_width: Vec4::splat(0.0),
        //     },
        // ];

        let rectangle_data_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("rectangle data uniform buffer"),
                contents: bytemuck::cast_slice(rectangles.as_slice()),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let rectangle_data_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("rectangle data uniform group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let rectangle_data_uniform_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("rectangle data uniform group"),
                layout: &rectangle_data_uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: rectangle_data_uniform_buffer.as_entire_binding(),
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &global_uniform_bind_group_layout,
                &rectangle_data_uniform_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(swapchain_capabilities.formats[0]);

        let (vertices, indices) = rectangles_to_vertices_and_indices(&rectangles);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
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
            ui,
            rectangles,
            rectangle_data_uniform_bind_group,
            rectangle_data_uniform_buffer,
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

                state.rectangles.clear();
                let layout = state.ui.compute_layout(Vec2::ZERO, Vec2::new(
                    state.global_uniform.viewport_width,
                    state.global_uniform.viewport_height,
                ));
                state.ui.to_draw_data(&layout, &mut state.rectangles);


                let (vertices, indices) = rectangles_to_vertices_and_indices(&state.rectangles);
                state
                    .queue
                    .write_buffer(&state.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
                state
                    .queue
                    .write_buffer(&state.index_buffer, 0, bytemuck::cast_slice(&indices));
                state.queue.write_buffer(
                    &state.rectangle_data_uniform_buffer,
                    0,
                    bytemuck::cast_slice(&state.rectangles),
                );
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
                    rpass.set_bind_group(1, &state.rectangle_data_uniform_bind_group, &[]);
                    rpass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
                    rpass.set_index_buffer(state.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..(state.rectangles.len() * 6) as u32, 0, 0..1);
                }

                state.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => match event.physical_key {
                PhysicalKey::Code(winit::keyboard::KeyCode::Escape)
                    if event.state == ElementState::Released =>
                {
                    event_loop.exit()
                }
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
