@group(0) @binding(0)
var<storage, read_write> data: array<atomic<u32>>;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    for (var i = 0u; i < arrayLength(&data); i++) {
        atomicAdd(&data[i], 1);
    }
}
