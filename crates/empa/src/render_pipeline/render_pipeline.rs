use std::borrow::Borrow;
use std::future::Future;
use std::marker;

use atomic_counter::AtomicCounter;
use futures::FutureExt;

use crate::device::{Device, ID_GEN};
use crate::driver;
use crate::driver::{Device as _, Driver, Dvr, PrimitiveState, PrimitiveTopology, ShaderStage};
use crate::render_pipeline::{
    DepthStencilTest, FragmentStage, FragmentState, FrontFace, IndexAny, MultisampleState,
    PipelineIndexFormat, PrimitiveAssembly, TypedVertexLayout, VertexStage, VertexState,
};
use crate::render_target::{MultisampleRenderLayout, RenderLayout, TypedMultisampleColorLayout};
use crate::resource_binding::{PipelineLayout, TypedPipelineLayout};

pub struct RenderPipeline<O, V, I, R> {
    pub(crate) handle: <Dvr as Driver>::RenderPipelineHandle,
    id: usize,
    _marker: marker::PhantomData<(*const O, *const V, *const I, *const R)>,
}

impl<O, V, I, R> RenderPipeline<O, V, I, R> {
    pub(crate) fn new_sync(
        device: &Device,
        descriptor: &RenderPipelineDescriptor<O, V, I, R>,
    ) -> Self {
        let id = ID_GEN.get();
        let handle = device
            .device_handle
            .create_render_pipeline(&descriptor.to_driver());

        RenderPipeline {
            handle,
            id,
            _marker: Default::default(),
        }
    }

    pub(crate) fn new_async(
        device: &Device,
        descriptor: &RenderPipelineDescriptor<O, V, I, R>,
    ) -> impl Future<Output = Self> {
        device
            .device_handle
            .create_render_pipeline_async(&descriptor.to_driver())
            .map(|handle| {
                let id = ID_GEN.get();

                RenderPipeline {
                    handle,
                    id,
                    _marker: Default::default(),
                }
            })
    }

    pub(crate) fn id(&self) -> usize {
        self.id
    }
}

pub struct RenderPipelineDescriptor<O, V, I, R> {
    vertex_state: VertexState,
    layout: <Dvr as Driver>::PipelineLayoutHandle,
    primitive_state: driver::PrimitiveState,
    fragment_state: Option<FragmentState>,
    depth_stencil_state: Option<driver::DepthStencilState>,
    multisample_state: Option<driver::MultisampleState>,
    _marker: marker::PhantomData<(*const O, *const V, *const I, *const R)>,
}

impl<O, V, I, R> RenderPipelineDescriptor<O, V, I, R> {
    fn to_driver(&self) -> driver::RenderPipelineDescriptor<Dvr> {
        driver::RenderPipelineDescriptor {
            layout: &self.layout,
            primitive_state: &self.primitive_state,
            vertex_state: driver::VertexState {
                shader_module: &self.vertex_state.shader_module,
                entry_point: &self.vertex_state.entry_point,
                constants: &self.vertex_state.constants,
                vertex_buffer_layouts: self.vertex_state.vertex_buffer_layouts.borrow(),
            },
            depth_stencil_state: self.depth_stencil_state.as_ref(),
            fragment_state: self.fragment_state.as_ref().map(|f| driver::FragmentState {
                shader_module: &f.shader_module,
                entry_point: &f.entry_point,
                constants: &f.constants,
                targets: &f.targets,
            }),
            multisample_state: self.multisample_state.as_ref(),
        }
    }
}

pub struct RenderPipelineDescriptorBuilder<
    Multisample,
    Layout,
    Vertex,
    Fragment,
    DepthStencil,
    Primitives,
> {
    vertex_state: Option<VertexState>,
    fragment_state: Option<FragmentState>,
    layout: Option<<Dvr as Driver>::PipelineLayoutHandle>,
    primitive_state: driver::PrimitiveState,
    depth_stencil_state: Option<driver::DepthStencilState>,
    multisample_state: Option<driver::MultisampleState>,
    _marker: marker::PhantomData<(
        Multisample,
        Layout,
        Vertex,
        Fragment,
        DepthStencil,
        Primitives,
    )>,
}

impl
    RenderPipelineDescriptorBuilder<
        (),
        (),
        (),
        (),
        DepthStencilTest<()>,
        PrimitiveAssembly<IndexAny>,
    >
{
    pub fn begin() -> Self {
        RenderPipelineDescriptorBuilder {
            vertex_state: None,
            fragment_state: None,
            layout: None,
            primitive_state: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::CounterClockwise,
                cull_mode: None,
            },
            depth_stencil_state: None,
            multisample_state: None,
            _marker: Default::default(),
        }
    }
}

