use crate::gfx_ctx::GraphicsContext;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferUsages, ComputePipeline, ComputePipelineDescriptor, FragmentState,
    FrontFace, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexState,
};

const MAX_PARTICLES: u64 = 1_000_000;

struct RenderStuff {
    particle_swapchain: [Buffer; 2],
    bind_groups: [BindGroup; 2],

    shaders: ShaderModule,
    compute_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
}

impl RenderStuff {
    fn new(gc: &mut GraphicsContext) -> RenderStuff {
        // using one shader module to keep things simple
        let shaders = gc.device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("particle demo shaders"),
            source: ShaderSource::Wgsl(include_str!("idk.wgsl").into()),
        });

        // these two buffers act as a swapchain for the particle buffers
        // basically we read from one, then calc and write the new data to the next buffer.
        // then the buffers are "swapped" so that the new becomes the old and vice-versa
        let particle_swapchain = [
            gc.device.create_buffer(&BufferDescriptor {
                label: Some("Particle Buffer 0"),
                size: MAX_PARTICLES * 48,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            gc.device.create_buffer(&BufferDescriptor {
                label: Some("Particle Buffer 0"),
                size: MAX_PARTICLES * 48,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
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

        // (D)RY :^))
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
                ],
            }),
            gc.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 1,
                        resource: particle_swapchain[1].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 0,
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
                        bind_group_layouts: &[&bind_group_layout],
                        push_constant_ranges: &[],
                    }),
                ),
                module: &shaders,
                entry_point: "main",
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
            compute_pipeline,
            render_pipeline,
            bind_group_layout,
            bind_groups,
        }
    }
}
