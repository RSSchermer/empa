struct VertexIn {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>
}

struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vert_main(vertex: VertexIn) -> VertexOut {
    var result = VertexOut();

    result.position = uniforms.projection * uniforms.view * uniforms.model * vertex.position;
    result.color = vertex.color;

    return result;
}

@fragment
fn frag_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
