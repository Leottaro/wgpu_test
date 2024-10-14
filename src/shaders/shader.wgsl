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
fn vertex_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    out.position = camera.view_proj * vec4<f32>(vertex.position, 1.0);
    return out;
}

@fragment
fn fragment_main(frag_data: VertexOutput) -> @location(0) vec4f {
    return textureSample(my_texture, my_sampler, frag_data.tex_coords);
}