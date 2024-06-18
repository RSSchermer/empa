#include "./util.wgsl"

@group(0) @binding(0)
var<storage, read_write> data: array<u32>;

@group(0) @binding(1)
var<storage, read_write> partial_sums: array<u32>;

var<workgroup> shared_data: array<u32, 512>;

// Note: this algorithm is specific to a workgroup size of 256 (the minimum size required by the WebGPU spec). Pipeline
// overridable constants don't quite work as this is being written. When this feature is available, this can be extended
// for greater workgroup sizes.
@compute @workgroup_size(256, 1, 1)
fn main(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(num_workgroups) grid_size: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32
) {
    let BLOCK_SIZE = 512u; // workgroup size * 2
    let group_index = group_index(workgroup_id, grid_size);
    let data_len = arrayLength(&data);

    // Load data into shared workgroup memory. As the block size in 2 x the workgroup size, each thread on the workgroup
    // will load 2 values. We do a bounds check and leave the shared memory value `0` if the index is out of bounds.
    let data_index_a = group_index * BLOCK_SIZE + local_index * 2;
    let data_index_b = group_index * BLOCK_SIZE + local_index * 2 + 1;

    if data_index_a < data_len {
        shared_data[local_index * 2] = data[data_index_a];
    }

    if data_index_b < data_len {
        shared_data[local_index * 2 + 1] = data[data_index_b];
    }

    // The "up-sweep" phase of the Blelloch algorithm. We implement an unrolled version  of the algorithm for the
    // specific workgroup size of `256`.
    //
    // We can do the first step before the first barrier, as in this step each thread only touches the 2 values it just
    // loaded itself. We then synchronize between each step. Every thread on the group participates in the first step,
    // then for each subsequent step the number of participating threads halves.

    shared_data[local_index * 2 + 1] += shared_data[local_index * 2];
    workgroupBarrier();

    if local_index < 128 {
        shared_data[local_index * 4 + 3] += shared_data[local_index * 4 + 1];
    }
    workgroupBarrier();

    if local_index < 64 {
        shared_data[local_index * 8 + 7] += shared_data[local_index * 8 + 3];
    }
    workgroupBarrier();

    if local_index < 32 {
        shared_data[local_index * 16 + 15] += shared_data[local_index * 16 + 7];
    }
    workgroupBarrier();

    if local_index < 16 {
        shared_data[local_index * 32 + 31] += shared_data[local_index * 32 + 15];
    }
    workgroupBarrier();

    if local_index < 8 {
        shared_data[local_index * 64 + 63] += shared_data[local_index * 64 + 31];
    }
    workgroupBarrier();

    if local_index < 4 {
        shared_data[local_index * 128 + 127] += shared_data[local_index * 128 + 63];
    }
    workgroupBarrier();

    if local_index < 2 {
        shared_data[local_index * 256 + 255] += shared_data[local_index * 256 + 127];
    }
    workgroupBarrier();

    // We skip the final up-sweep step, because this step only sets the last element in the shared array, which would
    // be set to `0` in the "down-sweep" phase.

    // The "down-sweep" phase of the Blelloch algorithm. We again implement an unrolled version of the algorithm for the
    // specific workgroup size of `256`, synchronizing between each step. Only one thread participates on the first
    // step, then for each subsequent step the number of threads that participate doubles, until all threads participate
    // in the final step.

    if local_index == 0 {
        // We can compute the total sum of all elements in this workgroup's data slice now, for the multilevel
        // recursive version of the algorithm for data lists with a length greater than 512.
        partial_sums[group_index] = shared_data[511] + shared_data[255];

        shared_data[511] = shared_data[255];
        shared_data[255] = 0u;
    }
    workgroupBarrier();

    if local_index < 2 {
        let v = shared_data[local_index * 256 + 255];

        shared_data[local_index * 256 + 255] += shared_data[local_index * 256 + 127];
        shared_data[local_index * 256 + 127] = v;
    }
    workgroupBarrier();

    if local_index < 4 {
        let v = shared_data[local_index * 128 + 127];

        shared_data[local_index * 128 + 127] += shared_data[local_index * 128 + 63];
        shared_data[local_index * 128 + 63] = v;
    }
    workgroupBarrier();

    if local_index < 8 {
        let v = shared_data[local_index * 64 + 63];

        shared_data[local_index * 64 + 63] += shared_data[local_index * 64 + 31];
        shared_data[local_index * 64 + 31] = v;
    }
    workgroupBarrier();

    if local_index < 16 {
        let v = shared_data[local_index * 32 + 31];

        shared_data[local_index * 32 + 31] += shared_data[local_index * 32 + 15];
        shared_data[local_index * 32 + 15] = v;
    }
    workgroupBarrier();

    if local_index < 32 {
        let v = shared_data[local_index * 16 + 15];

        shared_data[local_index * 16 + 15] += shared_data[local_index * 16 + 7];
        shared_data[local_index * 16 + 7] = v;
    }
    workgroupBarrier();

    if local_index < 64 {
        let v = shared_data[local_index * 8 + 7];

        shared_data[local_index * 8 + 7] += shared_data[local_index * 8 + 3];
        shared_data[local_index * 8 + 3] = v;
    }
    workgroupBarrier();

    if local_index < 128 {
        let v = shared_data[local_index * 4 + 3];

        shared_data[local_index * 4 + 3] += shared_data[local_index * 4 + 1];
        shared_data[local_index * 4 + 1] = v;
    }
    workgroupBarrier();

    let v = shared_data[local_index * 2 + 1];

    shared_data[local_index * 2 + 1] += shared_data[local_index * 2];
    shared_data[local_index * 2] = v;

    // All done! Copy the processed data from shared workgroup memory back into the data buffer. We do bounds checks,
    // as the WGSL spec allows an implentation to treat an out-of-bounds write as either a no-op, or as a redirect
    // to any location in the buffer, the latter being a case we want to avoid.
    if data_index_a < data_len {
        data[data_index_a] = shared_data[local_index * 2];
    }

    if data_index_b < data_len {
        data[data_index_b] = shared_data[local_index * 2 + 1];
    }
}
