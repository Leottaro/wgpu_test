struct VertexFragmentData {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
}

@vertex
fn vertex_main(@builtin(vertex_index) i: u32) -> VertexFragmentData {
    var positions = array<vec2f, 3>(
        vec2f(-0.75, -0.75),
        vec2f(0.75, -0.75),
        vec2f(0.0, 0.75),
    );

    var colors = array<vec3f, 3>(
        vec3f(1.0, 0.0, 0.0),
        vec3f(0.0, 1.0, 0.0),
        vec3f(0.0, 0.0, 1.0),
    );

    var out: VertexFragmentData;
    out.position = vec4f(positions[i], 0.0, 1.0);
    out.color = colors[i];
    return out;
}

@fragment
fn fragment_main(frag_data: VertexFragmentData) -> @location(0) vec4f {
    return vec4(frag_data.color, 1.0);
}