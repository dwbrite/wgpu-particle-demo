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
    particles : [[stride(64)]] array<Particle>;
};

[[group(0), binding(0)]] var<storage, read> particlesSrc : Particles;
[[group(0), binding(1)]] var<storage, read_write> particlesDst : Particles;
// should this be in bind_group 1?
[[group(1), binding(0)]] var<storage, read_write> helperData : HelperData;
[[group(1), binding(1)]] var<uniform> uniforms : Uniforms;


fn add_particle(particle: Particle) {
    let idx = atomicLoad(&helperData.idx);
    if (idx >= helperData.maxParticles) {
        return;
    }

    particlesDst.particles[idx] = particle;
    let tmp = atomicAdd(&helperData.idx, 1u);
//    abc();
//    helperData.idx = helperData.idx + 1u;
}

[[stage(compute), workgroup_size(64)]]
fn step_particles([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    var particle = particlesSrc.particles[atomicLoad(&helperData.idx)];

//    if (particle.lifetime <= 0.0) {
//        return;
//    }

    // TODO: calculate particle new position

    // calculate friction
//    particle.vel = vec3<f32>(particle.vel.x * 0.99998, particle.vel.y * 0.99998, particle.vel.z * 0.99998);
//    // then calculate acceleration towards mouse
//    if (uniforms.mouse_down == 1u) {
//        let a = particle.pos;
//        let b = uniforms.mouse_pos_last;
//        let dist_parts = a - b;
//
//        // then calculate the new velocity
//        let diff = a - b;
//        let g = 5.0;
//        // what the fuck is this??
//        let tmp = normalize(diff) / pow(diff, vec3<f32>(2.0));
//        particle.vel = particle.vel + vec3<f32>(tmp * g);
//    }
//    particle.pos = particle.pos + particle.vel;

    add_particle(particle);
}


[[stage(compute), workgroup_size(64)]]
fn emit([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    // TODO: emit particles in a spiral, parallel to the view plane
    let p = Particle(
        vec3<f32>(0.1, -0.1, 0.4),
        vec3<f32>(2.0, 0.5, 0.8),
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        1000.0,
    );

    add_particle(p);
}


[[stage(compute), workgroup_size(1)]]
fn swap() {
    helperData.srcLen = atomicLoad(&helperData.idx);
    atomicStore(&helperData.idx, 0u);
}

