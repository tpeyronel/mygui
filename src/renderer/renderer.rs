use std::{borrow::Cow, collections::HashMap, sync::Arc};

use futures::executor;
use glam::{Vec2, Vec4};
use wgpu::{util::DeviceExt, Extent3d};

use crate::{
    color::Color,
    font::font_face::GlyphPixelMode,
    image::image_manager::{ImageId, ImageManager},
    mesh::{
        mesh::{Mesh, MeshId, VertexAttribute},
        mesh_manager::MeshManager,
    },
    ui::{self, draw_command::DrawCommand},
};

const MSAA_SAMPLE_COUNT: u32 = 8;
const DEPTH_STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

macro_rules! vec2_vertex_buffer_layout {
    ($shader_location:expr) => {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vec2>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: $shader_location,
            }],
        }
    };
}

macro_rules! vec4_vertex_buffer_layout {
    ($shader_location:expr) => {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vec4>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: $shader_location,
            }],
        }
    };
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    multisampled_framebuffer_view: wgpu::TextureView,
    multisampled_depthbuffer_view: wgpu::TextureView,
    target_framebuffer_view: wgpu::TextureView,
    target_framebuffer_sampler: wgpu::Sampler,
    color_pipeline: wgpu::RenderPipeline,
    text_grayscale_pipeline: wgpu::RenderPipeline,
    text_subpixel_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    textures: HashMap<ImageId, Texture>,
    global_uniform: GlobalUniform,
    global_uniform_buffer: wgpu::Buffer,
    global_uniform_bind_group: wgpu::BindGroup,
    postprocess_bind_group_layout: wgpu::BindGroupLayout,
    postprocess_bind_group: wgpu::BindGroup,
    postprocess_pipeline: wgpu::RenderPipeline,
    config: wgpu::SurfaceConfiguration,
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
                    | wgpu::Features::DUAL_SOURCE_BLENDING
                    | wgpu::Features::POLYGON_MODE_LINE
                    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits,
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        ))
        .expect("Failed to create device");

        let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("color shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../assets/shaders/color_shader.wgsl"))),
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

        let postprocess_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("postprocess shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../assets/shaders/gamma_correction.wgsl"
            ))),
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

        let color_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&global_uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text pipeline layout"),
            bind_group_layouts: &[&global_uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let texture_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("texture pipeline layout"),
            bind_group_layouts: &[&global_uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = wgpu::TextureFormat::Bgra8Unorm;

        let multisampled_framebuffer_view =
            Self::create_msaa_framebuffer(&device, window_size.width, window_size.height, swapchain_format);

        let multisampled_depthbuffer_view =
            Self::create_msaa_depthbuffer(&device, window_size.width, window_size.height);

        let target_framebuffer_view =
            Self::create_target_framebuffer(&device, window_size.width, window_size.height, swapchain_format);

        let target_framebuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("target framebuffer sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        };

        let depth_stencil_state: Option<wgpu::DepthStencilState> = Some(wgpu::DepthStencilState {
            format: DEPTH_STENCIL_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            bias: wgpu::DepthBiasState::default(),
            stencil: wgpu::StencilState {
                front: stencil_state,
                back: stencil_state,
                read_mask: 0xff,
                write_mask: 0xff,
            },
        });

        let color_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("color pipeline"),
            layout: Some(&color_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &color_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    vec2_vertex_buffer_layout!(0), // Positions
                    vec4_vertex_buffer_layout!(1), // Colors
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &color_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                // polygon_mode: wgpu::PolygonMode::Line,
                ..Default::default()
            },
            depth_stencil: depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState {
                count: MSAA_SAMPLE_COUNT,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let text_grayscale_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text grayscale pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_grayscale_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    vec2_vertex_buffer_layout!(0), // Positions
                    vec2_vertex_buffer_layout!(1), // UVs
                    vec4_vertex_buffer_layout!(2), // Colors
                ],
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
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState {
                count: MSAA_SAMPLE_COUNT,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let text_subpixel_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text subpixel pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_subpixel_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    vec2_vertex_buffer_layout!(0), // Positions
                    vec2_vertex_buffer_layout!(1), // UVs
                    vec4_vertex_buffer_layout!(2), // Colors
                ],
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
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrc1,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState {
                count: MSAA_SAMPLE_COUNT,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let texture_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture pipeline"),
            layout: Some(&texture_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &texture_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    vec2_vertex_buffer_layout!(0), // Positions
                    vec2_vertex_buffer_layout!(1), // UVs
                ],
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
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState {
                count: MSAA_SAMPLE_COUNT,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let postprocess_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("postprocess bind group layout"),
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

        let postprocess_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("postprocess pipeline layout"),
            bind_group_layouts: &[&postprocess_bind_group_layout],
            push_constant_ranges: &[],
        });

        let postprocess_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("postprocess pipeline"),
            layout: Some(&postprocess_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &postprocess_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &postprocess_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let postprocess_bind_group = Self::create_postprocess_bind_group(
            &device,
            &texture_bind_group_layout,
            &target_framebuffer_view,
            &target_framebuffer_sampler,
        );

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
            multisampled_framebuffer_view,
            multisampled_depthbuffer_view,
            target_framebuffer_view,
            target_framebuffer_sampler,
            color_pipeline,
            text_grayscale_pipeline,
            text_subpixel_pipeline,
            texture_pipeline,
            texture_bind_group_layout,
            textures: HashMap::new(),
            global_uniform,
            global_uniform_buffer,
            global_uniform_bind_group,
            postprocess_bind_group_layout,
            postprocess_bind_group,
            postprocess_pipeline,
            config,
        }
    }

    pub fn on_resize(&mut self, new_window_width: u32, new_window_height: u32) {
        self.config.width = new_window_width.max(1);
        self.config.height = new_window_height.max(1);
        self.global_uniform.viewport_width = self.config.width as f32;
        self.global_uniform.viewport_height = self.config.height as f32;
        self.surface.configure(&self.device, &self.config);
        self.multisampled_framebuffer_view =
            Self::create_msaa_framebuffer(&self.device, self.config.width, self.config.height, self.config.format);
        self.multisampled_depthbuffer_view =
            Self::create_msaa_depthbuffer(&self.device, self.config.width, self.config.height);
        self.target_framebuffer_view =
            Self::create_target_framebuffer(&self.device, self.config.width, self.config.height, self.config.format);
        self.postprocess_bind_group = Self::create_postprocess_bind_group(
            &self.device,
            &self.postprocess_bind_group_layout,
            &self.target_framebuffer_view,
            &self.target_framebuffer_sampler,
        );
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

    fn create_mesh(&self, mesh_id: MeshId, mesh: &Mesh) -> MeshData {
        let mut vertex_buffers = HashMap::new();

        for (attrib, data) in &mesh.vertex_attributes {
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} {:?} buffer", mesh_id, attrib)),
                contents: data,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

            vertex_buffers.insert(*attrib, vertex_buffer);
        }

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} index buffer", mesh_id)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_count = mesh.indices.len() as u32;

        let mesh_data = MeshData {
            vertex_buffers,
            index_buffer,
            index_count,
            image_id: mesh.image_id,
        };

        mesh_data
    }

    pub fn render(&mut self, mesh_manager: &MeshManager, command_list: &[DrawCommand]) {
        let mesh_data: Vec<_> = mesh_manager
            .meshes()
            .iter()
            .enumerate()
            .map(|(idx, mesh)| self.create_mesh(MeshId(idx), mesh))
            .collect();

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
                    view: &self.multisampled_framebuffer_view,
                    resolve_target: Some(&self.target_framebuffer_view),
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.multisampled_depthbuffer_view,
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Discard,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_bind_group(0, &self.global_uniform_bind_group, &[]);

            let mut current_shader = None;

            for cmd in command_list {
                match cmd {
                    &DrawCommand::BindShader(shader) => {
                        match shader {
                            ui::draw_command::Shader::Shape => {
                                rpass.set_pipeline(&self.color_pipeline);
                            }
                            ui::draw_command::Shader::ShapeClip => todo!(),
                            ui::draw_command::Shader::Texture => {
                                rpass.set_pipeline(&self.texture_pipeline);
                            }
                            ui::draw_command::Shader::TextGrayscale => {
                                rpass.set_pipeline(&self.text_grayscale_pipeline);
                            }
                            ui::draw_command::Shader::TextSubpixel => {
                                rpass.set_pipeline(&self.text_subpixel_pipeline);
                            }
                        }

                        current_shader = Some(shader);
                    }
                    DrawCommand::DrawMesh(mesh_id) => {
                        let mesh_data = &mesh_data[mesh_id.0];

                        match current_shader.expect("invalid DrawMesh without a previous BindShader") {
                            ui::draw_command::Shader::Shape => {
                                rpass.set_vertex_buffer(
                                    0,
                                    mesh_data.vertex_buffers[&VertexAttribute::Position].slice(..),
                                );
                                rpass.set_vertex_buffer(1, mesh_data.vertex_buffers[&VertexAttribute::Color].slice(..));
                            }
                            ui::draw_command::Shader::ShapeClip => todo!(),
                            ui::draw_command::Shader::Texture
                            | ui::draw_command::Shader::TextGrayscale
                            | ui::draw_command::Shader::TextSubpixel => {
                                rpass.set_vertex_buffer(
                                    0,
                                    mesh_data.vertex_buffers[&VertexAttribute::Position].slice(..),
                                );
                                rpass.set_vertex_buffer(1, mesh_data.vertex_buffers[&VertexAttribute::Uv].slice(..));
                                rpass.set_vertex_buffer(2, mesh_data.vertex_buffers[&VertexAttribute::Color].slice(..));

                                if let Some(texture_id) = mesh_data.image_id {
                                    let texture = self.textures.get(&texture_id).expect("TODO");
                                    rpass.set_bind_group(1, &texture.bind_group, &[]);
                                }
                            }
                        }

                        rpass.set_stencil_reference(0);
                        rpass.set_index_buffer(mesh_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        rpass.draw_indexed(0..mesh_data.index_count, 0, 0..1);
                    }
                }
            }
        }

        // Postprocess render pass
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

            rpass.set_pipeline(&self.postprocess_pipeline);
            rpass.set_bind_group(0, &self.postprocess_bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn create_msaa_framebuffer(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("msaa framebuffer"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: MSAA_SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_msaa_depthbuffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("msaa depthbuffer"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: MSAA_SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_STENCIL_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_target_framebuffer(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("target framebuffer"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("target framebuffer view"),
                ..Default::default()
            })
    }

    fn create_postprocess_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_framebuffer_view: &wgpu::TextureView,
        target_framebuffer_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("postprocess bind group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(target_framebuffer_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(target_framebuffer_sampler),
                },
            ],
        })
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
        buffer_offsets: HashMap<VertexAttribute, u64>,
        first_index: u32,
        index_count: u32,
    },
    TextGlyph {
        buffer_offsets: HashMap<VertexAttribute, u64>,
        first_index: u32,
        text_color: Color,
        image_id: ImageId,
        pixel_mode: GlyphPixelMode,
    },
}

struct MeshData {
    vertex_buffers: HashMap<VertexAttribute, wgpu::Buffer>,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    image_id: Option<ImageId>,
}
