// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct InstanceInput {
    @location(5) x_axis: vec4<f32>,
    @location(6) y_axis: vec4<f32>,
    @location(7) z_axis: vec4<f32>,
    @location(8) w_axis: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32> (
        instance.x_axis,
        instance.y_axis,
        instance.z_axis,
        instance.w_axis,
    );
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
