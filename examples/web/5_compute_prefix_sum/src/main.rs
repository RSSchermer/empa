#![feature(int_roundings)]

use std::collections::VecDeque;
use std::error::Error;

use arwa::console;
use arwa::window::window;
use empa::arwa::{NavigatorExt, RequestAdapterOptions};
use empa::buffer;
use empa::buffer::{Buffer, ReadOnlyStorage, Storage, StorageBinding};
use empa::command::{DispatchWorkgroups, ResourceBindingCommandEncoder};
use empa::compute_pipeline::{
    ComputePipeline, ComputePipelineDescriptorBuilder, ComputeStageBuilder,
};
use empa::device::{Device, DeviceDescriptor};
use empa::resource_binding::BindGroupLayout;
use empa::shader_module::{shader_source, ShaderSource};
use futures::FutureExt;

const SCAN_SHADER: ShaderSource = shader_source!("scan.wgsl");

#[derive(empa::resource_binding::Resources)]
struct ScanResources {
    #[resource(binding = 0, visibility = "COMPUTE")]
    data: Storage<[u32]>,
    #[resource(binding = 1, visibility = "COMPUTE")]
    partial_sums: Storage<[u32]>,
}

type ScanLayout = <ScanResources as empa::resource_binding::Resources>::Layout;

const UNIFORM_ADD_SHADER: ShaderSource = shader_source!("uniform_add.wgsl");

#[derive(empa::resource_binding::Resources)]
struct UniformAddResources {
    #[resource(binding = 0, visibility = "COMPUTE")]
    sums: ReadOnlyStorage<[u32]>,
    #[resource(binding = 1, visibility = "COMPUTE")]
    data: Storage<[u32]>,
}

type UniformAddLayout = <UniformAddResources as empa::resource_binding::Resources>::Layout;

const BLOCK_SIZE: u32 = 512;

struct Evaluator {
    device: Device,
    scan_bind_group_layout: BindGroupLayout<ScanLayout>,
    scan_pipeline: ComputePipeline<(ScanLayout,)>,
    uniform_add_bind_group_layout: BindGroupLayout<UniformAddLayout>,
    uniform_add_pipeline: ComputePipeline<(UniformAddLayout,)>,
}

impl Evaluator {
    fn new(device: Device) -> Self {
        let scan_shader = device.create_shader_module(&SCAN_SHADER);

        let scan_bind_group_layout = device.create_bind_group_layout::<ScanLayout>();
        let scan_pipeline_layout = device.create_pipeline_layout(&scan_bind_group_layout);

        let scan_pipeline = device.create_compute_pipeline(
            &ComputePipelineDescriptorBuilder::begin()
                .layout(&scan_pipeline_layout)
                .compute(&ComputeStageBuilder::begin(&scan_shader, "main").finish())
                .finish(),
        );

        let uniform_add_shader = device.create_shader_module(&UNIFORM_ADD_SHADER);

        let uniform_add_bind_group_layout = device.create_bind_group_layout::<UniformAddLayout>();
        let uniform_add_pipeline_layout =
            device.create_pipeline_layout(&uniform_add_bind_group_layout);

        let uniform_add_pipeline = device.create_compute_pipeline(
            &ComputePipelineDescriptorBuilder::begin()
                .layout(&uniform_add_pipeline_layout)
                .compute(&ComputeStageBuilder::begin(&uniform_add_shader, "main").finish())
                .finish(),
        );

        Evaluator {
            device,
            scan_bind_group_layout,
            scan_pipeline,
            uniform_add_bind_group_layout,
            uniform_add_pipeline,
        }
    }

