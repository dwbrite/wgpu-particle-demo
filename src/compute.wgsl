[[block]]
struct HelperData {
    maxParticles: u32;
    srcLen: u32;
    dstLen: atomic<u32>;
    idx: atomic<u32>;
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
    [[location(0)]] pos: vec3<f32>;
    [[location(1)]] col: vec3<f32>;
};

[[block]]
struct Particles {
    group : array<[[stride(64)]] array<Particle, 256>>;
};

[[group(0), binding(0)]] var<storage, read_write> particlesSrc : Particles;
// should this be in bind_group 1?
[[group(1), binding(0)]] var<storage, read_write> helperData : HelperData;
[[group(1), binding(1)]] var<uniform> uniforms : Uniforms;


[[stage(compute), workgroup_size(64, 1, 1)]]
fn step_particles([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    for(var y: i32 = 0; y < 256; y = y + 1) {
        let particle: ptr<storage, Particle, read_write> = &particlesSrc.group[global_invocation_id.x][y];

        if ((*particle).lifetime <= 0.0) {
            continue;
        }

        // physic :)
        (*particle).lifetime = (*particle).lifetime - 0.16;
    }
}

[[stage(compute), workgroup_size(1)]]
fn emit([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
//    let p = Particle(
//        vec3<f32>(0.1, -0.1, 0.4),
//        vec3<f32>(2.0, 0.5, 0.8),
//        vec4<f32>(1.0, 0.0, 0.0, 0.0),
//        1000.0,
//    );
//
//    add_particle(p);
}
