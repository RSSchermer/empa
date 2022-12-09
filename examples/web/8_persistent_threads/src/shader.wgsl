const JOB_ADD_1 = 0u;
const JOB_ADD_2 = 1u;
const JOB_EXIT = 2u;

@group(0) @binding(0)
var<storage, read_write> data: array<u32>;

var<workgroup> iterations: u32;

var<workgroup> job: u32;

var<workgroup> count: atomic<u32>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(workgroup_id) workgroup_id: vec3<u32>, @builtin(local_invocation_index) local_index: u32) {
    loop {
        if local_index == 0 {
            if iterations >= 20 {
                data[workgroup_id.x] = atomicLoad(&count);
                job = JOB_EXIT;
            } else if iterations >= 10 {
                job = JOB_ADD_2;
            }

            iterations += 1;
        }

        workgroupBarrier();

        if job == JOB_ADD_1 {
            atomicAdd(&count, 1);
        } else if job == JOB_ADD_2 {
            atomicAdd(&count, 2);
        } else {
            return;
        }
    }
}
