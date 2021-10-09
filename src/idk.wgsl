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

[[group(0), binding(0)]] var<storage, read_write> particlesSrc : Particles;
[[group(0), binding(1)]] var<storage, read_write> particlesDst : Particles;


[[stage(compute), workgroup_size(64)]]
fn emit([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let idx = global_invocation_id.x;

    // even adding only 10 particles will cause a crash
    if (idx < 10u && idx >= 0u) {
        particlesDst.particles[idx] = Particle(
            vec3<f32>(1.0, 0.4, 0.7),
            vec3<f32>(2.0, 0.5, 0.8),
            vec4<f32>(3.0, 0.6, 0.9, 0.0),
            5.0,
        );
    }
}


[[stage(compute), workgroup_size(64)]]
fn step_particles() {
    // do literally nothing
}