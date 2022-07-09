use std::marker;

use web_sys::{GpuCullMode, GpuFrontFace, GpuIndexFormat, GpuPrimitiveState, GpuPrimitiveTopology};

mod pipeline_index_format_seal {
    pub trait Seal {}
}

pub struct IndexFormatId {
    pub(crate) inner: GpuIndexFormat,
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
    const FORMAT_ID: IndexFormatId;
}

impl StripIndexFormat for Index16 {
    const FORMAT_ID: IndexFormatId = IndexFormatId {
        inner: GpuIndexFormat::Uint16,
    };
}

impl StripIndexFormat for Index32 {
    const FORMAT_ID: IndexFormatId = IndexFormatId {
        inner: GpuIndexFormat::Uint32,
    };
}

mod index_data_seal {
    pub trait Seal {}
}

pub trait IndexData: index_data_seal::Seal {
    const FORMAT_ID: IndexFormatId;
}

impl index_data_seal::Seal for u16 {}
impl IndexData for u16 {
    const FORMAT_ID: IndexFormatId = IndexFormatId {
        inner: GpuIndexFormat::Uint16,
    };
}

impl index_data_seal::Seal for u32 {}
impl IndexData for u32 {
    const FORMAT_ID: IndexFormatId = IndexFormatId {
        inner: GpuIndexFormat::Uint32,
    };
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

impl FrontFace {
    fn to_web_sys(&self) -> GpuFrontFace {
        match self {
            FrontFace::Clockwise => GpuFrontFace::Cw,
            FrontFace::CounterClockwise => GpuFrontFace::Ccw,
        }
    }
}

impl Default for FrontFace {
    fn default() -> Self {
        FrontFace::CounterClockwise
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CullMode {
    None,
    Front,
    Back,
}

impl CullMode {
    fn to_web_sys(&self) -> GpuCullMode {
        match self {
            CullMode::None => GpuCullMode::None,
            CullMode::Front => GpuCullMode::Front,
            CullMode::Back => GpuCullMode::Back,
        }
    }
}

impl Default for CullMode {
    fn default() -> Self {
        CullMode::None
    }
}

pub struct PrimitiveAssembly<I> {
    pub(crate) inner: GpuPrimitiveState,
    _marker: marker::PhantomData<*const I>,
}

impl PrimitiveAssembly<()> {
    pub fn point_list() -> PrimitiveAssembly<IndexAny> {
        let mut inner = GpuPrimitiveState::new();

        inner.topology(GpuPrimitiveTopology::PointList);

        PrimitiveAssembly {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn line_list() -> PrimitiveAssembly<IndexAny> {
        let mut inner = GpuPrimitiveState::new();

        inner.topology(GpuPrimitiveTopology::LineList);

        PrimitiveAssembly {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn triangle_list() -> PrimitiveAssembly<IndexAny> {
        let mut inner = GpuPrimitiveState::new();

        inner.topology(GpuPrimitiveTopology::TriangleList);

        PrimitiveAssembly {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn line_strip<I: StripIndexFormat>() -> PrimitiveAssembly<I> {
        let mut inner = GpuPrimitiveState::new();

        inner.topology(GpuPrimitiveTopology::LineStrip);
        inner.strip_index_format(I::FORMAT_ID.inner);

        PrimitiveAssembly {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn triangle_strip<I: StripIndexFormat>() -> PrimitiveAssembly<I> {
        let mut inner = GpuPrimitiveState::new();

        inner.topology(GpuPrimitiveTopology::TriangleStrip);
        inner.strip_index_format(I::FORMAT_ID.inner);

        PrimitiveAssembly {
            inner,
            _marker: Default::default(),
        }
    }
}

impl<I> PrimitiveAssembly<I> {
    pub fn front_face(self, front_face: FrontFace) -> PrimitiveAssembly<I> {
        let PrimitiveAssembly { mut inner, .. } = self;

        inner.front_face(front_face.to_web_sys());

        PrimitiveAssembly {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn cull_mode(self, cull_mode: CullMode) -> PrimitiveAssembly<I> {
        let PrimitiveAssembly { mut inner, .. } = self;

        inner.cull_mode(cull_mode.to_web_sys());

        PrimitiveAssembly {
            inner,
            _marker: Default::default(),
        }
    }
}
