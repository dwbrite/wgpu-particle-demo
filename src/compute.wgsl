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

[[group(0), binding(0)]] var<storage, read> particlesSrc : Particles;
[[group(0), binding(1)]] var<storage, read_write> particlesDst : Particles;
// should this be in bind_group 1?
[[group(1), binding(0)]] var<storage, read_write> helperData : HelperData;
[[group(1), binding(1)]] var<uniform> uniforms : Uniforms;


fn add_particle(particle: Particle) {
//    let idx = atomicAdd(&helperData.idx, 1u);
//    if (idx >= helperData.maxParticles) {
//        return;
//    }

//    particlesDst.particles[idx] = particle;
}

// TODO: render pass for merging particle groups

[[stage(compute), workgroup_size(1)]]
fn step_particles([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    var group = particlesSrc.group[global_invocation_id.x];

    let len = 256;
    var src_idx = 0;
    var dst_idx = 0;
    loop {
        if (src_idx >= len) {
            break;
        }

        var particle = group[src_idx];
        // src_idx++ since we're done using it
        src_idx = src_idx + 1;

        // do physics on a particle
        if (particle.lifetime <= 0.0) {
            continue;
        }

        // then add it to dstGroup
        particlesDst.group[global_invocation_id.x][dst_idx] = particle;

        dst_idx = dst_idx + 1;
    }
}

// this could be done with dispatch_indirect
[[stage(compute), workgroup_size(128)]]
fn sort_particles([[builtin(global_invocation_id)]] invocation: vec3<u32>) {
    // typical parallel sorting algorithms are difficult to perform on a gpu.
    // this is due to them usually requiring a dynamic number of threads,
    // or needing to run multiple times sequentially.

    // let's try some fucked up variant of merge sort
    // basically, split the 4096* items we need to sort into groups of 32
    // that's 128 groups of 32
    // from there, sort each group (quicksort maybe?
    // then merge sort the groups with a single thread

    // maybe insertion sort isn't a terrible idea for an array that's only len 32
    // since it's parallelized anyway... :thinking:
}


[[stage(compute), workgroup_size(256)]]
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


[[stage(compute), workgroup_size(1)]]
fn swap() {
//    atomicStore(&helperData.idx, 0u);
}

