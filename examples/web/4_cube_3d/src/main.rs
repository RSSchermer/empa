use std::error::Error;
use std::f32::consts::PI;

use arwa::dom::{selector, ParentNode};
use arwa::html::HtmlCanvasElement;
use arwa::window::window;
use empa::arwa::{
    AlphaMode, CanvasConfiguration, HtmlCanvasElementExt, NavigatorExt, RequestAdapterOptions,
};
use empa::buffer::{Buffer, Uniform};
use empa::command::{
    DrawIndexed, DrawIndexedCommandEncoder, RenderBundleEncoderDescriptor, RenderPassDescriptor,
    RenderStateEncoder, ResourceBindingCommandEncoder,
};
use empa::device::DeviceDescriptor;
use empa::render_pipeline::{
    ColorOutput, ColorWriteMask, DepthStencilTest, FragmentStageBuilder,
    RenderPipelineDescriptorBuilder, VertexStageBuilder,
};
use empa::render_target::{
    DepthAttachment, DepthValue, FloatAttachment, LoadOp, RenderTarget, StoreOp,
};
use empa::resource_binding::Resources;
use empa::shader_module::{shader_source, ShaderSource};
use empa::texture::format::{depth24plus, rgba8unorm};
use empa::texture::{AttachableImageDescriptor, MipmapLevels, Texture2DDescriptor};
use empa::{abi, buffer, texture, CompareFunction};
use empa_glam::ToAbi;
use futures::FutureExt;

#[derive(empa::render_pipeline::Vertex, Clone, Copy)]
struct MyVertex {
    #[vertex_attribute(location = 0, format = "float32x4")]
    position: [f32; 4],
    #[vertex_attribute(location = 1, format = "unorm8x4")]
    color: [u8; 4],
}

#[derive(empa::abi::Sized, Clone, Copy)]
struct Uniforms {
    model: abi::Mat4x4,
    view: abi::Mat4x4,
    projection: abi::Mat4x4,
}

#[derive(empa::resource_binding::Resources)]
struct MyResources<'a> {
    #[resource(binding = 0, visibility = "VERTEX")]
    uniform_buffer: &'a Uniform<Uniforms>,
}

const SHADER: ShaderSource = shader_source!("shader.wgsl");

fn main() {
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
        view_formats: (),
        alpha_mode: AlphaMode::Opaque,
    });

    let shader = device.create_shader_module(&SHADER);

    type BindGroupLayout<'a> = <MyResources<'a> as Resources>::Layout;

    let bind_group_layout = device.create_bind_group_layout::<BindGroupLayout>();
    let pipeline_layout = device.create_pipeline_layout(&bind_group_layout);

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
            .depth_stencil_test(
                DepthStencilTest::read_write::<depth24plus>()
                    .depth_compare(CompareFunction::LessEqual),
            )
            .finish(),
    );

    let vertex_data = [
        MyVertex {
            position: [-5.0, -5.0, -5.0, 1.0],
            color: [255, 0, 0, 255],
        },
        MyVertex {
            position: [5.0, -5.0, -5.0, 1.0],
            color: [0, 255, 0, 255],
        },
        MyVertex {
            position: [-5.0, 5.0, -5.0, 1.0],
            color: [0, 0, 255, 255],
        },
        MyVertex {
            position: [5.0, 5.0, -5.0, 1.0],
            color: [255, 0, 0, 255],
        },
        MyVertex {
            position: [-5.0, -5.0, 5.0, 1.0],
            color: [0, 255, 255, 255],
        },
        MyVertex {
            position: [5.0, -5.0, 5.0, 1.0],
            color: [0, 0, 255, 255],
        },
        MyVertex {
            position: [-5.0, 5.0, 5.0, 1.0],
            color: [255, 255, 0, 255],
        },
        MyVertex {
            position: [5.0, 5.0, 5.0, 1.0],
            color: [0, 255, 0, 255],
        },
    ];

    let vertex_buffer: Buffer<[MyVertex], _> =
        device.create_buffer(vertex_data, buffer::Usages::vertex());

    let index_data: Vec<u16> = vec![
        0, 2, 1, // Back
        1, 2, 3, //
        0, 6, 2, // Left
        0, 4, 6, //
        1, 3, 7, // Right
        1, 7, 5, //
        2, 7, 3, // Top
        2, 6, 7, //
        0, 1, 5, // Bottom
        0, 5, 4, //
        4, 5, 7, // Front
        6, 4, 7, //
    ];

    let index_buffer: Buffer<[u16], _> = device.create_buffer(index_data, buffer::Usages::index());

    let view = glam::f32::Mat4::from_translation(glam::f32::Vec3::new(0.0, 0.0, 30.0))
        .inverse()
        .to_abi();
    let projection = glam::f32::Mat4::perspective_rh(0.3 * PI, 1.0, 1.0, 100.0).to_abi();
    let uniforms = Uniforms {
        model: glam::f32::Mat4::IDENTITY.to_abi(),
        view,
        projection,
    };

    let uniform_buffer =
        device.create_buffer(uniforms, buffer::Usages::uniform_binding().and_copy_dst());

    let bind_group = device.create_bind_group(
        &bind_group_layout,
        MyResources {
            uniform_buffer: &uniform_buffer.uniform(),
        },
    );

    let depth_texture = device.create_texture_2d(&Texture2DDescriptor {
        format: depth24plus,
        usage: texture::Usages::render_attachment(),
        view_formats: (),
        width: canvas.width(),
        height: canvas.height(),
        layers: 1,
        mipmap_levels: MipmapLevels::Partial(1),
    });

    let render_bundle_encoder = device.create_render_bundle_encoder(
        &RenderBundleEncoderDescriptor::new::<rgba8unorm>().depth_stencil_format::<depth24plus>(),
    );

    let render_bundle = render_bundle_encoder
        .set_pipeline(&pipeline)
        .set_vertex_buffers(&vertex_buffer)
        .set_index_buffer(&index_buffer)
        .set_bind_groups(&bind_group)
        .draw_indexed(DrawIndexed {
            index_count: index_buffer.len() as u32,
            instance_count: 1,
            first_index: 0,
            first_instance: 0,
            base_vertex: 0,
        })
        .finish();

    let queue = device.queue();

    loop {
        let time = window.request_animation_frame().await;
        let time = time as f32;

        let rotate_x = glam::f32::Mat4::from_rotation_x(time / 1000.0);
        let rotate_y = glam::f32::Mat4::from_rotation_y(time / 1000.0);
        let model = rotate_y * rotate_x;
        let uniforms = Uniforms {
            model: model.to_abi(),
            view,
            projection,
        };

        queue.write_buffer(uniform_buffer.view(), &uniforms);

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
                depth_stencil: DepthAttachment {
                    image: &depth_texture.attachable_image(&AttachableImageDescriptor::default()),
                    load_op: LoadOp::Clear(DepthValue::ONE),
                    store_op: StoreOp::Store,
                },
            }))
            .execute_bundle(&render_bundle)
            .end()
            .finish();

        queue.submit(command_buffer);
    }
}