    fn prefix_sum<U>(&self, data: &Buffer<[u32], U>)
    where
        U: StorageBinding,
    {
        let Evaluator {
            device,
            scan_bind_group_layout,
            scan_pipeline,
            uniform_add_bind_group_layout,
            uniform_add_pipeline,
        } = self;

        let mut recursion_level = 0;
        let mut remainder = data.len() as u32;
        let mut multilevel_buffers = VecDeque::new();

        let dummy_buffer = device.create_slice_buffer_uninit(1, buffer::Usages::storage_binding());
        let dummy_buffer: Buffer<[u32], _> = unsafe { dummy_buffer.assume_init() };

        multilevel_buffers.push_front(dummy_buffer);

        loop {
            remainder = remainder.div_ceil(BLOCK_SIZE);

            if remainder <= 1 {
                break;
            }

            recursion_level += 1;

            let len = 512usize.pow(recursion_level);
            let buffer = device.create_slice_buffer_uninit(len, buffer::Usages::storage_binding());
            let buffer = unsafe { buffer.assume_init() };

            multilevel_buffers.push_front(buffer);
        }

        let encoder = device.create_command_encoder();

        let bind_group = device.create_bind_group(
            scan_bind_group_layout,
            ScanResources {
                data: data.storage(),
                partial_sums: multilevel_buffers[0].storage(),
            },
        );

        let mut encoder = encoder
            .begin_compute_pass()
            .set_bind_groups(&bind_group)
            .set_pipeline(scan_pipeline)
            .dispatch_workgroups(DispatchWorkgroups {
                count_x: (data.len() as u32).div_ceil(BLOCK_SIZE),
                count_y: 1,
                count_z: 1,
            })
            .end();

        for i in 0..recursion_level {
            let data = &multilevel_buffers[i as usize];
            let partial_sums = &multilevel_buffers[i as usize + 1];

            let bind_group = device.create_bind_group(
                scan_bind_group_layout,
                ScanResources {
                    data: data.storage(),
                    partial_sums: partial_sums.storage(),
                },
            );

            encoder = encoder
                .begin_compute_pass()
                .set_bind_groups(&bind_group)
                .set_pipeline(scan_pipeline)
                .dispatch_workgroups(DispatchWorkgroups {
                    // Note `data.len()` is always an exact multiple of BLOCK_SIZE here, there are
                    // no rounding issues.
                    count_x: data.len() as u32 / BLOCK_SIZE,
                    count_y: 1,
                    count_z: 1,
                })
                .end();
        }

        if recursion_level > 1 {
            for i in (2..=recursion_level).rev() {
                let data = &multilevel_buffers[i as usize - 2];
                let sums = &multilevel_buffers[i as usize - 1];

                let bind_group = device.create_bind_group(
                    uniform_add_bind_group_layout,
                    UniformAddResources {
                        sums: sums.read_only_storage(),
                        data: data.storage(),
                    },
                );

                encoder = encoder
                    .begin_compute_pass()
                    .set_bind_groups(&bind_group)
                    .set_pipeline(uniform_add_pipeline)
                    .dispatch_workgroups(DispatchWorkgroups {
                        // Note `data.len()` is always an exact multiple of BLOCK_SIZE here, there
                        // are no rounding issues.
                        count_x: data.len() as u32 / BLOCK_SIZE,
                        count_y: 1,
                        count_z: 1,
                    })
                    .end();
            }
        }

        if recursion_level > 0 {
            let bind_group = device.create_bind_group(
                uniform_add_bind_group_layout,
                UniformAddResources {
                    sums: multilevel_buffers[0].read_only_storage(),
                    data: data.storage(),
                },
            );

            encoder = encoder
                .begin_compute_pass()
                .set_bind_groups(&bind_group)
                .set_pipeline(uniform_add_pipeline)
                .dispatch_workgroups(DispatchWorkgroups {
                    // Note `data.len()` is always an exact multiple of BLOCK_SIZE here, there are
                    // no rounding issues.
                    count_x: (data.len() as u32).div_ceil(BLOCK_SIZE),
                    count_y: 1,
                    count_z: 1,
                })
                .end();
        }

        device.queue().submit(encoder.finish());
    }
}

fn main() {
    arwa::spawn_local(compute().map(|res| res.unwrap()));
}

async fn compute() -> Result<(), Box<dyn Error>> {
    let window = window();
    let empa = window.navigator().empa();

    let adapter = empa
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .ok_or("adapter not found")?;
    let device = adapter.request_device(&DeviceDescriptor::default()).await?;

    let evaluator = Evaluator::new(device.clone());

    let data: Vec<u32> = vec![1; 1_000_000];

    let data_buffer: Buffer<[u32], _> =
        device.create_buffer(data, buffer::Usages::storage_binding().and_copy_src());

    evaluator.prefix_sum(&data_buffer);

    let readback_buffer: Buffer<[u32], _> = device.create_buffer(
        vec![0; 1_000_000],
        buffer::Usages::map_read().and_copy_dst(),
    );

    let command_buffer = device
        .create_command_encoder()
        .copy_buffer_to_buffer_slice(data_buffer.view(), readback_buffer.view())
        .finish();

    device.queue().submit(command_buffer);

    readback_buffer.map_read().await?;

    {
        let data = readback_buffer.mapped();

        console::log!("The first 10 numbers:", format!("{:#?}", &data[..10]));
        console::log!(
            "The last 10 numbers:",
            format!("{:#?}", &data[data.len() - 10..])
        );
    }

    readback_buffer.unmap();

    Ok(())
}
