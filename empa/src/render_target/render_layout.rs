use crate::texture::format::{
    ColorRenderable, DepthStencilRenderable, MultisampleColorRenderable, TextureFormatId,
};
use std::marker;

pub struct RenderLayoutDescriptor<'a> {
    pub color_layout: &'a [TextureFormatId],
    pub depth_stencil_layout: Option<DepthStencilLayout>,
    pub samples: u8,
}

pub struct RenderLayout<C, Ds> {
    _marker: marker::PhantomData<(C, Ds)>,
}

pub struct MultisampleRenderLayout<C, Ds, const SAMPLES: u8> {
    _marker: marker::PhantomData<(C, Ds)>,
}

mod typed_color_layout_seal {
    pub trait Seal {}
}

pub trait TypedColorLayout: typed_color_layout_seal::Seal {
    const COLOR_FORMATS: &'static [TextureFormatId];
}

macro_rules! impl_typed_color_layout {
    ($($color:ident),*) => {
        impl<$($color),*> typed_color_layout_seal::Seal for ($($color),*) where $($color: ColorRenderable,)* {}
        impl<$($color),*> TypedColorLayout for ($($color),*) where $($color: ColorRenderable,)* {
            const COLOR_FORMATS: &'static [TextureFormatId] =  &[$($color::FORMAT_ID),*];
        }
    }
}

impl_typed_color_layout!(C0);
impl_typed_color_layout!(C0, C1);
impl_typed_color_layout!(C0, C1, C2);
impl_typed_color_layout!(C0, C1, C2, C3);
impl_typed_color_layout!(C0, C1, C2, C3, C4);
impl_typed_color_layout!(C0, C1, C2, C3, C4, C5);
impl_typed_color_layout!(C0, C1, C2, C3, C4, C5, C6);
impl_typed_color_layout!(C0, C1, C2, C3, C4, C5, C6, C7);

mod typed_multisample_color_layout_seal {
    pub trait Seal {}
}

pub trait TypedMultisampleColorLayout: typed_multisample_color_layout_seal::Seal {
    const COLOR_FORMATS: &'static [TextureFormatId];
}

macro_rules! impl_typed_multisample_color_layout {
    ($($color:ident),*) => {
        impl<$($color),*> typed_multisample_color_layout_seal::Seal for ($($color),*) where $($color: MultisampleColorRenderable,)* {}
        impl<$($color),*> TypedMultisampleColorLayout for ($($color),*) where $($color: MultisampleColorRenderable,)* {
            const COLOR_FORMATS: &'static [TextureFormatId] =  &[$($color::FORMAT_ID),*];
        }
    }
}

impl_typed_multisample_color_layout!(C0);
impl_typed_multisample_color_layout!(C0, C1);
impl_typed_multisample_color_layout!(C0, C1, C2);
impl_typed_multisample_color_layout!(C0, C1, C2, C3);
impl_typed_multisample_color_layout!(C0, C1, C2, C3, C4);
impl_typed_multisample_color_layout!(C0, C1, C2, C3, C4, C5);
impl_typed_multisample_color_layout!(C0, C1, C2, C3, C4, C5, C6);
impl_typed_multisample_color_layout!(C0, C1, C2, C3, C4, C5, C6, C7);

mod typed_depth_stencil_layout_seal {
    pub trait Seal {}
}

