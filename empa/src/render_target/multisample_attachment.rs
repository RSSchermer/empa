use crate::render_target::{
    ColorTargetEncoding, DepthStencilTargetEncoding, LoadOp, StoreOp, TypedMultisampleColorLayout,
    TypedRenderLayout,
};
use crate::texture::format::{
    ColorRenderable, CombinedDepthStencilRenderable, DepthRenderable, DepthStencilRenderable,
    FloatRenderable, MultisampleColorRenderable, MultisampleFloatRenderable,
    MultisampleSignedIntegerRenderable, MultisampleUnsignedIntegerRenderable, Resolvable,
    SignedIntegerRenderable, StencilRenderable, UnsignedIntegerRenderable,
};
use crate::texture::{AttachableImage, AttachableMultisampledImage};
use web_sys::{GpuRenderPassColorAttachment, GpuRenderPassDepthStencilAttachment};

mod multisample_color_targets_seal {
    pub trait Seal<const SAMPLES: u8> {}
}

pub trait MultisampleColorTargets<const SAMPLES: u8>:
    multisample_color_targets_seal::Seal<SAMPLES>
{
    type Layout: TypedMultisampleColorLayout;

    type Encodings: Iterator<Item = ColorTargetEncoding>;

    fn encodings(&self) -> Self::Encodings;
}

macro_rules! impl_multisample_color_targets {
    ($n:literal, $($A:ident),*) => {
        impl<$($A,)* const SAMPLES: u8> multisample_color_targets_seal::Seal<SAMPLES> for ($($A),*) where $($A: MultisampleColorTarget<SAMPLES>),* {}
        impl<$($A,)* const SAMPLES: u8> MultisampleColorTargets<SAMPLES> for ($($A),*) where $($A: MultisampleColorTarget<SAMPLES>),* {
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

impl_multisample_color_targets!(1, A0);
impl_multisample_color_targets!(2, A0, A1);
impl_multisample_color_targets!(3, A0, A1, A2);
impl_multisample_color_targets!(4, A0, A1, A2, A3);
impl_multisample_color_targets!(5, A0, A1, A2, A3, A4);
impl_multisample_color_targets!(6, A0, A1, A2, A3, A4, A5);
impl_multisample_color_targets!(7, A0, A1, A2, A3, A4, A5, A6);
impl_multisample_color_targets!(8, A0, A1, A2, A3, A4, A5, A6, A7);

mod multisample_color_target_seal {
    pub trait Seal {}
}

pub trait MultisampleColorTarget<const SAMPLES: u8>: multisample_color_target_seal::Seal {
    type Format: MultisampleColorRenderable;

    fn to_encoding(&self) -> ColorTargetEncoding;
}

pub struct MultisampleFloatAttachment<'a, F, const SAMPLES: u8>
where
    F: MultisampleFloatRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

impl<'a, F, const SAMPLES: u8> multisample_color_target_seal::Seal
    for MultisampleFloatAttachment<'a, F, SAMPLES>
where
    F: MultisampleFloatRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleColorTarget<SAMPLES>
    for MultisampleFloatAttachment<'a, F, SAMPLES>
where
    F: MultisampleFloatRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorTargetEncoding {
        todo!()
    }
}

pub struct MultisampleResolveAttachment<'a, F, const SAMPLES: u8>
where
    F: Resolvable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
    pub resolve: &'a AttachableImage<F>,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

impl<'a, F, const SAMPLES: u8> multisample_color_target_seal::Seal
    for MultisampleResolveAttachment<'a, F, SAMPLES>
where
    F: Resolvable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleColorTarget<SAMPLES>
    for MultisampleResolveAttachment<'a, F, SAMPLES>
where
    F: Resolvable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorTargetEncoding {
        todo!()
    }
}

pub struct MultisampleSignedIntegerAttachment<'a, F, const SAMPLES: u8>
where
    F: MultisampleSignedIntegerRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

impl<'a, F, const SAMPLES: u8> multisample_color_target_seal::Seal
    for MultisampleSignedIntegerAttachment<'a, F, SAMPLES>
where
    F: MultisampleSignedIntegerRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleColorTarget<SAMPLES>
    for MultisampleSignedIntegerAttachment<'a, F, SAMPLES>
where
    F: MultisampleSignedIntegerRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorTargetEncoding {
        todo!()
    }
}

pub struct MultisampleUnsignedIntegerAttachment<'a, F, const SAMPLES: u8>
where
    F: MultisampleUnsignedIntegerRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

