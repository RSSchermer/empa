#![feature(const_ptr_offset_from)]

use arwa::dom::{selector, ParentNode};
use arwa::html::HtmlCanvasElement;
use arwa::window::window;
use empa::arwa::{
    CanvasConfiguration, CompositingAlphaMode, HtmlCanvasElementExt, NavigatorExt,
    PredefinedColorSpace, RequestAdapterOptions,
};
use empa::command::{Draw, RenderPassDescriptor};
use empa::device::DeviceDescriptor;
use empa::render_pipeline::{
    ColorOutput, ColorWriteMask, FragmentStageBuilder, RenderPipelineDescriptorBuilder,
    VertexStageBuilder,
};
use empa::render_target::{FloatAttachment, LoadOp, RenderTarget, StoreOp};
use empa::shader_module::{shader_source, ShaderSource};
use empa::texture::format::rgba8unorm;
use empa::texture::AttachableImageDescriptor;
use empa::type_flag::{O, X};
use empa::{buffer, texture};
use futures::FutureExt;
use std::convert::TryInto;
use std::error::Error;

mod console_error_panic_hook;

#[derive(empa::render_pipeline::Vertex, Clone, Copy)]
struct MyVertex {
    #[vertex_attribute(location = 0, format = "float32x2")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "unorm8x4")]
    color: [u8; 4],
}

const SHADER: ShaderSource = shader_source!("shader.wgsl");

fn main() {
    std::panic::set_hook(Box::new(crate::console_error_panic_hook::hook));

    arwa::spawn_local(render().map(|res| res.unwrap()));
}

async fn render() -> Result<(), Box<dyn Error>> {
    let window = window();
    let empa = window.navigator().empa();
    let canvas: HtmlCanvasElement = window
        .document()
        .query_selector(&selector!("#canvas"))
        .ok_or("canvas not found")?
        .try_into()?;

    let adapter = empa
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .ok_or("adapter not found")?;
    let device = adapter.request_device(&DeviceDescriptor::default()).await?;

    let context = canvas.empa_context().configure(&CanvasConfiguration {
        device: &device,
        format: rgba8unorm,
        usage: texture::Usages::render_attachment(),
        view_formats: (rgba8unorm,),
        color_space: PredefinedColorSpace::srgb,
        compositing_alpha_mode: CompositingAlphaMode::Opaque,
    });

    let shader = device.create_shader_module(&SHADER);

    let pipeline_layout = device.create_pipeline_layout::<()>();

    let pipeline = device.create_render_pipeline(
        &RenderPipelineDescriptorBuilder::begin()
            .layout(&pipeline_layout)
            .vertex(
                &VertexStageBuilder::begin(&shader, "vert_main")
                    .vertex_layout::<MyVertex>()
                    .finish(),
            )
            .fragment(
                &FragmentStageBuilder::begin(&shader, "frag_main")
                    .color_outputs(ColorOutput {
                        format: rgba8unorm,
                        write_mask: ColorWriteMask::ALL,
                    })
                    .finish(),
            )
            .finish(),
    );

    let vertex_data = [
        MyVertex {
            position: [0.0, 0.5],
            color: [255, 0, 0, 255],
        },
        MyVertex {
            position: [-0.5, -0.5],
            color: [0, 255, 0, 255],
        },
        MyVertex {
            position: [0.5, -0.5],
            color: [0, 0, 255, 255],
        },
    ];

    let vertex_buffer = device
        .create_buffer::<_, [MyVertex], buffer::Usages<O, O, O, O, X, O, O, O, O, O>>(vertex_data);

    let command_buffer = device
        .create_command_encoder()
        .begin_render_pass(&RenderPassDescriptor::new(&RenderTarget {
            color: FloatAttachment {
                image: &context
                    .get_current_texture()
                    .attachable_image(&AttachableImageDescriptor::default()),
                load_op: LoadOp::Clear([0.0; 4]),
                store_op: StoreOp::Store,
            },
            depth_stencil: (),
        }))
        .set_pipeline(&pipeline)
        .set_vertex_buffers(&vertex_buffer)
        .draw(Draw {
            vertex_count: vertex_buffer.len() as u32,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
        })
        .end()
        .finish();

    device.queue().submit(command_buffer);

    Ok(())
}