#[doc(hidden)]
pub struct ReadOnly<F> {
    _marker: marker::PhantomData<F>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DepthStencilLayout {
    pub format: TextureFormatId,
    pub read_only: bool,
}

pub trait TypedDepthStencilLayout: typed_depth_stencil_layout_seal::Seal {
    const LAYOUT: DepthStencilLayout;
}

impl<F> typed_depth_stencil_layout_seal::Seal for F where F: DepthStencilRenderable {}
impl<F> TypedDepthStencilLayout for F
where
    F: DepthStencilRenderable,
{
    const LAYOUT: DepthStencilLayout = DepthStencilLayout {
        format: F::FORMAT_ID,
        read_only: false,
    };
}

impl<F> typed_depth_stencil_layout_seal::Seal for ReadOnly<F> where F: DepthStencilRenderable {}
impl<F> TypedDepthStencilLayout for ReadOnly<F>
where
    F: DepthStencilRenderable,
{
    const LAYOUT: DepthStencilLayout = DepthStencilLayout {
        format: F::FORMAT_ID,
        read_only: true,
    };
}

mod typed_render_layout_seal {
    pub trait Seal {}
}

pub trait TypedRenderLayout: typed_render_layout_seal::Seal {
    const LAYOUT: RenderLayoutDescriptor<'static>;
}

impl<C, Ds> typed_render_layout_seal::Seal for RenderLayout<C, Ds>
where
    C: TypedColorLayout,
    Ds: TypedDepthStencilLayout,
{
}
impl<C, Ds> TypedRenderLayout for RenderLayout<C, Ds>
where
    C: TypedColorLayout,
    Ds: TypedDepthStencilLayout,
{
    const LAYOUT: RenderLayoutDescriptor<'static> = RenderLayoutDescriptor {
        color_layout: C::COLOR_FORMATS,
        depth_stencil_layout: Some(Ds::LAYOUT),
        samples: 1,
    };
}

impl<Ds> typed_render_layout_seal::Seal for RenderLayout<(), Ds> where Ds: TypedDepthStencilLayout {}
impl<Ds> TypedRenderLayout for RenderLayout<(), Ds>
where
    Ds: TypedDepthStencilLayout,
{
    const LAYOUT: RenderLayoutDescriptor<'static> = RenderLayoutDescriptor {
        color_layout: &[],
        depth_stencil_layout: Some(Ds::LAYOUT),
        samples: 1,
    };
}

impl<C> typed_render_layout_seal::Seal for RenderLayout<C, ()> where C: TypedColorLayout {}
impl<C> TypedRenderLayout for RenderLayout<C, ()>
where
    C: TypedColorLayout,
{
    const LAYOUT: RenderLayoutDescriptor<'static> = RenderLayoutDescriptor {
        color_layout: C::COLOR_FORMATS,
        depth_stencil_layout: None,
        samples: 1,
    };
}

impl<C, Ds, const SAMPLES: u8> typed_render_layout_seal::Seal
    for MultisampleRenderLayout<C, Ds, SAMPLES>
where
    C: TypedMultisampleColorLayout,
    Ds: TypedDepthStencilLayout,
{
}
impl<C, Ds, const SAMPLES: u8> TypedRenderLayout for MultisampleRenderLayout<C, Ds, SAMPLES>
where
    C: TypedMultisampleColorLayout,
    Ds: TypedDepthStencilLayout,
{
    const LAYOUT: RenderLayoutDescriptor<'static> = RenderLayoutDescriptor {
        color_layout: C::COLOR_FORMATS,
        depth_stencil_layout: Some(Ds::LAYOUT),
        samples: SAMPLES,
    };
}

impl<C, const SAMPLES: u8> typed_render_layout_seal::Seal
    for MultisampleRenderLayout<C, (), SAMPLES>
where
    C: TypedMultisampleColorLayout,
{
}
impl<C, const SAMPLES: u8> TypedRenderLayout for MultisampleRenderLayout<C, (), SAMPLES>
where
    C: TypedMultisampleColorLayout,
{
    const LAYOUT: RenderLayoutDescriptor<'static> = RenderLayoutDescriptor {
        color_layout: C::COLOR_FORMATS,
        depth_stencil_layout: None,
        samples: SAMPLES,
    };
}

impl<Ds, const SAMPLES: u8> typed_render_layout_seal::Seal
    for MultisampleRenderLayout<(), Ds, SAMPLES>
where
    Ds: TypedDepthStencilLayout,
{
}
impl<Ds, const SAMPLES: u8> TypedRenderLayout for MultisampleRenderLayout<(), Ds, SAMPLES>
where
    Ds: TypedDepthStencilLayout,
{
    const LAYOUT: RenderLayoutDescriptor<'static> = RenderLayoutDescriptor {
        color_layout: &[],
        depth_stencil_layout: Some(Ds::LAYOUT),
        samples: SAMPLES,
    };
}
