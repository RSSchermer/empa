#![feature(const_ptr_offset_from)]

use std::error::Error;

use arwa::dom::{selector, ParentNode};
use arwa::html::{HtmlCanvasElement, HtmlImgElement};
use arwa::image_bitmap::{create_image_bitmap, ImageBitmapOptions, ImageRegion};
use arwa::window::window;
use empa::arwa::{
    CanvasConfiguration, CompositingAlphaMode, ExternalImageCopySrc, HtmlCanvasElementExt,
    NavigatorExt, PredefinedColorSpace, QueueExt, RequestAdapterOptions, Texture2DExt,
};
use empa::buffer::Buffer;
use empa::command::{Draw, RenderPassDescriptor};
use empa::device::DeviceDescriptor;
use empa::render_pipeline::{
    ColorOutput, ColorWriteMask, FragmentStageBuilder, RenderPipelineDescriptorBuilder,
    VertexStageBuilder,
};
use empa::render_target::{FloatAttachment, LoadOp, RenderTarget, StoreOp};
use empa::resource_binding::Resources;
use empa::sampler::{FilterMode, MipmapFilterMode, Sampler, SamplerDescriptor};
use empa::shader_module::{shader_source, ShaderSource};
use empa::texture::format::{rgba8unorm, rgba8unorm_srgb};
use empa::texture::{
    AttachableImageDescriptor, ImageCopySize2D, MipmapLevels, Sampled2DFloat, Texture2DDescriptor,
};
use empa::{buffer, texture};
use futures::FutureExt;

#[derive(empa::render_pipeline::Vertex, Clone, Copy)]
struct MyVertex {
    #[vertex_attribute(location = 0, format = "float32x2")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "float32x2")]
    texture_coordinates: [f32; 2],
}

#[derive(empa::resource_binding::Resources)]
struct MyResources<'a> {
    #[resource(binding = 0, visibility = "VERTEX | FRAGMENT")]
    texture: &'a Sampled2DFloat,
    #[resource(binding = 1, visibility = "VERTEX | FRAGMENT")]
    sampler: &'a Sampler,
}

const SHADER: ShaderSource = shader_source!("shader.wgsl");

fn main() {
    arwa::spawn_local(render().map(|res| res.unwrap()));
}

async fn render() -> Result<(), Box<dyn Error>> {
    let window = window();
    let document = window.document();
    let empa = window.navigator().empa();
    let canvas: HtmlCanvasElement = document
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
        color_space: PredefinedColorSpace::srgb,
        compositing_alpha_mode: CompositingAlphaMode::Opaque,
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
            .finish(),
    );

    let img: HtmlImgElement = document
        .query_selector(&selector!("#checkerboard_gradient"))
        .ok_or("texture image not found")?
        .try_into()?;
    let bitmap = create_image_bitmap(
        &img,
        ImageRegion {
            x: 0,
            y: 0,
            width: 256,
            height: 256,
        },
        ImageBitmapOptions::default(),
    )
    .await?;

    let vertex_data = [
        MyVertex {
            position: [0.0, 0.5],
            texture_coordinates: [0.5, 0.0],
        },
        MyVertex {
            position: [-0.5, -0.5],
            texture_coordinates: [0.0, 1.0],
        },
        MyVertex {
            position: [0.5, -0.5],
            texture_coordinates: [1.0, 1.0],
        },
    ];

    let vertex_buffer: Buffer<[MyVertex], _> =
        device.create_buffer(vertex_data, buffer::Usages::vertex());
    let texture = device.create_texture_2d(&Texture2DDescriptor {
        format: rgba8unorm_srgb,
        usage: texture::Usages::copy_dst()
            .and_render_attachment()
            .and_texture_binding(),
        view_formats: (rgba8unorm_srgb,),
        width: 256,
        height: 256,
        layers: 1,
        mipmap_levels: MipmapLevels::Partial(1),
    });
    let sampler = device.create_sampler(&SamplerDescriptor {
        magnification_filter: FilterMode::Linear,
        minification_filter: FilterMode::Linear,
        mipmap_filter: MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(
        &bind_group_layout,
        MyResources {
            texture: &texture.sampled_float(&Default::default()),
            sampler: &sampler,
        },
    );

    let queue = device.queue();

    let texture_src = ExternalImageCopySrc::image_bitmap(&bitmap, Default::default());
    let texture_dst = texture.external_image_copy_dst(Default::default());

    queue.copy_external_image_to_texture(
        &texture_src,
        &texture_dst,
        ImageCopySize2D {
            width: 256,
            height: 256,
        },
    );

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
        .set_bind_groups(&bind_group)
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
