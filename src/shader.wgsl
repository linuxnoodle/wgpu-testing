struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct RotationUniform {
    rot_mat: mat4x4<f32>,
}

struct DataUniform {
    iFrame: u32,
    iTimeDelta: f32,
    iTime: f32,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> rotation: RotationUniform;

@group(3) @binding(0)
var<uniform> data: DataUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

@vertex 
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32> (
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * rotation.rot_mat * vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return textureSample(t_diffuse, s_diffuse, in.tex_coords);
     return vec4(0.5 + 0.5 * cos(data.iTime + in.tex_coords.x),
                 0.5 + 0.5 * cos(data.iTime + in.tex_coords.y + 2.0),
                 0.5 + 0.5 * cos(data.iTime + in.tex_coords.x + 4.0),
                 1.0
            ) * textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
