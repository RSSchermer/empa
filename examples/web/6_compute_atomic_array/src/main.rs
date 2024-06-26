use std::error::Error;
use std::ops::Deref;

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

    let data: Vec<u32> = vec![0; 32];

    let data_buffer: Buffer<[u32], _> =
        device.create_buffer(data, buffer::Usages::storage_binding().and_copy_src());
    let readback_buffer: Buffer<[u32], _> =
        device.create_buffer(vec![0u32; 32], buffer::Usages::map_read().and_copy_dst());

    let bind_group = device.create_bind_group(
        &bind_group_layout,
        MyResources {
            data: data_buffer.storage(),
        },
    );

    let command_buffer = device
        .create_command_encoder()
        .begin_compute_pass()
        .set_pipeline(&pipeline)
        .set_bind_groups(&bind_group)
        .dispatch_workgroups(DispatchWorkgroups {
            count_x: 1024,
            count_y: 1,
            count_z: 1,
        })
        .end()
        .copy_buffer_to_buffer_slice(data_buffer.view(), readback_buffer.view())
        .finish();

    device.queue().submit(command_buffer);

    readback_buffer.map_read().await?;

    {
        let data = readback_buffer.mapped();

        console::log!(
            "Atomic addition on the GPU:",
            format!("{:#?}", data.deref())
        );
    }

    readback_buffer.unmap();

    Ok(())
}
