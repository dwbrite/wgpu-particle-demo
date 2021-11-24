[[block]]
struct HelperData {
    maxParticles: u32;
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
    [[location(1)]] tex_coords: vec2<f32>;
    [[location(2)]] lifetime: f32;
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

[[group(2), binding(0)]] var r_color: texture_2d<u32>;
[[group(2), binding(1)]] var r_sampler: sampler;

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] idx: u32) -> VertexOut {
    let obj_idx = u32(floor(f32(idx) / 3.0));
    let rel_idx = idx % 3u;

    let p = particlesSrc.particles[obj_idx];

    var vertex = vec3<f32>(0.0, 0.0, p.pos.z);
    var tex_coord = vec2<f32>(0.0);

    if (p.lifetime <= 0.0) {
        return VertexOut(p.col.xyz, tex_coord, p.lifetime, vec4<f32>(vertex, -2.0));
    }

    // for the eventual 3D transforms... I'm going to need more brain power

    if (rel_idx == 0u ) {
        vertex.x = f32(p.pos.x);
        vertex.y = f32(p.pos.y + 0.0138564 / 2.0); // equilateral es importante
        tex_coord = vec2<f32>(0.5, -0.07);
    } elseif (rel_idx == 1u) {
        vertex.x = f32(p.pos.x - 0.008 / 2.0);
        vertex.y = f32(p.pos.y);
        tex_coord = vec2<f32>(0.0, 0.78);
    } elseif (rel_idx == 2u) {
        vertex.x = f32(p.pos.x + 0.008 / 2.0);
        vertex.y = f32(p.pos.y);
        tex_coord = vec2<f32>(1.0, 0.78);
    }

    return VertexOut(p.col.xyz, tex_coord, p.lifetime / 600.0, vec4<f32>(vertex, 1.0));
}


[[stage(fragment)]]
fn main(idkbro: VertexOut) -> [[location(0)]] vec4<f32> {
    //return textureSample(r_color, r_sampler, idkbro.tex_coords);

    let tex = textureLoad(r_color, vec2<i32>(idkbro.tex_coords * 512.0), 0);
    let brightness = f32(tex.x) / 255.0;

    return vec4<f32>(1.0 - idkbro.lifetime, 0.0, idkbro.lifetime, brightness * idkbro.lifetime * idkbro.lifetime);
}
