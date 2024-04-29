use web_sys::{GpuRenderPassColorAttachment, GpuRenderPassDepthStencilAttachment};

use crate::render_target::{
    ColorTargetEncoding, DepthStencilTargetEncoding, DepthValue, LoadOp, StoreOp, TypedColorLayout,
};
use crate::texture::format::{
    ColorRenderable, CombinedDepthStencilRenderable, DepthRenderable, DepthStencilRenderable,
    FloatRenderable, SignedIntegerRenderable, StencilRenderable, UnsignedIntegerRenderable,
};
use crate::texture::AttachableImage;

mod color_targets_seal {
    pub trait Seal {}
}

pub trait ColorTargets: color_targets_seal::Seal {
    type Layout: TypedColorLayout;

    type Encodings: Iterator<Item = ColorTargetEncoding>;

    fn encodings(&self) -> Self::Encodings;
}

macro_rules! impl_color_targets {
    ($n:literal, $($A:ident),*) => {
        #[allow(unused_parens)]
        impl<$($A),*> color_targets_seal::Seal for ($($A),*) where $($A: ColorTarget),* {}

        #[allow(unused_parens)]
        impl<$($A),*> ColorTargets for ($($A),*) where $($A: ColorTarget),* {
            type Layout = ($($A::Format),*);
            type Encodings = <[ColorTargetEncoding; $n] as IntoIterator>::IntoIter;

            fn encodings(&self) -> Self::Encodings {
                #[allow(non_snake_case)]
                let ($($A),*) = self;

                [$($A.to_encoding()),*].into_iter()
            }
        }
    }
}

impl_color_targets!(1, A0);
impl_color_targets!(2, A0, A1);
impl_color_targets!(3, A0, A1, A2);
impl_color_targets!(4, A0, A1, A2, A3);
impl_color_targets!(5, A0, A1, A2, A3, A4);
impl_color_targets!(6, A0, A1, A2, A3, A4, A5);
impl_color_targets!(7, A0, A1, A2, A3, A4, A5, A6);
impl_color_targets!(8, A0, A1, A2, A3, A4, A5, A6, A7);

mod color_target_seal {
    pub trait Seal {}
}

pub trait ColorTarget: color_target_seal::Seal {
    type Format: ColorRenderable;

    fn to_encoding(&self) -> ColorTargetEncoding;
}

pub struct FloatAttachment<'a, F>
where
    F: FloatRenderable,
{
    pub image: &'a AttachableImage<F>,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

impl<'a, F> color_target_seal::Seal for FloatAttachment<'a, F> where F: FloatRenderable {}
impl<'a, F> ColorTarget for FloatAttachment<'a, F>
where
    F: FloatRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorTargetEncoding {
        let FloatAttachment {
            image,
            load_op,
            store_op,
        } = self;

        let mut inner = GpuRenderPassColorAttachment::new(
            load_op.op_to_web_sys(),
            store_op.to_web_sys(),
            &image.inner,
        );

        if let Some(value) = load_op.value_to_web_sys() {
            inner.clear_value(value.as_ref());
        }

        ColorTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}

pub struct SignedIntegerAttachment<'a, F>
where
    F: SignedIntegerRenderable,
{
    pub image: &'a AttachableImage<F>,
    pub load_op: LoadOp<[i32; 4]>,
    pub store_op: StoreOp,
}

impl<'a, F> color_target_seal::Seal for SignedIntegerAttachment<'a, F> where
    F: SignedIntegerRenderable
{
}
impl<'a, F> ColorTarget for SignedIntegerAttachment<'a, F>
where
    F: SignedIntegerRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorTargetEncoding {
        let SignedIntegerAttachment {
            image,
            load_op,
            store_op,
        } = self;

        let mut inner = GpuRenderPassColorAttachment::new(
            load_op.op_to_web_sys(),
            store_op.to_web_sys(),
            &image.inner,
        );

        if let Some(value) = load_op.value_to_web_sys() {
            inner.clear_value(value.as_ref());
        }

        ColorTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}

pub struct UnsignedIntegerAttachment<'a, F>
where
    F: UnsignedIntegerRenderable,
{
    pub image: &'a AttachableImage<F>,
    pub load_op: LoadOp<[u32; 4]>,
    pub store_op: StoreOp,
}

impl<'a, F> color_target_seal::Seal for UnsignedIntegerAttachment<'a, F> where
    F: UnsignedIntegerRenderable
{
}
impl<'a, F> ColorTarget for UnsignedIntegerAttachment<'a, F>
where
    F: UnsignedIntegerRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorTargetEncoding {
        let UnsignedIntegerAttachment {
            image,
            load_op,
            store_op,
        } = self;

        let mut inner = GpuRenderPassColorAttachment::new(
            load_op.op_to_web_sys(),
            store_op.to_web_sys(),
            &image.inner,
        );

        if let Some(value) = load_op.value_to_web_sys() {
            inner.clear_value(value.as_ref());
        }

        ColorTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}

mod depth_stencil_target_seal {
    pub trait Seal {}
}

pub trait DepthStencilTarget: depth_stencil_target_seal::Seal {
    type Format: DepthStencilRenderable;

    fn to_encoding(&self) -> DepthStencilTargetEncoding;
}

pub struct DepthStencilAttachment<'a, F>
where
    F: CombinedDepthStencilRenderable,
{
    pub image: &'a AttachableImage<F>,
    pub depth_load_op: LoadOp<DepthValue>,
    pub depth_store_op: StoreOp,
    pub stencil_load_op: LoadOp<u32>,
    pub stencil_store_op: StoreOp,
}

impl<'a, F> depth_stencil_target_seal::Seal for DepthStencilAttachment<'a, F> where
    F: CombinedDepthStencilRenderable
{
}
impl<'a, F> DepthStencilTarget for DepthStencilAttachment<'a, F>
where
    F: CombinedDepthStencilRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        let DepthStencilAttachment {
            image,
            depth_load_op,
            depth_store_op,
            stencil_load_op,
            stencil_store_op,
        } = self;

        let mut inner = GpuRenderPassDepthStencilAttachment::new(&image.inner);

        inner.depth_read_only(false);
        inner.depth_load_op(depth_load_op.op_to_web_sys());
        inner.depth_store_op(depth_store_op.to_web_sys());

        if let Some(value) = depth_load_op.value_to_web_sys() {
            inner.depth_clear_value(value);
        }

        inner.stencil_read_only(false);
        inner.stencil_load_op(stencil_load_op.op_to_web_sys());
        inner.stencil_store_op(stencil_store_op.to_web_sys());

        if let Some(value) = stencil_load_op.value_to_web_sys() {
            inner.stencil_clear_value(value);
        }

        DepthStencilTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}

