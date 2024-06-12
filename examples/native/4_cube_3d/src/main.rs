use std::error::Error;
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use empa::buffer::{Buffer, Uniform};
use empa::command::{
    DrawIndexed, DrawIndexedCommandEncoder, RenderBundle, RenderBundleEncoderDescriptor,
    RenderPassDescriptor, RenderStateEncoder, ResourceBindingCommandEncoder,
};
use empa::device::{Device, DeviceDescriptor};
use empa::native::{AdapterOptions, ConfiguredSurface, Instance, SurfaceConfiguration};
use empa::render_pipeline::{
    ColorOutput, ColorWrite, DepthStencilTest, FragmentStageBuilder,
    RenderPipelineDescriptorBuilder, VertexStageBuilder,
};
use empa::render_target::{
    DepthAttachment, DepthValue, FloatAttachment, LoadOp, RenderLayout, RenderTarget, StoreOp,
};
use empa::shader_module::{shader_source, ShaderSource};
use empa::texture::format::{bgra8unorm, depth24plus};
use empa::texture::{AttachableImageDescriptor, MipmapLevels, Texture2D, Texture2DDescriptor};
use empa::type_flag::{O, X};
use empa::{abi, buffer, texture, CompareFunction};
use empa_glam::ToAbi;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(empa::render_pipeline::Vertex, Clone, Copy)]
struct Vertex {
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
struct Resources<'a> {
    #[resource(binding = 0, visibility = "VERTEX | FRAGMENT")]
    uniform_buffer: Uniform<'a, Uniforms>,
}

type ResourceLayout = <Resources<'static> as empa::resource_binding::Resources>::Layout;

const SHADER: ShaderSource = shader_source!("shader.wgsl");

struct AppState {
    device: Device,
    view: abi::Mat4x4,
    projection: abi::Mat4x4,
    uniform_buffer: Buffer<Uniforms, buffer::Usages<O, O, O, X, O, O, X, O, O, O>>,
    depth_texture: Texture2D<depth24plus, texture::Usages<X, O, O, O, O>>,
    render_bundle: RenderBundle<RenderLayout<bgra8unorm, depth24plus>>,
    window: Arc<Window>,
    surface: ConfiguredSurface<'static, bgra8unorm, texture::Usages<X, O, O, O, O>>,
}

impl AppState {
    async fn init(window: Window) -> Result<Self, Box<dyn Error>> {
        let mut size = window.inner_size();

        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let window = Arc::new(window);
        let instance = Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance.get_adapter(AdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
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
                    .depth_stencil_test(
                        DepthStencilTest::read_write::<depth24plus>()
                            .depth_compare(CompareFunction::LessEqual),
                    )
                    .finish(),
            )
            .await;

        let vertex_data = [
            Vertex {
                position: [-5.0, -5.0, -5.0, 1.0],
                color: [255, 0, 0, 255],
            },
            Vertex {
                position: [5.0, -5.0, -5.0, 1.0],
                color: [0, 255, 0, 255],
            },
            Vertex {
                position: [-5.0, 5.0, -5.0, 1.0],
                color: [0, 0, 255, 255],
            },
            Vertex {
                position: [5.0, 5.0, -5.0, 1.0],
                color: [255, 0, 0, 255],
            },
            Vertex {
                position: [-5.0, -5.0, 5.0, 1.0],
                color: [0, 255, 255, 255],
            },
            Vertex {
                position: [5.0, -5.0, 5.0, 1.0],
                color: [0, 0, 255, 255],
            },
            Vertex {
                position: [-5.0, 5.0, 5.0, 1.0],
                color: [255, 255, 0, 255],
            },
            Vertex {
                position: [5.0, 5.0, 5.0, 1.0],
                color: [0, 255, 0, 255],
            },
        ];

        let vertex_buffer: Buffer<[Vertex], _> =
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

        let index_buffer: Buffer<[u16], _> =
            device.create_buffer(index_data, buffer::Usages::index());

        let view = glam::f32::Mat4::from_translation(glam::f32::Vec3::new(0.0, 0.0, 30.0))
            .inverse()
            .to_abi();
        let projection = glam::f32::Mat4::perspective_rh(
            0.3 * PI,
            size.width as f32 / size.height as f32,
            1.0,
            100.0,
        )
        .to_abi();
        let uniforms = Uniforms {
            model: glam::f32::Mat4::IDENTITY.to_abi(),
            view,
            projection,
        };

        let uniform_buffer =
            device.create_buffer(uniforms, buffer::Usages::uniform_binding().and_copy_dst());

        let bind_group = device.create_bind_group(
            &bind_group_layout,
            Resources {
                uniform_buffer: uniform_buffer.uniform(),
            },
        );

        let depth_texture = device.create_texture_2d(&Texture2DDescriptor {
            format: depth24plus,
            usage: texture::Usages::render_attachment(),
            view_formats: (),
            width: size.width,
            height: size.height,
            layers: 1,
            mipmap_levels: MipmapLevels::Partial(1),
        });

        let render_bundle_encoder = device.create_render_bundle_encoder(
            &RenderBundleEncoderDescriptor::new::<bgra8unorm>()
                .depth_stencil_format::<depth24plus>(),
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
            view,
            projection,
            uniform_buffer,
            depth_texture,
            render_bundle,
            window,
            surface,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(width, height);
        self.depth_texture = self.device.create_texture_2d(&Texture2DDescriptor {
            format: depth24plus,
            usage: texture::Usages::render_attachment(),
            view_formats: (),
            width,
            height,
            layers: 1,
            mipmap_levels: MipmapLevels::Partial(1),
        });
        self.projection =
            glam::f32::Mat4::perspective_rh(0.3 * PI, width as f32 / height as f32, 1.0, 100.0)
                .to_abi();
    }

    pub fn draw_frame(&self) {
        let AppState {
            device,
            view,
            projection,
            uniform_buffer,
            depth_texture,
            render_bundle,
            surface,
            ..
        } = self;

        let current_system_time = SystemTime::now();
        let duration_since_epoch = current_system_time.duration_since(UNIX_EPOCH).unwrap();
        let ms = duration_since_epoch.as_millis() as u16;

        let queue = device.queue();

        let rotate_x = glam::f32::Mat4::from_rotation_x(ms as f32 / 1000.0);
        let rotate_y = glam::f32::Mat4::from_rotation_y(ms as f32 / 1000.0);
        let model = rotate_y * rotate_x;
        let uniforms = Uniforms {
            model: model.to_abi(),
            view: *view,
            projection: *projection,
        };

        queue.write_buffer(uniform_buffer.view(), &uniforms);

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
                depth_stencil: DepthAttachment {
                    image: depth_texture.attachable_image(&AttachableImageDescriptor::default()),
                    load_op: LoadOp::Clear(DepthValue::ONE),
                    store_op: StoreOp::Discard,
                },
            }))
            .execute_bundle(&render_bundle)
            .end()
            .finish();

        queue.submit(command_buffer);

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

                    state.resize(width, height);
                }
            }
            WindowEvent::RedrawRequested => {
                let state = self.state.as_ref().unwrap();

                state.draw_frame();
                state.window.request_redraw();
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