impl<M, L, V, F, D, P> RenderPipelineDescriptorBuilder<M, L, V, F, D, P> {
    pub fn depth_stencil_test<Format>(
        self,
        depth_stencil_test: DepthStencilTest<Format>,
    ) -> RenderPipelineDescriptorBuilder<M, L, V, F, DepthStencilTest<Format>, P> {
        RenderPipelineDescriptorBuilder {
            vertex_state: self.vertex_state,
            fragment_state: self.fragment_state,
            layout: self.layout,
            primitive_state: self.primitive_state,
            depth_stencil_state: Some(depth_stencil_test.inner),
            multisample_state: self.multisample_state,
            _marker: Default::default(),
        }
    }

    pub fn primitive_assembly<Format>(
        self,
        primitive_assembly: PrimitiveAssembly<Format>,
    ) -> RenderPipelineDescriptorBuilder<M, L, V, F, D, PrimitiveAssembly<Format>>
    where
        Format: PipelineIndexFormat,
    {
        RenderPipelineDescriptorBuilder {
            vertex_state: self.vertex_state,
            fragment_state: self.fragment_state,
            layout: self.layout,
            primitive_state: primitive_assembly.inner,
            depth_stencil_state: self.depth_stencil_state,
            multisample_state: self.multisample_state,
            _marker: Default::default(),
        }
    }
}

impl<M, D, P> RenderPipelineDescriptorBuilder<M, (), (), (), D, P> {
    pub fn layout<Layout>(
        self,
        layout: &PipelineLayout<Layout>,
    ) -> RenderPipelineDescriptorBuilder<M, PipelineLayout<Layout>, (), (), D, P> {
        RenderPipelineDescriptorBuilder {
            vertex_state: self.vertex_state,
            fragment_state: self.fragment_state,
            layout: Some(layout.handle.clone()),
            primitive_state: self.primitive_state,
            depth_stencil_state: self.depth_stencil_state,
            multisample_state: self.multisample_state,
            _marker: Default::default(),
        }
    }
}

impl<L, V, D, P> RenderPipelineDescriptorBuilder<(), L, V, (), D, P> {
    pub fn multisample<const SAMPLES: u8>(
        self,
        multisample_state: MultisampleState<SAMPLES>,
    ) -> RenderPipelineDescriptorBuilder<MultisampleState<SAMPLES>, L, V, (), D, P> {
        RenderPipelineDescriptorBuilder {
            vertex_state: self.vertex_state,
            fragment_state: self.fragment_state,
            layout: self.layout,
            primitive_state: self.primitive_state,
            depth_stencil_state: self.depth_stencil_state,
            multisample_state: Some(multisample_state.inner),
            _marker: Default::default(),
        }
    }
}

impl<M, Layout, F, D, P> RenderPipelineDescriptorBuilder<M, PipelineLayout<Layout>, (), F, D, P>
where
    Layout: TypedPipelineLayout,
{
    pub fn vertex<VertexLayout: TypedVertexLayout>(
        self,
        vertex_stage: VertexStage<VertexLayout>,
    ) -> RenderPipelineDescriptorBuilder<
        M,
        PipelineLayout<Layout>,
        VertexStage<VertexLayout>,
        F,
        D,
        P,
    > {
        let layout = Layout::BIND_GROUP_LAYOUTS;

        for resource_binding in vertex_stage.shader_meta.resource_bindings() {
            let group = if let Some(group) = layout.get(resource_binding.group as usize) {
                group
            } else {
                panic!("shader expects bind group `{}`", resource_binding.group);
            };

            let entry = if let Some(Some(entry)) = group.get(resource_binding.binding as usize) {
                entry
            } else {
                panic!(
                    "shader expects binding `{}` in group `{}`",
                    resource_binding.binding, resource_binding.group
                );
            };

            if !entry.visibility.contains(ShaderStage::Vertex) {
                panic!(
                    "binding `{}` in group `{}` is not visible to the vertex stage",
                    resource_binding.binding, resource_binding.group
                );
            }

            if entry.binding_type != resource_binding.binding_type {
                panic!(
                    "the binding type for binding `{}` in group `{}` does not match the shader \
                type (shader: {:#?}, actual: {:#?})",
                    resource_binding.binding,
                    resource_binding.group,
                    &resource_binding.binding_type,
                    &entry.binding_type
                )
            }
        }

        RenderPipelineDescriptorBuilder {
            vertex_state: Some(vertex_stage.vertex_state),
            fragment_state: self.fragment_state,
            layout: self.layout,
            primitive_state: self.primitive_state,
            depth_stencil_state: self.depth_stencil_state,
            multisample_state: self.multisample_state,
            _marker: Default::default(),
        }
    }
}