pub struct ReadOnlyDepthStencilAttachment<'a, F>
where
    F: CombinedDepthStencilRenderable,
{
    pub image: &'a AttachableImage<F>,
}

impl<'a, F> depth_stencil_target_seal::Seal for ReadOnlyDepthStencilAttachment<'a, F> where
    F: CombinedDepthStencilRenderable
{
}
impl<'a, F> DepthStencilTarget for ReadOnlyDepthStencilAttachment<'a, F>
where
    F: CombinedDepthStencilRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        let ReadOnlyDepthStencilAttachment { image } = self;

        let mut inner = GpuRenderPassDepthStencilAttachment::new(&image.inner);

        inner.depth_read_only(true);
        inner.stencil_read_only(true);

        DepthStencilTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}

pub struct DepthAttachment<'a, F>
where
    F: DepthRenderable,
{
    pub image: &'a AttachableImage<F>,
    pub load_op: LoadOp<DepthValue>,
    pub store_op: StoreOp,
}

impl<'a, F> depth_stencil_target_seal::Seal for DepthAttachment<'a, F> where F: DepthRenderable {}
impl<'a, F> DepthStencilTarget for DepthAttachment<'a, F>
where
    F: DepthRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        let DepthAttachment {
            image,
            load_op,
            store_op,
        } = self;

        let mut inner = GpuRenderPassDepthStencilAttachment::new(&image.inner);

        inner.depth_read_only(false);
        inner.depth_load_op(load_op.op_to_web_sys());
        inner.depth_store_op(store_op.to_web_sys());

        if let Some(value) = load_op.value_to_web_sys() {
            inner.depth_clear_value(value);
        }

        DepthStencilTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}

pub struct ReadOnlyDepthAttachment<'a, F>
where
    F: DepthRenderable,
{
    pub image: &'a AttachableImage<F>,
}

impl<'a, F> depth_stencil_target_seal::Seal for ReadOnlyDepthAttachment<'a, F> where
    F: DepthRenderable
{
}
impl<'a, F> DepthStencilTarget for ReadOnlyDepthAttachment<'a, F>
where
    F: DepthRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        let ReadOnlyDepthAttachment { image } = self;

        let mut inner = GpuRenderPassDepthStencilAttachment::new(&image.inner);

        inner.depth_read_only(true);

        DepthStencilTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}

pub struct StencilAttachment<'a, F>
where
    F: StencilRenderable,
{
    pub image: &'a AttachableImage<F>,
    pub load_op: LoadOp<u32>,
    pub store_op: StoreOp,
}

impl<'a, F> depth_stencil_target_seal::Seal for StencilAttachment<'a, F> where F: StencilRenderable {}
impl<'a, F> DepthStencilTarget for StencilAttachment<'a, F>
where
    F: StencilRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        let StencilAttachment {
            image,
            load_op,
            store_op,
        } = self;

        let mut inner = GpuRenderPassDepthStencilAttachment::new(&image.inner);

        inner.stencil_read_only(false);
        inner.stencil_load_op(load_op.op_to_web_sys());
        inner.stencil_store_op(store_op.to_web_sys());

        if let Some(value) = load_op.value_to_web_sys() {
            inner.stencil_clear_value(value);
        }

        DepthStencilTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}

pub struct ReadOnlyStencilAttachment<'a, F>
where
    F: StencilRenderable,
{
    pub image: &'a AttachableImage<F>,
}

impl<'a, F> depth_stencil_target_seal::Seal for ReadOnlyStencilAttachment<'a, F> where
    F: StencilRenderable
{
}
impl<'a, F> DepthStencilTarget for ReadOnlyStencilAttachment<'a, F>
where
    F: StencilRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        let ReadOnlyStencilAttachment { image } = self;

        let mut inner = GpuRenderPassDepthStencilAttachment::new(&image.inner);

        inner.stencil_read_only(true);

        DepthStencilTargetEncoding {
            inner,
            width: image.width,
            height: image.height,
            _texture_handle: image.texture_handle.clone(),
        }
    }
}
