use crate::gfx_ctx::GraphicsContext;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState,
    Buffer, BufferBindingType, BufferUsages, ColorTargetState, ComputePipeline,
    ComputePipelineDescriptor, FragmentState, FrontFace, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, ShaderModule, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, TextureSampleType, TextureViewDimension, VertexState,
};

#[cfg(not(target_arch = "wasm32"))]
pub const MAX_PARTICLES: u32 = 1_048_576;

#[cfg(target_arch = "wasm32")]
pub const MAX_PARTICLES: u32 = 393_216;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    // no bools in Pod, and bools don't have a known in-memory representation
    // so we need to use u32 in its place.
    pub(crate) paused: u32,
    pub(crate) mouse_down: u32,
    pub(crate) mouse_pos_last: [f32; 2],
    // make sure we stay 16 byte aligned, especially when using arrays
    // TODO: camera
}

pub struct RenderStuff {
    pub shaders: ShaderModule,
    pub shared: Shared,
    pub compute: Compute,
    pub render: Render,
}

pub struct Compute {
    pub particle_buffer: Buffer,

    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,

    pub emit_pipeline: ComputePipeline,
    pub compute_pipeline: ComputePipeline,
}

impl Compute {
    fn new(
        gc: &mut GraphicsContext,
        shaders: &ShaderModule,
        shared_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let particle_buffer = gc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Particle Buffer 0"),
            contents: &vec![0u8; MAX_PARTICLES as usize * 64],
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
        });

        let compute_bind_group_layout =
            gc.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let compute_bind_group = gc.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: particle_buffer.as_entire_binding(),
            }],
        });

        let compute_pipeline = gc
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: Some(
                    &gc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[
                            &compute_bind_group_layout,
                            &shared_bind_group_layout,
                        ],
                        push_constant_ranges: &[],
                    }),
                ),
                module: &shaders,
                entry_point: "step_particles",
            });

        let emit_pipeline = gc
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: Some(
                    &gc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[
                            &compute_bind_group_layout,
                            &shared_bind_group_layout,
                        ],
                        push_constant_ranges: &[],
                    }),
                ),
                module: &shaders,
                entry_point: "emit",
            });

        Compute {
            particle_buffer,
            bind_group: compute_bind_group,
            bind_group_layout: compute_bind_group_layout,
            emit_pipeline,
            compute_pipeline,
        }
    }
}

pub struct Render {
    pub render_pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub texture_bind_group: BindGroup,
}

impl Render {
    fn new(
        gc: &mut GraphicsContext,
        shaders: &ShaderModule,
        shared_render_bgl: &BindGroupLayout,
        particle_buffer: &Buffer,
    ) -> Self {
        let diffuse_bytes = include_bytes!("particle.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_r = diffuse_image.as_luma8().unwrap();

        let texture_extent = wgpu::Extent3d {
            width: 512,
            height: 512,
            depth_or_array_layers: 1,
        };

        let texture = gc.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        gc.queue.write_texture(
            texture.as_image_copy(),
            diffuse_r,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::num::NonZeroU32::new(512).unwrap()),
                rows_per_image: None,
            },
            texture_extent,
        );

        let texture_sampler = gc.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // mag_filter: wgpu::FilterMode::Linear,
            // min_filter: wgpu::FilterMode::Nearest,
            // mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            gc.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Uint,
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });

        let texture_bind_group = gc.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let render_bind_group_layout =
            gc.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = gc.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &render_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: particle_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline = gc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(
                &gc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &render_bind_group_layout,
                        &shared_render_bgl,
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: VertexState {
                module: &shaders,
                entry_point: "main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: gc.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shaders,
                entry_point: "main",
                targets: &[ColorTargetState {
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::DstAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent::OVER,
                    }),
                    ..gc.config.format.into()
                }],
            }),
            multiview: None,
        });

        Render {
            render_pipeline,
            bind_group,
            texture_bind_group,
        }
    }
}

pub struct Shared {
    pub helper_data: Buffer,
    pub uniforms: Buffer,
    pub compute_bind_layout: BindGroupLayout,
    pub render_bind_layout: BindGroupLayout,
    pub compute_bind_group: BindGroup,
    pub render_bind_group: BindGroup,
}

impl Shared {
    fn new(gc: &mut GraphicsContext) -> Self {
        let helper_data = gc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("helper data for compute shaders"),
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            contents: bytemuck::bytes_of(&HelperData {
                max_particles: MAX_PARTICLES,
            }),
        });

        let uniforms = gc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera + mouse data uniforms"),
            contents: bytemuck::cast_slice(&[Uniforms {
                paused: 0,
                mouse_down: 0,
                mouse_pos_last: [0.0, 0.0],
            }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let shared_compute_bind_layout =
            gc.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::all(),
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let shared_render_bind_layout =
            gc.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::all(),
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::all(),
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let compute_bind_group = gc.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &shared_compute_bind_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: helper_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: uniforms.as_entire_binding(),
                },
            ],
        });

        let render_bind_group = gc.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &shared_render_bind_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: helper_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: uniforms.as_entire_binding(),
                },
            ],
        });

        Shared {
            helper_data,
            uniforms,
            compute_bind_layout: shared_compute_bind_layout,
            render_bind_layout: shared_render_bind_layout,
            compute_bind_group,
            render_bind_group,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct HelperData {
    max_particles: u32,
}

impl RenderStuff {
    pub fn new(gc: &mut GraphicsContext) -> RenderStuff {
        let compute_shaders = gc.device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("particle demo shaders"),
            source: ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
        });

        let render_shaders = gc.device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("particle demo shaders"),
            source: ShaderSource::Wgsl(include_str!("frag_vert.wgsl").into()),
        });

        let shared = Shared::new(gc);
        let compute = Compute::new(gc, &compute_shaders, &shared.compute_bind_layout);

        let render = Render::new(
            gc,
            &render_shaders,
            &shared.render_bind_layout,
            &compute.particle_buffer,
        );

        RenderStuff {
            shaders: compute_shaders,
            shared,
            compute,
            render,
        }
    }
}
