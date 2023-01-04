use std::error::Error;

use arwa::console;
use arwa::window::window;
use empa::adapter::Features;
use empa::arwa::{NavigatorExt, RequestAdapterOptions};
use empa::buffer;
use empa::buffer::{Buffer, Storage};
use empa::command::{DispatchWorkgroups, ResourceBindingCommandEncoder};
use empa::compute_pipeline::{ComputePipelineDescriptorBuilder, ComputeStageBuilder};
use empa::device::DeviceDescriptor;
use empa::resource_binding::Resources;
use empa::shader_module::{shader_source, ShaderSource};
use futures::FutureExt;
use std::ops::Deref;

#[derive(empa::resource_binding::Resources)]
struct MyResources<'a> {
    #[resource(binding = 0, visibility = "COMPUTE")]
    data: &'a Storage<[u32]>,
}

const SHADER: ShaderSource = shader_source!("shader.wgsl");

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    arwa::spawn_local(render().map(|res| res.unwrap()));
}

async fn render() -> Result<(), Box<dyn Error>> {
    let window = window();
    let empa = window.navigator().empa();

    let adapter = empa
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .ok_or("adapter not found")?;
    let device = adapter
        .request_device(&DeviceDescriptor {
            required_features: Features::TIMESTAMP_QUERY,
            ..Default::default()
        })
        .await?;

    let shader = device.create_shader_module(&SHADER);

    type BindGroupLayout<'a> = <MyResources<'a> as Resources>::Layout;

    let bind_group_layout = device.create_bind_group_layout::<BindGroupLayout>();
    let pipeline_layout = device.create_pipeline_layout(&bind_group_layout);

    let pipeline = device.create_compute_pipeline(
        &ComputePipelineDescriptorBuilder::begin()
            .layout(&pipeline_layout)
            .compute(&ComputeStageBuilder::begin(&shader, "main").finish())
            .finish(),
    );

    let data: Vec<u32> = (0..1024).collect();

    let data_buffer: Buffer<[u32], _> =
        device.create_buffer(data, buffer::Usages::storage_binding().and_copy_src());

    let bind_group = device.create_bind_group(
        &bind_group_layout,
        MyResources {
            data: &data_buffer.storage(),
        },
    );

    let query_set = device.create_timestamp_query_set(2);
    let query_resolve_buffer: Buffer<[u64], _> =
        device.create_slice_buffer_zeroed(2, buffer::Usages::query_resolve().and_copy_src());
    let readback_buffer: Buffer<[u64], _> =
        device.create_slice_buffer_zeroed(2, buffer::Usages::copy_dst().and_map_read());

    let command_buffer = device
        .create_command_encoder()
        .write_timestamp(&query_set, 0)
        .begin_compute_pass()
        .set_pipeline(&pipeline)
        .set_bind_groups(&bind_group)
        .dispatch_workgroups(DispatchWorkgroups {
            count_x: data_buffer.len() as u32,
            count_y: 1,
            count_z: 1,
        })
        .end()
        .write_timestamp(&query_set, 1)
        .resolve_timestamp_query_set(&query_set, 0, query_resolve_buffer.view())
        .copy_buffer_to_buffer_slice(query_resolve_buffer.view(), readback_buffer.view())
        .finish();

    device.queue().submit(command_buffer);

    readback_buffer.map_read().await?;

    {
        let mapped = readback_buffer.mapped();
        let time_elapsed = mapped[1] - mapped[0];

        console::log!("Time elapsed: %i nanoseconds", time_elapsed);
    }

    readback_buffer.unmap();

    Ok(())
}
