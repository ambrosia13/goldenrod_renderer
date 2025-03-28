struct Vertex_0
{
     position_0 : vec2<f32>,
     uv_0 : vec2<f32>,
     texcoord_0 : vec2<f32>,
};

fn Vertex_x24init_0( position_1 : vec2<f32>,  uv_1 : vec2<f32>,  texcoord_1 : vec2<f32>) -> Vertex_0
{
    var _S1 : Vertex_0;
    _S1.position_0 = position_1;
    _S1.uv_0 = uv_1;
    _S1.texcoord_0 = texcoord_1;
    return _S1;
}

struct VertexOutput_0
{
    @builtin(position) clipPosition_0 : vec4<f32>,
    @location(0) uv_2 : vec2<f32>,
    @location(1) texcoord_2 : vec2<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) vertexIndex_0 : u32) -> VertexOutput_0
{
    const _S2 : vec2<f32> = vec2<f32>(0.0f, 1.0f);
    const _S3 : vec2<f32> = vec2<f32>(0.0f, 0.0f);
    const _S4 : vec2<f32> = vec2<f32>(1.0f, 1.0f);
    const _S5 : vec2<f32> = vec2<f32>(1.0f, 0.0f);
    var vertices_0 : array<Vertex_0, i32(4)> = array<Vertex_0, i32(4)>( Vertex_x24init_0(vec2<f32>(-1.0f, -1.0f), _S2, _S3), Vertex_x24init_0(vec2<f32>(1.0f, -1.0f), _S4, _S5), Vertex_x24init_0(_S4, _S5, _S4), Vertex_x24init_0(vec2<f32>(-1.0f, 1.0f), _S3, _S2) );
    var indices_0 : array<u32, i32(6)> = array<u32, i32(6)>( u32(0), u32(1), u32(2), u32(0), u32(2), u32(3) );
    var output_0 : VertexOutput_0;
    output_0.clipPosition_0 = vec4<f32>(vertices_0[indices_0[vertexIndex_0]].position_0.xy, 0.0f, 1.0f);
    output_0.uv_2 = vertices_0[indices_0[vertexIndex_0]].uv_0;
    output_0.texcoord_2 = vertices_0[indices_0[vertexIndex_0]].texcoord_0;
}

