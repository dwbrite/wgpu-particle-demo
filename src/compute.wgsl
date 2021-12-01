[[block]]
struct HelperData {
    maxParticles: u32;
};

[[block]]
struct Uniforms {
    paused: u32;
    mouse_down: u32;
    mouse_pos_last: vec2<f32>;
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


[[stage(compute), workgroup_size(256, 1, 1)]]
fn step_particles([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    for(var y: i32 = 0; y < 256; y = y + 1) {
        let particle: ptr<storage, Particle, read_write> = &particlesSrc.group[global_invocation_id.x][y];

        if ((*particle).lifetime < 0.0) {
            continue;
        }

        // physic :)
        (*particle).lifetime = (*particle).lifetime - 0.16;
        (*particle).pos = (*particle).pos + (*particle).vel;
        (*particle).vel = (*particle).vel * vec3<f32>(0.998); // friction
    }
}

[[stage(compute), workgroup_size(1)]]
fn emit([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let total_to_add = 512;
    var left_to_add = total_to_add;

    if (uniforms.mouse_down == 0u) {
        return;
    }

    for(var group: i32 = 0; group < i32(helperData.maxParticles) / 256; group = group + 1) {
        let particle: ptr<storage, Particle, read_write> = &particlesSrc.group[group][0];

        if ((*particle).lifetime > 0.0) {
            continue;
        }
        for (var p: i32 = 0; p < 256; p = p + 1) {
            let particle: ptr<storage, Particle, read_write> = &particlesSrc.group[group][p];

            (*particle).lifetime = 600.0;
            let x = cos(2.0*3.14159*(f32(left_to_add)/(f32(total_to_add))));
            let y = sin(2.0*3.14159*(f32(left_to_add)/(f32(total_to_add))));
            (*particle).pos = vec3<f32>(uniforms.mouse_pos_last, 0.5); // TODO: 3D transform mouse position based on camera

            // TODO: explode particles based on mouse velocity normal?
            (*particle).vel = vec3<f32>(x, y, 0.0) * vec3<f32>(0.002, 0.002, 0.0);

            left_to_add = left_to_add - 1;
            if (left_to_add == 0) {
                return;
            }
        }
    }
}
