struct Vertex {
	@location(0) position: vec3f,
	@location(1) color: vec3f,
}

struct VertexFragmentData {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
}

@vertex
fn vertex_main(vertex: Vertex) -> VertexFragmentData {
    var out: VertexFragmentData;
    out.position = vec4f(vertex.position, 1.0);
    out.color = vertex.color;
    return out;
}

@fragment
fn fragment_main(frag_data: VertexFragmentData) -> @location(0) vec4f {
    return vec4(frag_data.color, 1.0);
}