

[[block]]
struct HelperData {
    srcLen: u32;
    dstLen: u32;
    idx: u32;
};

struct Particle {
  pos : vec3<f32>;
  vel : vec3<f32>;
  col : vec4<f32>; // color+brightness
  lifetime: f32;
};

[[block]]
struct Particles {
  particles : [[stride(64)]] array<Particle>;
};

[[group(0), binding(0)]] var<storage, read> particlesSrc : Particles;
[[group(0), binding(1)]] var<storage, read_write> particlesDst : Particles;
[[group(0), binding(2)]] var<storage, read_write> helperData : HelperData;


[[stage(compute), workgroup_size(64)]]
fn emit([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    // emits particles, adding them to particlesDst
    let idx = global_invocation_id.x;
    let mouse_location = vec3<f32>(0.0);

    let new_idx = helperData.dstLen + idx;

    if (helperData.dstLen >= 20000u) {
        return;
    }

    particlesDst.particles[new_idx] = Particle(
        vec3<f32>(1.0, 0.4, 0.7),
        vec3<f32>(2.0, 0.5, 0.8),
        vec4<f32>(3.0, 0.6, 0.9, 0.0),
        5.0,
    );

    helperData.dstLen = helperData.dstLen + 1u;

    // on the last emission, swap length
    if (idx == 4999u) {
        helperData.srcLen = helperData.dstLen;
        helperData.dstLen = 0u;
    }
}


[[stage(compute), workgroup_size(64)]]
fn step_particles() {
    // do nothing
    helperData.idx = helperData.idx + 1u;
    helperData.idx = helperData.idx - 1u;
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
