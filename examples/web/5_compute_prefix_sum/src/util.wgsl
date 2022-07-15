#pragma once

fn group_index(global_id: vec3<u32>, grid_size: vec3<u32>) -> u32 {
    return global_id.x + (global_id.y * grid_size.x) + (global_id.z * grid_size.x * grid_size.y);
}
