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
    particlesDst.particles[helperData.idx] = particle;
    helperData.idx = helperData.idx + 1u;
}

[[stage(compute), workgroup_size(64)]]
fn step_particles([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let idx = global_invocation_id.x;

    if (idx >= helperData.srcLen) {
        return;
    }

    var particle = particlesSrc.particles[idx];
    if (particle.lifetime <= 0.0) {
        return;
    }

    // TODO: calculate particle new position

    // calculate friction
    particle.vel = vec3<f32>(particle.vel.x * 0.97, particle.vel.y * 0.97, particle.vel.z * 0.97);
    // then calculate acceleration towards mouse
    if (uniforms.mouse_down == 1u) {
        let a = particle.pos;
        let b = uniforms.mouse_pos_last;
        let distance = distance(a, b);
        let dist_parts = a - b;

        // then calculate the new velocity
        let diff = a - b;
        let g = 5.0;
        // what the fuck is this??
        let tmp = normalize(diff) / pow(diff, vec3<f32>(2.0));
        particle.vel = particle.vel + vec3<f32>(tmp * g);
    }
    particle.pos = particle.pos + particle.vel;
    add_particle(particle);
}


[[stage(compute), workgroup_size(64)]]
fn emit([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let idx = helperData.dstLen + global_invocation_id.x;
    if (idx >= helperData.maxParticles) {
        return;
    }

    // TODO: emit particles in a spiral, parallel to the view plane
    // TODO: only emit when mouse pressed? that could probably be done cpu side

    particlesDst.particles[idx] = Particle(
        vec3<f32>(0.5, 0.4, 0.7),
        vec3<f32>(2.0, 0.5, 0.8),
        vec4<f32>(3.0, 0.6, 0.9, 0.0),
        5.0,
    );

    helperData.dstLen = helperData.dstLen + 1u;

    // on the last emission, swap length
    if (idx == 4999u) {
        helperData.srcLen = helperData.dstLen;
        helperData.dstLen = 0u;
        helperData.idx = 0u;
    }
}

[[stage(vertex)]]
fn main() -> [[builtin(position)]] vec4<f32> {
    // create a billboard from the position
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}


[[stage(fragment)]]
fn main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