impl<M, Layout, V, D, P> RenderPipelineDescriptorBuilder<M, PipelineLayout<Layout>, V, (), D, P>
where
    Layout: TypedPipelineLayout,
{
    fn fragment_internal<ColorLayout>(
        self,
        fragment_stage: FragmentStage<ColorLayout>,
    ) -> RenderPipelineDescriptorBuilder<
        M,
        PipelineLayout<Layout>,
        V,
        FragmentStage<ColorLayout>,
        D,
        P,
    > {
        let layout = Layout::BIND_GROUP_LAYOUTS;

        for resource_binding in fragment_stage.shader_meta.resource_bindings() {
            let group = if let Some(group) = layout.get(resource_binding.group as usize) {
                group
            } else {
                panic!("shader expects bind group `{}`", resource_binding.group);
            };

            let entry = if let Some(Some(entry)) = group.get(resource_binding.binding as usize) {
                entry
            } else {
                panic!(
                    "shader expects binding `{}` in group `{}`",
                    resource_binding.binding, resource_binding.group
                );
            };

            if !entry.visibility.contains(ShaderStage::Fragment) {
                panic!(
                    "binding `{}` in group `{}` is not visible to the fragment stage",
                    resource_binding.binding, resource_binding.group
                );
            }

            if entry.binding_type != resource_binding.binding_type {
                panic!("the binding type for binding `{}` in group `{}` does not match the shader type", resource_binding.binding, resource_binding.group)
            }
        }

        RenderPipelineDescriptorBuilder {
            vertex_state: self.vertex_state,
            fragment_state: Some(fragment_stage.fragment_state),
            layout: self.layout,
            primitive_state: self.primitive_state,
            depth_stencil_state: self.depth_stencil_state,
            multisample_state: self.multisample_state,
            _marker: Default::default(),
        }
    }
}

impl<Layout, V, D, P> RenderPipelineDescriptorBuilder<(), PipelineLayout<Layout>, V, (), D, P>
where
    Layout: TypedPipelineLayout,
{
    pub fn fragment<ColorLayout>(
        self,
        fragment_stage: FragmentStage<ColorLayout>,
    ) -> RenderPipelineDescriptorBuilder<
        (),
        PipelineLayout<Layout>,
        V,
        FragmentStage<ColorLayout>,
        D,
        P,
    > {
        self.fragment_internal(fragment_stage)
    }
}

impl<Layout, V, D, P, const SAMPLES: u8>
    RenderPipelineDescriptorBuilder<MultisampleState<SAMPLES>, PipelineLayout<Layout>, V, (), D, P>
where
    Layout: TypedPipelineLayout,
{
    pub fn fragment<ColorLayout: TypedMultisampleColorLayout>(
        self,
        fragment_stage: FragmentStage<ColorLayout>,
    ) -> RenderPipelineDescriptorBuilder<
        MultisampleState<SAMPLES>,
        PipelineLayout<Layout>,
        V,
        FragmentStage<ColorLayout>,
        D,
        P,
    > {
        self.fragment_internal(fragment_stage)
    }
}

impl<Layout, Vertex, Color, DepthStencil, Index>
    RenderPipelineDescriptorBuilder<
        (),
        PipelineLayout<Layout>,
        VertexStage<Vertex>,
        FragmentStage<Color>,
        DepthStencilTest<DepthStencil>,
        PrimitiveAssembly<Index>,
    >
{
    pub fn finish(
        self,
    ) -> RenderPipelineDescriptor<RenderLayout<Color, DepthStencil>, Vertex, Index, Layout> {
        RenderPipelineDescriptor {
            vertex_state: self.vertex_state.unwrap(),
            layout: self.layout.unwrap(),
            primitive_state: self.primitive_state,
            fragment_state: self.fragment_state,
            depth_stencil_state: self.depth_stencil_state,
            multisample_state: self.multisample_state,
            _marker: Default::default(),
        }
    }
}

impl<Layout, Vertex, Color, DepthStencil, Index, const SAMPLES: u8>
    RenderPipelineDescriptorBuilder<
        MultisampleState<SAMPLES>,
        PipelineLayout<Layout>,
        VertexStage<Vertex>,
        FragmentStage<Color>,
        DepthStencilTest<DepthStencil>,
        PrimitiveAssembly<Index>,
    >
{
    pub fn finish(
        self,
    ) -> RenderPipelineDescriptor<
        MultisampleRenderLayout<Color, DepthStencil, SAMPLES>,
        Vertex,
        Index,
        Layout,
    > {
        RenderPipelineDescriptor {
            vertex_state: self.vertex_state.unwrap(),
            layout: self.layout.unwrap(),
            primitive_state: self.primitive_state,
            fragment_state: self.fragment_state,
            depth_stencil_state: self.depth_stencil_state,
            multisample_state: self.multisample_state,
            _marker: Default::default(),
        }
    }
}
