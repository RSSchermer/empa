use std::marker;

use atomic_counter::AtomicCounter;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{GpuRenderPipeline, GpuRenderPipelineDescriptor};

use crate::device::{Device, ID_GEN};
use crate::render_pipeline::{
    DepthStencilTest, FragmentStage, IndexAny, MultisampleState, PipelineIndexFormat,
    PrimitiveAssembly, TypedVertexLayout, VertexStage,
};
use crate::render_target::{MultisampleRenderLayout, RenderLayout, TypedMultisampleColorLayout};
use crate::resource_binding::{PipelineLayout, ShaderStages, TypedPipelineLayout};

pub struct RenderPipeline<O, V, I, R> {
    inner: GpuRenderPipeline,
    id: usize,
    _marker: marker::PhantomData<(*const O, *const V, *const I, *const R)>,
}

impl<O, V, I, R> RenderPipeline<O, V, I, R> {
    pub(crate) fn new(device: &Device, descriptor: &RenderPipelineDescriptor<O, V, I, R>) -> Self {
        let id = ID_GEN.get();
        let inner = device.inner.create_render_pipeline(&descriptor.inner);

        RenderPipeline {
            inner,
            id,
            _marker: Default::default(),
        }
    }

    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn as_web_sys(&self) -> &GpuRenderPipeline {
        &self.inner
    }
}

pub struct RenderPipelineDescriptor<O, V, I, R> {
    inner: GpuRenderPipelineDescriptor,
    _marker: marker::PhantomData<(*const O, *const V, *const I, *const R)>,
}

pub struct RenderPipelineDescriptorBuilder<
    Multisample,
    Layout,
    Vertex,
    Fragment,
    DepthStencil,
    Primitives,
> {
    inner: GpuRenderPipelineDescriptor,
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
        let inner = GpuRenderPipelineDescriptor::new(&JsValue::null().unchecked_into());

        RenderPipelineDescriptorBuilder {
            inner,
            _marker: Default::default(),
        }
    }
}

impl<M, L, V, F, D, P> RenderPipelineDescriptorBuilder<M, L, V, F, D, P> {
    pub fn depth_stencil_test<Format>(
        mut self,
        depth_stencil_test: DepthStencilTest<Format>,
    ) -> RenderPipelineDescriptorBuilder<M, L, V, F, DepthStencilTest<Format>, P> {
        self.inner.depth_stencil(&depth_stencil_test.inner);

        RenderPipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }

    pub fn primitive_assembly<Format>(
        mut self,
        primitive_assembly: PrimitiveAssembly<Format>,
    ) -> RenderPipelineDescriptorBuilder<M, L, V, F, D, PrimitiveAssembly<Format>>
    where
        Format: PipelineIndexFormat,
    {
        self.inner.primitive(&primitive_assembly.inner);

        RenderPipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

impl<M, D, P> RenderPipelineDescriptorBuilder<M, (), (), (), D, P> {
    pub fn layout<Layout>(
        mut self,
        layout: &PipelineLayout<Layout>,
    ) -> RenderPipelineDescriptorBuilder<M, PipelineLayout<Layout>, (), (), D, P> {
        self.inner.layout(&layout.inner);

        RenderPipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

impl<L, V, D, P> RenderPipelineDescriptorBuilder<(), L, V, (), D, P> {
    pub fn multisample<const SAMPLES: u8>(
        mut self,
        multisample_state: MultisampleState<SAMPLES>,
    ) -> RenderPipelineDescriptorBuilder<MultisampleState<SAMPLES>, L, V, (), D, P> {
        self.inner.multisample(&multisample_state.inner);

        RenderPipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

impl<M, Layout, F, D, P> RenderPipelineDescriptorBuilder<M, PipelineLayout<Layout>, (), F, D, P>
where
    Layout: TypedPipelineLayout,
{
    pub fn vertex<VertexLayout: TypedVertexLayout>(
        mut self,
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

            if !entry.visibility.intersects(ShaderStages::VERTEX) {
                panic!(
                    "binding `{}` in group `{}` is not visible to the vertex stage",
                    resource_binding.binding, resource_binding.group
                );
            }

            if entry.binding_type != resource_binding.binding_type {
                panic!("the binding type for binding `{}` in group `{}` does not match the shader type", resource_binding.binding, resource_binding.group)
            }
        }

        self.inner.vertex(&vertex_stage.inner);

        RenderPipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

impl<M, Layout, V, D, P> RenderPipelineDescriptorBuilder<M, PipelineLayout<Layout>, V, (), D, P>
where
    Layout: TypedPipelineLayout,
{
    fn fragment_internal<ColorLayout>(
        mut self,
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

            if !entry.visibility.intersects(ShaderStages::VERTEX) {
                panic!(
                    "binding `{}` in group `{}` is not visible to the fragment stage",
                    resource_binding.binding, resource_binding.group
                );
            }

            if entry.binding_type != resource_binding.binding_type {
                panic!("the binding type for binding `{}` in group `{}` does not match the shader type", resource_binding.binding, resource_binding.group)
            }
        }

        self.inner.fragment(&fragment_stage.inner);

        RenderPipelineDescriptorBuilder {
            inner: self.inner,
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
            inner: self.inner,
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
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}
