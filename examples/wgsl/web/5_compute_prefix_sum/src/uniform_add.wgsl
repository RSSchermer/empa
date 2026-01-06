#include "./util.wgsl"

@group(0) @binding(0)
var<storage, read> sums: array<u32>;

@group(0) @binding(1)
var<storage, read_write> data: array<u32>;

@compute @workgroup_size(256, 1, 1)
fn main(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(num_workgroups) grid_size: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32
) {
    let BLOCK_SIZE = 512u; // workgroup size * 2
    let group_index = group_index(workgroup_id, grid_size);
    let data_len = arrayLength(&data);

    let sum = sums[group_index];

    let data_index_a = group_index * BLOCK_SIZE + local_index * 2;
    let data_index_b = group_index * BLOCK_SIZE + local_index * 2 + 1;

    // We do bounds checks, as the WGSL spec allows an implentation to treat an out-of-bounds write as either a no-op,
    // or as a redirect to any location in the buffer, the latter being a case we want to avoid.

    if data_index_a < data_len {
        data[data_index_a] += sum;
    }

    if data_index_b < data_len {
        data[data_index_b] += sum;
    }
}
