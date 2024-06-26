use crate::driver::{
    DepthStencilOperations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
};
use crate::render_target::{
    ColorTargetEncoding, DepthStencilTargetEncoding, DepthValue, LoadOp, StoreOp,
    TypedMultisampleColorLayout,
};
use crate::texture::format::{
    CombinedDepthStencilRenderable, DepthRenderable, DepthStencilRenderable,
    MultisampleColorRenderable, MultisampleFloatRenderable, MultisampleSignedIntegerRenderable,
    MultisampleUnsignedIntegerRenderable, Resolvable, StencilRenderable,
};
use crate::texture::{AttachableImage, AttachableMultisampledImage};

mod multisample_color_targets_seal {
    pub trait Seal<const SAMPLES: u8> {}
}

pub trait MultisampleColorTargets<const SAMPLES: u8>:
    multisample_color_targets_seal::Seal<SAMPLES>
{
    type Layout: TypedMultisampleColorLayout;

    type Encodings<'a>: IntoIterator<Item = ColorTargetEncoding<'a>>
    where
        Self: 'a;

    fn encodings<'a>(&'a self) -> Self::Encodings<'a>;
}

macro_rules! impl_multisample_color_targets {
    ($n:literal, $($A:ident),*) => {
        #[allow(unused_parens)]
        impl<$($A,)* const SAMPLES: u8> multisample_color_targets_seal::Seal<SAMPLES> for ($($A),*) where $($A: MultisampleColorTarget<SAMPLES>),* {}

        #[allow(unused_parens)]
        impl<$($A,)* const SAMPLES: u8> MultisampleColorTargets<SAMPLES> for ($($A),*) where $($A: MultisampleColorTarget<SAMPLES>),* {
            type Layout = ($($A::Format),*);
            type Encodings<'a> = [ColorTargetEncoding<'a>; $n] where Self: 'a;

            fn encodings<'a>(&'a self) -> Self::Encodings<'a> {
                #[allow(non_snake_case)]
                let ($($A),*) = self;

                [$($A.to_encoding()),*]
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
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
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
        let MultisampleFloatAttachment {
            image,
            load_op,
            store_op,
        } = self;

        ColorTargetEncoding {
            inner: Some(RenderPassColorAttachment {
                view: image.inner.clone(),
                resolve_target: None,
                load_op: load_op.to_4xf64(),
                store_op: *store_op,
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

pub struct MultisampleResolveAttachment<'a, F, const SAMPLES: u8>
where
    F: Resolvable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
    pub resolve: AttachableImage<'a, F>,
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
        let MultisampleResolveAttachment {
            image,
            resolve,
            load_op,
            store_op,
        } = self;

        if image.width != resolve.width || image.height != resolve.height {
            panic!("image and resolve target dimensions must match");
        }

        ColorTargetEncoding {
            inner: Some(RenderPassColorAttachment {
                view: image.inner.clone(),
                resolve_target: Some(resolve.inner.clone()),
                load_op: load_op.to_4xf64(),
                store_op: *store_op,
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

pub struct MultisampleSignedIntegerAttachment<'a, F, const SAMPLES: u8>
where
    F: MultisampleSignedIntegerRenderable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
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
        let MultisampleSignedIntegerAttachment {
            image,
            load_op,
            store_op,
        } = self;

        ColorTargetEncoding {
            inner: Some(RenderPassColorAttachment {
                view: image.inner.clone(),
                resolve_target: None,
                load_op: load_op.to_4xf64(),
                store_op: *store_op,
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

pub struct MultisampleUnsignedIntegerAttachment<'a, F, const SAMPLES: u8>
where
    F: MultisampleUnsignedIntegerRenderable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
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
        let MultisampleUnsignedIntegerAttachment {
            image,
            load_op,
            store_op,
        } = self;

        ColorTargetEncoding {
            inner: Some(RenderPassColorAttachment {
                view: image.inner.clone(),
                resolve_target: None,
                load_op: load_op.to_4xf64(),
                store_op: *store_op,
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

mod multisample_depth_stencil_target_seal {
    pub trait Seal {}
}

pub trait MultisampleDepthStencilTarget<const SAMPLES: u8>:
    multisample_depth_stencil_target_seal::Seal
{
    type Format: DepthStencilRenderable;

    fn to_encoding(&self) -> DepthStencilTargetEncoding;
}

pub struct MultisampleDepthStencilAttachment<'a, F, const SAMPLES: u8>
where
    F: CombinedDepthStencilRenderable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
    pub depth_load_op: LoadOp<DepthValue>,
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
        let MultisampleDepthStencilAttachment {
            image,
            depth_load_op,
            depth_store_op,
            stencil_load_op,
            stencil_store_op,
        } = self;

        DepthStencilTargetEncoding {
            inner: Some(RenderPassDepthStencilAttachment {
                view: image.inner.clone(),
                depth_operations: Some(DepthStencilOperations {
                    load_op: depth_load_op.to_f32(),
                    store_op: *depth_store_op,
                }),
                stencil_operations: Some(DepthStencilOperations {
                    load_op: *stencil_load_op,
                    store_op: *stencil_store_op,
                }),
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

pub struct MultisampleReadOnlyDepthStencilAttachment<'a, F, const SAMPLES: u8>
where
    F: CombinedDepthStencilRenderable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
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
        let MultisampleReadOnlyDepthStencilAttachment { image } = self;

        DepthStencilTargetEncoding {
            inner: Some(RenderPassDepthStencilAttachment {
                view: image.inner.clone(),
                depth_operations: None,
                stencil_operations: None,
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

pub struct MultisampleDepthAttachment<'a, F, const SAMPLES: u8>
where
    F: DepthStencilRenderable + DepthRenderable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
    pub load_op: LoadOp<DepthValue>,
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
        let MultisampleDepthAttachment {
            image,
            load_op,
            store_op,
        } = self;

        DepthStencilTargetEncoding {
            inner: Some(RenderPassDepthStencilAttachment {
                view: image.inner.clone(),
                depth_operations: Some(DepthStencilOperations {
                    load_op: load_op.to_f32(),
                    store_op: *store_op,
                }),
                stencil_operations: None,
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

pub struct MultisampleReadOnlyDepthAttachment<'a, F, const SAMPLES: u8>
where
    F: DepthRenderable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
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
        let MultisampleReadOnlyDepthAttachment { image } = self;

        DepthStencilTargetEncoding {
            inner: Some(RenderPassDepthStencilAttachment {
                view: image.inner.clone(),
                depth_operations: None,
                stencil_operations: None,
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

pub struct MultisampleStencilAttachment<'a, F, const SAMPLES: u8>
where
    F: StencilRenderable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
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
        let MultisampleStencilAttachment {
            image,
            load_op,
            store_op,
        } = self;

        DepthStencilTargetEncoding {
            inner: Some(RenderPassDepthStencilAttachment {
                view: image.inner.clone(),
                depth_operations: None,
                stencil_operations: Some(DepthStencilOperations {
                    load_op: *load_op,
                    store_op: *store_op,
                }),
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}

pub struct MultisampleReadOnlyStencilAttachment<'a, F, const SAMPLES: u8>
where
    F: StencilRenderable,
{
    pub image: AttachableMultisampledImage<'a, F, SAMPLES>,
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
        let MultisampleReadOnlyStencilAttachment { image } = self;

        DepthStencilTargetEncoding {
            inner: Some(RenderPassDepthStencilAttachment {
                view: image.inner.clone(),
                depth_operations: None,
                stencil_operations: None,
            }),
            width: image.width,
            height: image.height,
            _marker: Default::default(),
        }
    }
}
