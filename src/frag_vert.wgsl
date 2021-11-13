[[block]]
struct HelperData {
    maxParticles: u32;
    srcLen: u32;
    dstLen: u32;
    idx: u32;
};

[[block]]
struct Uniforms {
    paused: u32;
    mouse_down: u32;
    mouse_pos_last: vec3<f32>;
};

struct Particle {
    pos : vec3<f32>;
    vel : vec3<f32>;
    col : vec4<f32>; // color+brightness
    lifetime : f32;
};

struct VertexOut {
    [[location(0)]] col: vec3<f32>;
    [[builtin(position)]] pos: vec4<f32>;
};

[[block]]
struct Particles {
    particles : [[stride(64)]] array<Particle>;
};

[[group(0), binding(0)]] var<storage, read> particlesSrc : Particles;
// should this be in bind_group 1?
[[group(1), binding(0)]] var<storage, read> helperData : HelperData;
[[group(1), binding(1)]] var<uniform> uniforms : Uniforms;


[[stage(vertex)]]
fn main([[builtin(vertex_index)]] idx: u32) -> VertexOut {
    let obj_idx = u32(floor(f32(idx) / 3.0));
    let rel_idx = idx % 3u;

    let p = particlesSrc.particles[obj_idx];

    var vertex = vec3<f32>(0.0, 0.0, p.pos.z);
    if (rel_idx == 0u ) {
        vertex.x = f32(p.pos.x);
        vertex.y = f32(p.pos.y + (0.02 * (1.0 / p.pos.z)));
//        vertex.y = f32(p.pos.y + (0.5));
    } elseif (rel_idx == 1u) {
        vertex.x = f32(p.pos.x - (0.02 * (1.0 / p.pos.z)));
//        vertex.x = f32(p.pos.x - (0.5));
        vertex.y = f32(p.pos.y);
    } elseif (rel_idx == 2u) {
        vertex.x = f32(p.pos.x + (0.02 * (1.0 / p.pos.z)));
//        vertex.x = f32(p.pos.x + (0.5));
        vertex.y = f32(p.pos.y);
    }

    return VertexOut(p.col.xyz, vec4<f32>(vertex, 1.0));
}


[[stage(fragment)]]
fn main(idkbro: VertexOut) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
