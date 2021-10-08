use wgpu::{BindGroup, BindGroupLayout, RenderPipeline, ComputePipeline, ShaderModule};

struct RenderStuff {
    shader: ShaderModule,
    compute_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}