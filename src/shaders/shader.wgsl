@group(0) @binding(0) var my_texture: texture_2d<f32>;
@group(0) @binding(1) var my_sampler: sampler;

struct Vertex {
	@location(0) position: vec3f,
	@location(1) color: vec3f,
}

struct VertexFragmentData {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
	@location(1) texcoord: vec2f,
}

@vertex
fn vertex_main(vertex: Vertex) -> VertexFragmentData {
    var out: VertexFragmentData;
    out.position = vec4f(vertex.position, 1.0);
    out.color = vertex.color;
    out.texcoord = vec2f(vertex.position.x, -vertex.position.y);
    return out;
}

@fragment
fn fragment_main(frag_data: VertexFragmentData) -> @location(0) vec4f {
    return textureSample(my_texture, my_sampler, frag_data.texcoord);
}