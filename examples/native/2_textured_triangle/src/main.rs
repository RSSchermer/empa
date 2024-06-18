use std::error::Error;
use std::fs::File;

use empa::buffer::{Buffer, BufferUsages};
use empa::command::{
    Draw, DrawCommandEncoder, RenderPassDescriptor, RenderStateEncoder,
    ResourceBindingCommandEncoder,
};
use empa::device::{Device, DeviceDescriptor};
use empa::native::{
    AdapterOptions, ConfiguredSurface, Instance, PowerPreference, SurfaceConfiguration,
};
use empa::render_pipeline::{
    ColorOutput, ColorWrite, FragmentStageBuilder, IndexAny, RenderPipeline,
    RenderPipelineDescriptorBuilder, VertexStageBuilder,
};
use empa::render_target::{FloatAttachment, LoadOp, RenderLayout, RenderTarget, StoreOp};
use empa::resource_binding::BindGroup;
use empa::sampler::{FilterMode, Sampler, SamplerDescriptor};
use empa::shader_module::{shader_source, ShaderSource};
use empa::texture::format::{bgra8unorm, rgba8unorm_srgb};
use empa::texture::{
    AttachableImageDescriptor, ImageDataLayout, MipmapLevels, Sampled2DFloat, Texture2DDescriptor,
    TextureUsages,
};
use empa::{buffer, texture};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(empa::render_pipeline::Vertex, Clone, Copy)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "float32x2")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "float32x2")]
    texture_coordinates: [f32; 2],
}

#[derive(empa::resource_binding::Resources)]
struct Resources<'a> {
    #[resource(binding = 0, visibility = "VERTEX | FRAGMENT")]
    texture: Sampled2DFloat<'a>,
    #[resource(binding = 1, visibility = "VERTEX | FRAGMENT")]
    sampler: &'a Sampler,
}

type ResourceLayout = <Resources<'static> as empa::resource_binding::Resources>::Layout;

const SHADER: ShaderSource = shader_source!("shader.wgsl");

struct AppState {
    device: Device,
    pipeline: RenderPipeline<RenderLayout<bgra8unorm, ()>, Vertex, IndexAny, (ResourceLayout,)>,
    bind_group: BindGroup<ResourceLayout>,
    vertex_buffer: Buffer<[Vertex], BufferUsages!(Vertex)>,
    surface: ConfiguredSurface<'static, bgra8unorm, TextureUsages!(RenderAttachment)>,
}

impl AppState {
    async fn init(window: Window) -> Result<Self, Box<dyn Error>> {
        let mut size = window.inner_size();

        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = Instance::default();
        let surface = instance.create_surface(window)?;
        let adapter = instance.get_adapter(AdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })?;

        let device = adapter.request_device(&DeviceDescriptor::default()).await?;

        let shader = device.create_shader_module(&SHADER);

        let bind_group_layout = device.create_bind_group_layout::<ResourceLayout>();
        let pipeline_layout = device.create_pipeline_layout(&bind_group_layout);

        let pipeline = device
            .create_render_pipeline(
                &RenderPipelineDescriptorBuilder::begin()
                    .layout(&pipeline_layout)
                    .vertex(
                        VertexStageBuilder::begin(&shader, "vert_main")
                            .vertex_layout::<Vertex>()
                            .finish(),
                    )
                    .fragment(
                        FragmentStageBuilder::begin(&shader, "frag_main")
                            .color_outputs(ColorOutput {
                                format: bgra8unorm,
                                write_mask: ColorWrite::All,
                            })
                            .finish(),
                    )
                    .finish(),
            )
            .await;

        let vertex_data = [
            Vertex {
                position: [0.0, 0.5],
                texture_coordinates: [0.5, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5],
                texture_coordinates: [0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5],
                texture_coordinates: [1.0, 1.0],
            },
        ];

        let vertex_buffer: Buffer<[Vertex], _> =
            device.create_buffer(vertex_data, buffer::Usages::vertex());

        let texture = device.create_texture_2d(&Texture2DDescriptor {
            format: rgba8unorm_srgb,
            usage: texture::Usages::copy_dst().and_texture_binding(),
            view_formats: (rgba8unorm_srgb,),
            width: 256,
            height: 256,
            layers: 1,
            mipmap_levels: MipmapLevels::Partial(1),
        });
        let sampler = device.create_sampler(&SamplerDescriptor {
            magnification_filter: FilterMode::Linear,
            minification_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(
            &bind_group_layout,
            Resources {
                texture: texture.sampled_float(&Default::default()),
                sampler: &sampler,
            },
        );

        let decoder = png::Decoder::new(File::open("checkerboard_gradient.png")?);
        let mut reader = decoder.read_info()?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf)?;
        let bytes = &buf[..info.buffer_size()];
        let bytes_rgba: &[[u8; 4]] = bytemuck::cast_slice(bytes);

        device.queue().write_texture(
            texture.image_copy_from_buffer_dst(0),
            bytes_rgba,
            ImageDataLayout {
                blocks_per_row: 256,
                rows_per_image: 256,
            },
        );

        let surface = surface.configure(
            &device,
            &SurfaceConfiguration {
                format: bgra8unorm,
                usage: texture::Usages::render_attachment(),
                width: size.width,
                height: size.height,
                present_mode: Default::default(),
                desired_maximum_frame_latency: 0,
                alpha_mode: Default::default(),
                view_formats: (),
            },
        );

        Ok(AppState {
            device,
            pipeline,
            bind_group,
            vertex_buffer,
            surface,
        })
    }

    pub fn draw_frame(&self) {
        let AppState {
            device,
            pipeline,
            vertex_buffer,
            bind_group,
            surface,
            ..
        } = self;

        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let command_buffer = device
            .create_command_encoder()
            .begin_render_pass(RenderPassDescriptor::new(&RenderTarget {
                color: FloatAttachment {
                    image: frame.attachable_image(&AttachableImageDescriptor::default()),
                    load_op: LoadOp::Clear([0.0; 4]),
                    store_op: StoreOp::Store,
                },
                depth_stencil: (),
            }))
            .set_pipeline(&pipeline)
            .set_vertex_buffers(&*vertex_buffer)
            .set_bind_groups(&*bind_group)
            .draw(Draw {
                vertex_count: vertex_buffer.len() as u32,
                instance_count: 1,
                first_vertex: 0,
                first_instance: 0,
            })
            .end()
            .finish();

        device.queue().submit(command_buffer);

        frame.present();
    }
}

struct App {
    state: Option<AppState>,
}

impl App {
    fn new() -> Self {
        App { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        let state = pollster::block_on(async move { AppState::init(window).await }).unwrap();

        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    let width = size.width.max(1);
                    let height = size.height.max(1);

                    state.surface.resize(width, height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.state.as_ref().unwrap().draw_frame();
            }
            _ => (),
        }
    }
}

pub fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
