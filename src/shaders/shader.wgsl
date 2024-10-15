struct InstanceInput {
    @location(5) vertex_matrix_0: vec4<f32>,
    @location(6) vertex_matrix_1: vec4<f32>,
    @location(7) vertex_matrix_2: vec4<f32>,
    @location(8) vertex_matrix_3: vec4<f32>,
	@location(9) position: vec3<f32>,
	@location(10) scale: f32,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var my_texture: texture_2d<f32>;
@group(1) @binding(1) var my_sampler: sampler;

struct VertexInput {
	@location(0) position: vec3f,
	@location(1) tex_coords: vec2f,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
	@location(0) tex_coords: vec2f,
}

@vertex
fn vertex_main(vertex: VertexInput,
    instance: InstanceInput) -> VertexOutput {
    let instance_transform = mat4x4<f32>(
        instance.vertex_matrix_0,
        instance.vertex_matrix_1,
        instance.vertex_matrix_2,
        instance.vertex_matrix_3,
    );
    let instance_view_proj = camera.view_proj * instance_transform;

    let scaled_position = (vertex.position - instance.position) * instance.scale + instance.position * instance.scale;
    let projected_position = instance_view_proj * vec4<f32>(scaled_position, 1.0);

    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    out.position = projected_position;
    return out;
}

@fragment
fn fragment_main(frag_data: VertexOutput) -> @location(0) vec4f {
    return textureSample(my_texture, my_sampler, frag_data.tex_coords);
}