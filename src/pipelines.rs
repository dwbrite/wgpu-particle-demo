use crate::gfx_ctx::GraphicsContext;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ComputePipeline,
    ComputePipelineDescriptor, PipelineLayoutDescriptor, ShaderModule, ShaderModuleDescriptor,
    ShaderSource, ShaderStages,
};
use std::thread;
use std::time::Duration;

pub const MAX_PARTICLES: u64 = 100;

pub struct RenderStuff {
    pub particle_swapchain: [Buffer; 2],

    pub bind_group_layout: BindGroupLayout,
    pub bind_group_swapchain: [BindGroup; 2],

    pub shaders: ShaderModule,
    pub emit_pipeline: ComputePipeline,
    pub compute_pipeline: ComputePipeline,
}

impl RenderStuff {
    pub fn new(gc: &mut GraphicsContext) -> RenderStuff {
        // using one shader module to keep things simple
        let shaders = gc.device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("particle demo shaders"),
            source: ShaderSource::Wgsl(include_str!("idk.wgsl").into()),
        });

        // these two buffers act as a swapchain for the particle buffers
        // basically we read from one, then calc and write the new data to the next buffer.
        // then the bind groups are "swapped" so that the source buffer becomes the destination and vice versa
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

        let bind_groups = [
            gc.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0, // 0:0 is read only
                        resource: particle_swapchain[0].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1, // 1:1 is read/write
                        resource: particle_swapchain[1].as_entire_binding(),
                    },
                ],
            }),
            gc.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0, // now 0:1 is read only
                        resource: particle_swapchain[1].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1, // and 1:0 is read/write
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

        RenderStuff {
            particle_swapchain,
            shaders,
            emit_pipeline,
            compute_pipeline,
            bind_group_layout,
            bind_group_swapchain: bind_groups,
        }
    }
}
