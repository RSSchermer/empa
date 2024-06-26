use std::error::Error;
use std::mem;

use arwa::console;
use arwa::window::window;
use empa::access_mode::ReadWrite;
use empa::arwa::{NavigatorExt, RequestAdapterOptions};
use empa::buffer;
use empa::buffer::{Buffer, Storage};
use empa::command::{DispatchWorkgroups, ResourceBindingCommandEncoder};
use empa::compute_pipeline::{ComputePipelineDescriptorBuilder, ComputeStageBuilder};
use empa::device::DeviceDescriptor;
use empa::resource_binding::Resources;
use empa::shader_module::{shader_source, ShaderSource};
use futures::FutureExt;

#[derive(empa::resource_binding::Resources)]
struct MyResources<'a> {
    #[resource(binding = 0, visibility = "COMPUTE")]
    data: Storage<'a, [u32], ReadWrite>,
}

const SHADER: ShaderSource = shader_source!("shader.wgsl");
const WORKGROUP_SIZE: u32 = 64;

fn main() {
    arwa::spawn_local(render().map(|res| res.unwrap()));
}

async fn render() -> Result<(), Box<dyn Error>> {
    let window = window();
    let empa = window.navigator().empa();

    let adapter = empa
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .ok_or("adapter not found")?;
    let device = adapter.request_device(&DeviceDescriptor::default()).await?;

    let shader = device.create_shader_module(&SHADER);

    type BindGroupLayout<'a> = <MyResources<'a> as Resources>::Layout;

    let bind_group_layout = device.create_bind_group_layout::<BindGroupLayout>();
    let pipeline_layout = device.create_pipeline_layout(&bind_group_layout);

    let pipeline = device
        .create_compute_pipeline(
            &ComputePipelineDescriptorBuilder::begin()
                .layout(&pipeline_layout)
                .compute(ComputeStageBuilder::begin(&shader, "main").finish())
                .finish(),
        )
        .await;

    let data: Vec<u32> = (0..1024).collect();

    let data_buffer: Buffer<[u32], _> =
        device.create_buffer(data, buffer::Usages::storage_binding().and_copy_src());
    let readback_buffer: Buffer<[u32], _> =
        device.create_buffer(vec![0u32; 1024], buffer::Usages::map_read().and_copy_dst());

    let bind_group = device.create_bind_group(
        &bind_group_layout,
        MyResources {
            data: data_buffer.storage(),
        },
    );

    let workgroups = (data_buffer.len() as u32).div_ceil(WORKGROUP_SIZE);

    let command_buffer = device
        .create_command_encoder()
        .begin_compute_pass()
        .set_pipeline(&pipeline)
        .set_bind_groups(&bind_group)
        .dispatch_workgroups(DispatchWorkgroups {
            count_x: workgroups,
            count_y: 1,
            count_z: 1,
        })
        .end()
        .copy_buffer_to_buffer_slice(data_buffer.view(), readback_buffer.view())
        .finish();

    device.queue().submit(command_buffer);

    readback_buffer.map_read().await?;

    let mapped = readback_buffer.mapped();

    console::log!("Asserting that the results are correct...");

    for i in 0..mapped.len() {
        assert_eq!(mapped[i], (i * i) as u32);
    }

    console::log!("...successfully!");

    console::log!(
        "First 10 numbers squared on the GPU:",
        format!("{:#?}", &mapped[..10])
    );
    console::log!(
        "Last 10 numbers squared on the GPU:",
        format!("{:#?}", &mapped[mapped.len() - 10..])
    );

    // Make sure we drop the mapped data before unmapping, otherwise unmapping will panic.
    mem::drop(mapped);

    readback_buffer.unmap();

    Ok(())
}
