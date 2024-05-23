use std::marker;

use crate::driver::{DepthStencilState, StencilFaceState, StencilOperation};
use crate::render_target::ReadOnly;
use crate::texture::format::{
    depth16unorm, depth24plus, depth24plus_stencil8, depth32float, depth32float_stencil8, stencil8,
    DepthStencilTestFormat,
};
use crate::CompareFunction;

mod depth_stencil_test_seal {
    pub trait Seal {}
}

impl depth_stencil_test_seal::Seal for depth16unorm {}
impl depth_stencil_test_seal::Seal for depth24plus {}
impl depth_stencil_test_seal::Seal for depth32float {}
impl depth_stencil_test_seal::Seal for depth24plus_stencil8 {}
impl depth_stencil_test_seal::Seal for depth32float_stencil8 {}
impl depth_stencil_test_seal::Seal for stencil8 {}
impl depth_stencil_test_seal::Seal for ReadOnly<depth16unorm> {}
impl depth_stencil_test_seal::Seal for ReadOnly<depth24plus> {}
impl depth_stencil_test_seal::Seal for ReadOnly<depth32float> {}
impl depth_stencil_test_seal::Seal for ReadOnly<depth24plus_stencil8> {}
impl depth_stencil_test_seal::Seal for ReadOnly<depth32float_stencil8> {}
impl depth_stencil_test_seal::Seal for ReadOnly<stencil8> {}

pub trait DepthTest: depth_stencil_test_seal::Seal {}

impl DepthTest for depth16unorm {}
impl DepthTest for depth24plus {}
impl DepthTest for depth32float {}
impl DepthTest for depth24plus_stencil8 {}
impl DepthTest for depth32float_stencil8 {}
impl DepthTest for ReadOnly<depth16unorm> {}
impl DepthTest for ReadOnly<depth24plus> {}
impl DepthTest for ReadOnly<depth32float> {}
impl DepthTest for ReadOnly<depth24plus_stencil8> {}
impl DepthTest for ReadOnly<depth32float_stencil8> {}

pub trait StencilTest {}

impl StencilTest for depth24plus_stencil8 {}
impl StencilTest for depth32float_stencil8 {}
impl StencilTest for stencil8 {}
impl StencilTest for ReadOnly<depth24plus_stencil8> {}
impl StencilTest for ReadOnly<depth32float_stencil8> {}
impl StencilTest for ReadOnly<stencil8> {}

pub struct DepthStencilTest<F> {
    pub(crate) inner: DepthStencilState,
    _marker: marker::PhantomData<*const F>,
}

impl DepthStencilTest<()> {
    pub fn read_write<F>() -> DepthStencilTest<F>
    where
        F: DepthStencilTestFormat,
    {
        DepthStencilTest {
            inner: DepthStencilState {
                format: F::FORMAT_ID,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil_front: StencilFaceState {
                    compare: CompareFunction::Always,
                    depth_fail_op: StencilOperation::Keep,
                    fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::Keep,
                },
                stencil_back: StencilFaceState {
                    compare: CompareFunction::Always,
                    depth_fail_op: StencilOperation::Keep,
                    fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::Keep,
                },
                stencil_read_mask: 0xFFFFFFFF,
                stencil_write_mask: 0xFFFFFFFF,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            },
            _marker: Default::default(),
        }
    }

    pub fn read_only<F>() -> DepthStencilTest<ReadOnly<F>>
    where
        F: DepthStencilTestFormat,
    {
        DepthStencilTest {
            inner: DepthStencilState {
                format: F::FORMAT_ID,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Less,
                stencil_front: StencilFaceState {
                    compare: CompareFunction::Always,
                    depth_fail_op: StencilOperation::Keep,
                    fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::Keep,
                },
                stencil_back: StencilFaceState {
                    compare: CompareFunction::Always,
                    depth_fail_op: StencilOperation::Keep,
                    fail_op: StencilOperation::Keep,
                    pass_op: StencilOperation::Keep,
                },
                stencil_read_mask: 0xFFFFFFFF,
                stencil_write_mask: 0xFFFFFFFF,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            },
            _marker: Default::default(),
        }
    }
}

impl<F> DepthStencilTest<F>
where
    F: DepthTest,
{
    pub fn depth_compare(mut self, depth_compare: CompareFunction) -> Self {
        self.inner.depth_compare = depth_compare;

        self
    }

    pub fn depth_bias(mut self, depth_bias: i32) -> Self {
        self.inner.depth_bias = depth_bias;

        self
    }

    pub fn depth_bias_clamp(mut self, depth_bias_clamp: f32) -> Self {
        self.inner.depth_bias_clamp = depth_bias_clamp;

        self
    }

    pub fn depth_bias_slope_scale(mut self, depth_bias_slope_scale: f32) -> Self {
        self.inner.depth_bias_slope_scale = depth_bias_slope_scale;

        self
    }
}

impl<F> DepthStencilTest<F>
where
    F: StencilTest,
{
    pub fn stencil_read_mask(mut self, stencil_read_mask: u32) -> Self {
        self.inner.stencil_read_mask = stencil_read_mask;

        self
    }

    pub fn stencil_write_mask(mut self, stencil_write_mask: u32) -> Self {
        self.inner.stencil_write_mask = stencil_write_mask;

        self
    }

    pub fn stencil_front(mut self, stencil_front: StencilFaceState) -> Self {
        self.inner.stencil_front = stencil_front;

        self
    }

    pub fn stencil_back(mut self, stencil_back: StencilFaceState) -> Self {
        self.inner.stencil_back = stencil_back;

        self
    }
}
