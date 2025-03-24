#include assets/shaders/lib/header.wgsl

struct Vertex {
    position: vec2<f32>,
    uv: vec2<f32>,
    texcoord: vec2<f32>,
}

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    var vertices = array<Vertex, 4>(
        Vertex(vec2(-1.0, -1.0), vec2(0.0, 1.0), vec2(0.0, 0.0)),
        Vertex(vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(1.0, 0.0)),
        Vertex(vec2(1.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0)),
        Vertex(vec2(-1.0, 1.0), vec2(0.0, 0.0), vec2(0.0, 1.0))
    );

    var indices = array<u32, 6>(0, 1, 2, 0, 2, 3);

    let index = indices[vertex_index];
    let vertex = vertices[index];
    
    out.clip_position = vec4(vertex.position.xy, 0.0, 1.0);
    out.uv = vertex.uv;
    out.texcoord = vertex.texcoord;

    return out;
}