use crate::gfx_ctx::GraphicsContext;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ComputePipeline, ComputePipelineDescriptor,
    FragmentState, FrontFace, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexState,
};

/*
see: https://sotrh.github.io/learn-wgpu/showcase/windowless/#getting-data-out-of-a-buffer

particles will also need a bool to determine if they are dead or not :thinking:
*/

pub const MAX_PARTICLES: u32 = 2_000_000;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    paused: u8,
    // TODO: camera
    // TODO: mouse position (vec3?)
}

pub struct RenderStuff {
    pub shaders: ShaderModule,
    pub shared: Shared,
    pub compute: Compute,
    pub render: Render,
}

pub struct Compute {
    pub particle_swapchain: [Buffer; 2],

    pub bind_groups: [BindGroup; 2],
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
        // these two buffers act as a swapchain for the particle buffers
        // basically we read from one, then calc and write the new data to the next buffer.
        // then the buffers are "swapped" so that the new becomes the old and vice-versa
        let particle_swapchain = [
            gc.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Particle Buffer 0"),
                contents: &vec![0u8; MAX_PARTICLES as usize * 64],
                usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            }),
            gc.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Particle Buffer 1"),
                contents: &vec![0u8; MAX_PARTICLES as usize * 64],
                usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            }),
        ];

        let compute_bind_group_layout =
            gc.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let compute_bind_groups = [
            gc.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &compute_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: particle_swapchain[0].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: particle_swapchain[1].as_entire_binding(),
                    },
                ],
            }),
            gc.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &compute_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: particle_swapchain[1].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: particle_swapchain[0].as_entire_binding(),
                    },
                ],
            }),
        ];

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
            particle_swapchain,
            bind_groups: compute_bind_groups,
            bind_group_layout: compute_bind_group_layout,
            emit_pipeline,
            compute_pipeline,
        }
    }
}

pub struct Render {
    pub render_pipeline: RenderPipeline,
}

impl Render {
    fn new(
        gc: &mut GraphicsContext,
        shaders: &ShaderModule,
        _shared_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let render_pipeline = gc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(
                &gc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
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
                clamp_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shaders,
                entry_point: "main",
                targets: &[],
            }),
        });

        Render { render_pipeline }
    }
}

pub struct Shared {
    pub helper_data: Buffer,
    pub uniforms: Buffer,
    pub compute_bind_layout: BindGroupLayout,
    pub render_bind_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl Shared {
    fn new(gc: &mut GraphicsContext) -> Self {
        let helper_data = gc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("helper data for compute shaders"),
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            contents: bytemuck::bytes_of(&HelperData {
                max_particles: MAX_PARTICLES,
                dst_len: 0,
                src_len: 0,
                idx: 0,
            }),
        });

        let uniforms = gc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera + mouse data uniforms"),
            contents: bytemuck::bytes_of(&Uniforms { paused: 0 }),
            usage: BufferUsages::UNIFORM,
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

        let shared_bind_group = gc.device.create_bind_group(&BindGroupDescriptor {
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

        Shared {
            helper_data,
            uniforms,
            compute_bind_layout: shared_compute_bind_layout,
            render_bind_layout: shared_render_bind_layout,
            bind_group: shared_bind_group,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct HelperData {
    max_particles: u32,
    dst_len: u32,
    src_len: u32,
    idx: u32,
}

impl RenderStuff {
    pub fn new(gc: &mut GraphicsContext) -> RenderStuff {
        // using one shader module to keep things simple
        let shaders = gc.device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("particle demo shaders"),
            source: ShaderSource::Wgsl(include_str!("idk.wgsl").into()),
        });

        let shared = Shared::new(gc);
        let compute = Compute::new(gc, &shaders, &shared.compute_bind_layout);
        let render = Render::new(gc, &shaders, &shared.render_bind_layout);

        RenderStuff {
            shaders,
            shared,
            compute,
            render,
        }
    }
}