impl<'a, F, const SAMPLES: u8> multisample_color_target_seal::Seal
    for MultisampleUnsignedIntegerAttachment<'a, F, SAMPLES>
where
    F: MultisampleUnsignedIntegerRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleColorTarget<SAMPLES>
    for MultisampleUnsignedIntegerAttachment<'a, F, SAMPLES>
where
    F: MultisampleUnsignedIntegerRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorTargetEncoding {
        todo!()
    }
}

mod multisample_depth_stencil_target_seal {
    pub trait Seal {}
}

pub trait MultisampleDepthStencilTarget<const SAMPLES: u8>:
    multisample_depth_stencil_target_seal::Seal
{
    type Format: DepthStencilRenderable;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        todo!()
    }
}

pub struct MultisampleDepthStencilAttachment<'a, F, const SAMPLES: u8>
where
    F: CombinedDepthStencilRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
    pub depth_load_op: LoadOp<f32>,
    pub depth_store_op: StoreOp,
    pub stencil_load_op: LoadOp<u32>,
    pub stencil_store_op: StoreOp,
}

impl<'a, F, const SAMPLES: u8> multisample_depth_stencil_target_seal::Seal
    for MultisampleDepthStencilAttachment<'a, F, SAMPLES>
where
    F: CombinedDepthStencilRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleDepthStencilTarget<SAMPLES>
    for MultisampleDepthStencilAttachment<'a, F, SAMPLES>
where
    F: CombinedDepthStencilRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        todo!()
    }
}

pub struct MultisampleReadOnlyDepthStencilAttachment<'a, F, const SAMPLES: u8>
where
    F: CombinedDepthStencilRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
}

impl<'a, F, const SAMPLES: u8> multisample_depth_stencil_target_seal::Seal
    for MultisampleReadOnlyDepthStencilAttachment<'a, F, SAMPLES>
where
    F: CombinedDepthStencilRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleDepthStencilTarget<SAMPLES>
    for MultisampleReadOnlyDepthStencilAttachment<'a, F, SAMPLES>
where
    F: CombinedDepthStencilRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        todo!()
    }
}

pub struct MultisampleDepthAttachment<'a, F, const SAMPLES: u8>
where
    F: DepthStencilRenderable + DepthRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
    pub load_op: LoadOp<f32>,
    pub store_op: StoreOp,
}

impl<'a, F, const SAMPLES: u8> multisample_depth_stencil_target_seal::Seal
    for MultisampleDepthAttachment<'a, F, SAMPLES>
where
    F: DepthRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleDepthStencilTarget<SAMPLES>
    for MultisampleDepthAttachment<'a, F, SAMPLES>
where
    F: DepthRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        todo!()
    }
}

pub struct MultisampleReadOnlyDepthAttachment<'a, F, const SAMPLES: u8>
where
    F: DepthRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
}

impl<'a, F, const SAMPLES: u8> multisample_depth_stencil_target_seal::Seal
    for MultisampleReadOnlyDepthAttachment<'a, F, SAMPLES>
where
    F: DepthRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleDepthStencilTarget<SAMPLES>
    for MultisampleReadOnlyDepthAttachment<'a, F, SAMPLES>
where
    F: DepthRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        todo!()
    }
}

pub struct MultisampleStencilAttachment<'a, F, const SAMPLES: u8>
where
    F: StencilRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
    pub load_op: LoadOp<u32>,
    pub store_op: StoreOp,
}

impl<'a, F, const SAMPLES: u8> multisample_depth_stencil_target_seal::Seal
    for MultisampleStencilAttachment<'a, F, SAMPLES>
where
    F: StencilRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleDepthStencilTarget<SAMPLES>
    for MultisampleStencilAttachment<'a, F, SAMPLES>
where
    F: StencilRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        todo!()
    }
}

pub struct MultisampleReadOnlyStencilAttachment<'a, F, const SAMPLES: u8>
where
    F: StencilRenderable,
{
    pub image: &'a AttachableMultisampledImage<F, SAMPLES>,
}

impl<'a, F, const SAMPLES: u8> multisample_depth_stencil_target_seal::Seal
    for MultisampleReadOnlyStencilAttachment<'a, F, SAMPLES>
where
    F: StencilRenderable,
{
}
impl<'a, F, const SAMPLES: u8> MultisampleDepthStencilTarget<SAMPLES>
    for MultisampleReadOnlyStencilAttachment<'a, F, SAMPLES>
where
    F: StencilRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> DepthStencilTargetEncoding {
        todo!()
    }
}
