use crate::gfx_ctx::GraphicsContext;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferSize, BufferUsages, ComputePipeline, ComputePipelineDescriptor,
    FragmentState, FrontFace, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexState,
};

/*
see: https://sotrh.github.io/learn-wgpu/showcase/windowless/#getting-data-out-of-a-buffer

particles will also need a bool to determine if they are dead or not :thinking:
*/

pub const MAX_PARTICLES: u64 = 1_000_000;

pub struct RenderStuff {
    // should only bind_groups be a swapchain???
    pub particle_swapchain: [Buffer; 2],
    pub bind_groups: [BindGroup; 2],

    pub shaders: ShaderModule,
    pub emit_pipeline: ComputePipeline,
    pub compute_pipeline: ComputePipeline,
    pub render_pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BufferStuff {
    dst_len: u64,
    src_len: u64,
}

impl RenderStuff {
    pub fn new(gc: &mut GraphicsContext) -> RenderStuff {
        // using one shader module to keep things simple
        let shaders = gc.device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("particle demo shaders"),
            source: ShaderSource::Wgsl(include_str!("idk.wgsl").into()),
        });

        let shared_data_buffer = gc.device.create_buffer(&BufferDescriptor {
            label: Some("helper data for compute shaders"),
            size: 12,
            usage: BufferUsages::STORAGE
                | BufferUsages::COPY_DST
                | BufferUsages::COPY_SRC
                | BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        // these two buffers act as a swapchain for the particle buffers
        // basically we read from one, then calc and write the new data to the next buffer.
        // then the buffers are "swapped" so that the new becomes the old and vice-versa
        let particle_swapchain = [
            gc.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Particle Buffer 0"),
                contents: &vec![0u8; MAX_PARTICLES as usize * 64],
                usage: BufferUsages::STORAGE
                    | BufferUsages::COPY_DST
                    | BufferUsages::COPY_SRC
                    | BufferUsages::INDIRECT,
            }),
            gc.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Particle Buffer 1"),
                contents: &vec![0u8; MAX_PARTICLES as usize * 64],
                usage: BufferUsages::STORAGE
                    | BufferUsages::COPY_DST
                    | BufferUsages::COPY_SRC
                    | BufferUsages::INDIRECT,
            }),
        ];

        let bind_group_layout = gc
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(12),
                        },
                        count: None,
                    },
                ],
            });

        // (D)RY :^)
        let bind_groups = [
            gc.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: particle_swapchain[0].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: particle_swapchain[1].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: shared_data_buffer.as_entire_binding(),
                    },
                ],
            }),
            gc.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: particle_swapchain[1].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: particle_swapchain[0].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: shared_data_buffer.as_entire_binding(),
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
                        bind_group_layouts: &[&bind_group_layout],
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
                        bind_group_layouts: &[&bind_group_layout],
                        push_constant_ranges: &[],
                    }),
                ),
                module: &shaders,
                entry_point: "emit",
            });

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

        RenderStuff {
            particle_swapchain,
            shaders,
            emit_pipeline,
            compute_pipeline,
            render_pipeline,
            bind_group_layout,
            bind_groups,
        }
    }
}
