struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>
}

@group(0) @binding(0)
var<uniform> scale: f32;

@vertex
fn vert_main(vertex: VertexIn) -> VertexOut {
    var result = VertexOut();

    result.position = vec4(scale * vertex.position, 0.0, 1.0);
    result.color = vertex.color;

    return result;
}

@fragment
fn frag_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
