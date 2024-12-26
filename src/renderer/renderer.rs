use std::{borrow::Cow, collections::HashMap, sync::Arc};

use futures::executor;
use wgpu::util::DeviceExt;

use crate::{
    font::font_face::GlyphPixelMode,
    image::image_manager::{ImageId, ImageManager},
    rectangle::Rectangle,
    ui::draw_element::DrawElement,
    vertex::{Color, Vertex},
};

use super::mesh::Mesh;

const MAX_RECTANGLES: u64 = 2048;
const MAX_VERTICES: u64 = 4 * MAX_RECTANGLES;
const MAX_INDICES: u64 = 6 * MAX_RECTANGLES;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    box_pipeline: wgpu::RenderPipeline,
    text_grayscale_pipeline: wgpu::RenderPipeline,
    text_subpixel_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    textures: HashMap<ImageId, Texture>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    global_uniform: GlobalUniform,
    global_uniform_buffer: wgpu::Buffer,
    global_uniform_bind_group: wgpu::BindGroup,
    rectangle_data_uniform_buffer: wgpu::Buffer,
    rectangle_data_uniform_bind_group: wgpu::BindGroup,
    config: wgpu::SurfaceConfiguration,
    processed_meshes: Vec<ProcessedMesh>,
}

impl Renderer {
    pub fn new(window: Arc<winit::window::Window>) -> Self {
        let instance = wgpu::Instance::default();
        let window_size = {
            let inner_size = window.inner_size();
            winit::dpi::PhysicalSize {
                width: inner_size.width.max(1),
                height: inner_size.height.max(1),
            }
        };

        let surface = instance.create_surface(window).unwrap();

        let adapter = executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface), // Request an adapter which can render to our surface
        }))
        .expect("Failed to find an appropriate adapter");

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
        let mut required_limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        required_limits.max_storage_buffers_per_shader_stage =
            wgpu::Limits::default().max_storage_buffers_per_shader_stage;
        required_limits.max_storage_buffer_binding_size = wgpu::Limits::default().max_storage_buffer_binding_size;
        required_limits.max_push_constant_size = 128;

        // Create the logical device and command queue
        let (device, queue) = executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::BUFFER_BINDING_ARRAY
                    | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY // TODO: apparently not needed?
                    | wgpu::Features::PUSH_CONSTANTS
                    | wgpu::Features::DUAL_SOURCE_BLENDING,
                required_limits,
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        ))
        .expect("Failed to create device");

        let box_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("box shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../assets/shaders/box_shader.wgsl"))),
        });

        let text_grayscale_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text grayscale shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../assets/shaders/text_grayscale_shader.wgsl"
            ))),
        });

        let text_subpixel_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text subpixel shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../assets/shaders/text_subpixel_shader.wgsl"
            ))),
        });

        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("texture shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../assets/shaders/texture_shader.wgsl"))),
        });

        let global_uniform = GlobalUniform {
            viewport_width: window_size.width as f32,
            viewport_height: window_size.height as f32,
        };

        let global_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("global uniform buffer"),
            contents: bytemuck::cast_slice(&[global_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let rectangle_data_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: "rectangle data uniform buffer".into(),
            size: MAX_RECTANGLES * std::mem::size_of::<Rectangle>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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

        let rectangle_data_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rectangle data uniform group"),
            layout: &rectangle_data_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: rectangle_data_uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let box_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &global_uniform_bind_group_layout,
                &rectangle_data_uniform_bind_group_layout,
            ],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..4,
            }],
        });

        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text pipeline layout"),
            bind_group_layouts: &[&global_uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..16,
            }],
        });

        let texture_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("texture pipeline layout"),
            bind_group_layouts: &[&global_uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..16,
            }],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(swapchain_capabilities.formats[0]);

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: "vertex buffer".into(),
            size: MAX_VERTICES * std::mem::size_of::<Vertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: "index buffer".into(),
            size: MAX_INDICES * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let box_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("box pipeline"),
            layout: Some(&box_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &box_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::vertex_buffer_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &box_shader,
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

        let text_grayscale_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text grayscale pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_grayscale_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::vertex_buffer_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_grayscale_shader,
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

        let text_subpixel_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text subpixel pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_subpixel_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::vertex_buffer_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_subpixel_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Src1,
                            dst_factor: wgpu::BlendFactor::OneMinusSrc1,
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

        let texture_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture pipeline"),
            layout: Some(&texture_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &texture_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::vertex_buffer_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &texture_shader,
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
            width: window_size.width,
            height: window_size.height,
            present_mode: swapchain_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            box_pipeline,
            text_grayscale_pipeline,
            text_subpixel_pipeline,
            texture_pipeline,
            texture_bind_group_layout,
            textures: HashMap::new(),
            vertex_buffer,
            index_buffer,
            global_uniform,
            global_uniform_buffer,
            global_uniform_bind_group,
            rectangle_data_uniform_buffer,
            rectangle_data_uniform_bind_group,
            config,
            processed_meshes: Vec::new(),
        }
    }

    pub fn on_resize(&mut self, new_window_width: u32, new_window_height: u32) {
        self.config.width = new_window_width.max(1);
        self.config.height = new_window_height.max(1);
        self.global_uniform.viewport_width = self.config.width as f32;
        self.global_uniform.viewport_height = self.config.height as f32;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn register_image(&mut self, image_manager: &ImageManager, image_id: ImageId) {
        assert!(!self.textures.contains_key(&image_id));

        let image = image_manager.get_image(image_id);

        let texture_size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{:?} texture", image_id)),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: image.format().to_wgpu_texture_format().expect("TODO"),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image.data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(image.pitch()),
                rows_per_image: Some(image.height()),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{:?} texture view", image_id)),
            ..Default::default()
        });

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{:?} sampler", image_id)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{:?} bind group", image_id)),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let texture = Texture {
            texture,
            texture_view,
            sampler,
            bind_group,
        };

        self.textures.insert(image_id, texture);
    }

    pub fn update_draw_data(&mut self, rectangles: &[DrawElement]) {
        let meshes: Vec<Mesh> = rectangles.iter().map(|e| Mesh::from_draw_element(e)).collect();

        let mut all_vertices: Vec<Vertex> = vec![];
        let mut all_indices: Vec<u32> = vec![];
        let mut all_rectangle_data = vec![];

        let processed_meshes = meshes
            .into_iter()
            .map(|m| {
                let base_vertex = all_vertices.len() as i32;
                let first_index = all_indices.len() as u32;
                match m {
                    Mesh::Rectangle {
                        vertices,
                        indices,
                        rectangle_data,
                    } => {
                        all_vertices.extend(vertices);
                        all_indices.extend(indices);

                        let rectangle_data_index = all_rectangle_data.len() as u32;
                        all_rectangle_data.push(rectangle_data);

                        ProcessedMesh::Rectangle {
                            base_vertex,
                            first_index,
                            rectangle_data_index,
                        }
                    }
                    Mesh::TextGlyph {
                        vertices,
                        indices,
                        text_color,
                        image_id,
                        pixel_mode,
                    } => {
                        all_vertices.extend(vertices);
                        all_indices.extend(indices);

                        ProcessedMesh::TextGlyph {
                            base_vertex,
                            first_index,
                            text_color,
                            image_id,
                            pixel_mode,
                        }
                    }
                }
            })
            .collect();

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&all_indices));
        self.queue.write_buffer(
            &self.rectangle_data_uniform_buffer,
            0,
            bytemuck::cast_slice(&all_rectangle_data),
        );
        self.processed_meshes = processed_meshes;
    }

    pub fn render(&mut self) {
        self.queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.global_uniform]),
        );

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
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
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for mesh in &self.processed_meshes {
                match mesh {
                    ProcessedMesh::Rectangle {
                        base_vertex,
                        first_index,
                        rectangle_data_index,
                    } => {
                        rpass.set_pipeline(&self.box_pipeline);
                        rpass.set_bind_group(0, &self.global_uniform_bind_group, &[]);
                        rpass.set_bind_group(1, &self.rectangle_data_uniform_bind_group, &[]);
                        rpass.set_push_constants(
                            wgpu::ShaderStages::VERTEX_FRAGMENT,
                            0,
                            bytemuck::cast_slice(&[*rectangle_data_index]),
                        );
                        rpass.draw_indexed(*first_index..*first_index + 6, *base_vertex, 0..1);
                    }
                    ProcessedMesh::TextGlyph {
                        base_vertex,
                        first_index,
                        text_color,
                        image_id,
                        pixel_mode,
                    } => {
                        let pipeline = match pixel_mode {
                            GlyphPixelMode::Grayscale => &self.text_grayscale_pipeline,
                            GlyphPixelMode::Subpixel => &self.text_subpixel_pipeline,
                            GlyphPixelMode::Color => &self.texture_pipeline,
                        };
                        rpass.set_pipeline(pipeline);
                        rpass.set_push_constants(
                            wgpu::ShaderStages::VERTEX_FRAGMENT,
                            0,
                            bytemuck::cast_slice(std::slice::from_ref(text_color)),
                        );
                        rpass.set_bind_group(0, &self.global_uniform_bind_group, &[]);
                        let texture = self.textures.get(&image_id).expect("TODO");
                        rpass.set_bind_group(1, &texture.bind_group, &[]);
                        rpass.draw_indexed(*first_index..*first_index + 6, *base_vertex, 0..1);
                    }
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

struct Texture {
    #[allow(unused)]
    texture: wgpu::Texture,
    #[allow(unused)]
    texture_view: wgpu::TextureView,
    #[allow(unused)]
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform {
    viewport_width: f32,
    viewport_height: f32,
}

enum ProcessedMesh {
    Rectangle {
        base_vertex: i32,
        first_index: u32,
        rectangle_data_index: u32,
    },
    TextGlyph {
        base_vertex: i32,
        first_index: u32,
        text_color: Color,
        image_id: ImageId,
        pixel_mode: GlyphPixelMode,
    },
}
