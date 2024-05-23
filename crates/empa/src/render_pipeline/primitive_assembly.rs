use std::marker;

use crate::driver::{PrimitiveState, PrimitiveTopology};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndexFormat {
    U16,
    U32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

impl Default for FrontFace {
    fn default() -> Self {
        FrontFace::CounterClockwise
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CullMode {
    Front,
    Back,
}

mod pipeline_index_format_seal {
    pub trait Seal {}
}

pub trait PipelineIndexFormat: pipeline_index_format_seal::Seal {}

pub struct Index16 {}

impl pipeline_index_format_seal::Seal for Index16 {}
impl PipelineIndexFormat for Index16 {}

pub struct Index32 {}

impl pipeline_index_format_seal::Seal for Index32 {}
impl PipelineIndexFormat for Index32 {}

pub struct IndexAny {}

impl pipeline_index_format_seal::Seal for IndexAny {}
impl PipelineIndexFormat for IndexAny {}

pub trait StripIndexFormat: PipelineIndexFormat {
    const FORMAT: IndexFormat;
}

impl StripIndexFormat for Index16 {
    const FORMAT: IndexFormat = IndexFormat::U16;
}

impl StripIndexFormat for Index32 {
    const FORMAT: IndexFormat = IndexFormat::U32;
}

mod index_data_seal {
    pub trait Seal {}
}

pub trait IndexData: index_data_seal::Seal {
    const FORMAT: IndexFormat;
}

impl index_data_seal::Seal for u16 {}
impl IndexData for u16 {
    const FORMAT: IndexFormat = IndexFormat::U16;
}

impl index_data_seal::Seal for u32 {}
impl IndexData for u32 {
    const FORMAT: IndexFormat = IndexFormat::U32;
}

pub trait PipelineIndexFormatCompatible<I>: IndexData
where
    I: PipelineIndexFormat,
{
}

impl PipelineIndexFormatCompatible<Index16> for u16 {}

impl PipelineIndexFormatCompatible<Index32> for u32 {}

impl PipelineIndexFormatCompatible<IndexAny> for u16 {}
impl PipelineIndexFormatCompatible<IndexAny> for u32 {}

pub struct PrimitiveAssembly<I> {
    pub(crate) inner: PrimitiveState,
    _marker: marker::PhantomData<*const I>,
}

impl PrimitiveAssembly<()> {
    pub fn point_list() -> PrimitiveAssembly<IndexAny> {
        PrimitiveAssembly {
            inner: PrimitiveState {
                topology: PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: FrontFace::CounterClockwise,
                cull_mode: None,
            },
            _marker: Default::default(),
        }
    }

    pub fn line_list() -> PrimitiveAssembly<IndexAny> {
        PrimitiveAssembly {
            inner: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::CounterClockwise,
                cull_mode: None,
            },
            _marker: Default::default(),
        }
    }

    pub fn triangle_list() -> PrimitiveAssembly<IndexAny> {
        PrimitiveAssembly {
            inner: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::CounterClockwise,
                cull_mode: None,
            },
            _marker: Default::default(),
        }
    }

    pub fn line_strip<I: StripIndexFormat>() -> PrimitiveAssembly<I> {
        PrimitiveAssembly {
            inner: PrimitiveState {
                topology: PrimitiveTopology::LineStrip,
                strip_index_format: Some(I::FORMAT),
                front_face: FrontFace::CounterClockwise,
                cull_mode: None,
            },
            _marker: Default::default(),
        }
    }

    pub fn triangle_strip<I: StripIndexFormat>() -> PrimitiveAssembly<I> {
        PrimitiveAssembly {
            inner: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(I::FORMAT),
                front_face: FrontFace::CounterClockwise,
                cull_mode: None,
            },
            _marker: Default::default(),
        }
    }
}

impl<I> PrimitiveAssembly<I> {
    pub fn front_face(mut self, front_face: FrontFace) -> PrimitiveAssembly<I> {
        self.inner.front_face = front_face;

        self
    }

    pub fn cull_mode(mut self, cull_mode: CullMode) -> PrimitiveAssembly<I> {
        self.inner.cull_mode = Some(cull_mode);

        self
    }
}
