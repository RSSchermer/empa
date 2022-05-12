use crate::render_target::ReadOnly;
use crate::texture::format::DepthStencilFormat;
use crate::CompareFunction;
use std::marker;
use web_sys::{GpuDepthStencilState, GpuStencilFaceState, GpuStencilOperation};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StencilFaceState {
    pub compare: CompareFunction,
    pub depth_fail_op: StencilOperation,
    pub fail_op: StencilOperation,
    pub pass_op: StencilOperation,
}

impl StencilFaceState {
    fn to_web_sys(&self) -> GpuStencilFaceState {
        let StencilFaceState {
            compare,
            depth_fail_op,
            fail_op,
            pass_op,
        } = *self;

        let mut state = GpuStencilFaceState::new();

        state.compare(compare.to_web_sys());
        state.depth_fail_op(depth_fail_op.to_web_sys());
        state.fail_op(depth_fail_op.to_web_sys());
        state.pass_op(depth_fail_op.to_web_sys());

        state
    }
}

impl Default for StencilFaceState {
    fn default() -> Self {
        StencilFaceState {
            compare: CompareFunction::Always,
            depth_fail_op: StencilOperation::Keep,
            fail_op: StencilOperation::Keep,
            pass_op: StencilOperation::Keep,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StencilOperation {
    Keep,
    Zero,
    Replace,
    Invert,
    IncrementClamp,
    DecrementClamp,
    IncrementWrap,
    DecrementWrap,
}

impl StencilOperation {
    fn to_web_sys(&self) -> GpuStencilOperation {
        match self {
            StencilOperation::Keep => GpuStencilOperation::Keep,
            StencilOperation::Zero => GpuStencilOperation::Zero,
            StencilOperation::Replace => GpuStencilOperation::Replace,
            StencilOperation::Invert => GpuStencilOperation::Invert,
            StencilOperation::IncrementClamp => GpuStencilOperation::IncrementClamp,
            StencilOperation::DecrementClamp => GpuStencilOperation::DecrementClamp,
            StencilOperation::IncrementWrap => GpuStencilOperation::IncrementWrap,
            StencilOperation::DecrementWrap => GpuStencilOperation::DecrementWrap,
        }
    }
}

pub struct DepthStencilTest<F> {
    pub(crate) inner: GpuDepthStencilState,
    _marker: marker::PhantomData<*const F>,
}

impl DepthStencilTest<()> {
    pub fn read_write<F>() -> DepthStencilTest<F>
    where
        F: DepthStencilFormat,
    {
        let mut inner = GpuDepthStencilState::new(F::FORMAT_ID.to_web_sys());

        inner.depth_write_enabled(true);

        DepthStencilTest {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn read_only<F>() -> DepthStencilTest<ReadOnly<F>>
    where
        F: DepthStencilFormat,
    {
        let inner = GpuDepthStencilState::new(F::FORMAT_ID.to_web_sys());

        DepthStencilTest {
            inner,
            _marker: Default::default(),
        }
    }
}

impl<F> DepthStencilTest<F> {
    pub fn depth_compare(mut self, depth_compare: CompareFunction) -> Self {
        self.inner.depth_compare(depth_compare.to_web_sys());

        self
    }

    pub fn depth_bias(mut self, depth_bias: i32) -> Self {
        self.inner.depth_bias(depth_bias);

        self
    }

    pub fn depth_bias_clamp(mut self, depth_bias_clamp: f32) -> Self {
        self.inner.depth_bias_clamp(depth_bias_clamp);

        self
    }

    pub fn depth_bias_slope_scale(mut self, depth_bias_slope_scale: f32) -> Self {
        self.inner.depth_bias_slope_scale(depth_bias_slope_scale);

        self
    }

    pub fn stencil_read_mask(mut self, stencil_read_mask: u32) -> Self {
        self.inner.stencil_read_mask(stencil_read_mask);

        self
    }
}

impl<F> DepthStencilTest<F>
where
    F: DepthStencilFormat,
{
    pub fn stencil_front(mut self, stencil_front: StencilFaceState) -> Self {
        self.inner.stencil_front(&stencil_front.to_web_sys());

        self
    }

    pub fn stencil_back(mut self, stencil_back: StencilFaceState) -> Self {
        self.inner.stencil_back(&stencil_back.to_web_sys());

        self
    }

    pub fn stencil_write_mask(mut self, stencil_write_mask: u32) -> Self {
        self.inner.stencil_write_mask(stencil_write_mask);

        self
    }
}

impl<F> DepthStencilTest<ReadOnly<F>>
where
    F: DepthStencilFormat,
{
    pub fn stencil_compare_front(mut self, stencil_compare_front: CompareFunction) -> Self {
        let mut stencil_state = GpuStencilFaceState::new();

        stencil_state.compare(stencil_compare_front.to_web_sys());

        self.inner.stencil_front(&stencil_state);

        self
    }

    pub fn stencil_compare_back(mut self, stencil_compare_back: CompareFunction) -> Self {
        let mut stencil_state = GpuStencilFaceState::new();

        stencil_state.compare(stencil_compare_back.to_web_sys());

        self.inner.stencil_back(&stencil_state);

        self
    }
}
