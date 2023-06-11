use std::marker;

use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::{
    GpuBlendComponent, GpuBlendFactor, GpuBlendOperation, GpuBlendState, GpuColorTargetState,
    GpuFragmentState,
};

use crate::pipeline_constants::PipelineConstants;
use crate::render_target::TypedColorLayout;
use crate::shader_module::{ShaderModule, ShaderSourceInternal};
use crate::texture::format::{Blendable, ColorRenderable};
use empa_reflect::ShaderStage;

pub enum BlendFactor {
    Zero,
    One,
    Src,
    OneMinusSrc,
    SrcAlpha,
    OneMinusSrcAlpha,
    Dst,
    OneMinusDst,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturated,
    Constant,
    OneMinusConstant,
}

impl BlendFactor {
    pub(crate) fn to_web_sys(&self) -> GpuBlendFactor {
        match self {
            BlendFactor::Zero => GpuBlendFactor::Zero,
            BlendFactor::One => GpuBlendFactor::One,
            BlendFactor::Src => GpuBlendFactor::Src,
            BlendFactor::OneMinusSrc => GpuBlendFactor::OneMinusSrc,
            BlendFactor::SrcAlpha => GpuBlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => GpuBlendFactor::OneMinusSrcAlpha,
            BlendFactor::Dst => GpuBlendFactor::Dst,
            BlendFactor::OneMinusDst => GpuBlendFactor::OneMinusDst,
            BlendFactor::DstAlpha => GpuBlendFactor::DstAlpha,
            BlendFactor::OneMinusDstAlpha => GpuBlendFactor::OneMinusDstAlpha,
            BlendFactor::SrcAlphaSaturated => GpuBlendFactor::SrcAlphaSaturated,
            BlendFactor::Constant => GpuBlendFactor::Constant,
            BlendFactor::OneMinusConstant => GpuBlendFactor::OneMinusConstant,
        }
    }
}

pub enum BlendComponent {
    Add {
        src_factor: BlendFactor,
        dst_factor: BlendFactor,
    },
    Subtract {
        src_factor: BlendFactor,
        dst_factor: BlendFactor,
    },
    ReverseSubtract {
        src_factor: BlendFactor,
        dst_factor: BlendFactor,
    },
    Min,
    Max,
}

impl BlendComponent {
    pub(crate) fn to_web_sys(&self) -> GpuBlendComponent {
        let mut blend = GpuBlendComponent::new();

        match self {
            BlendComponent::Add {
                src_factor,
                dst_factor,
            } => {
                blend.operation(GpuBlendOperation::Add);
                blend.src_factor(src_factor.to_web_sys());
                blend.dst_factor(dst_factor.to_web_sys());
            }
            BlendComponent::Subtract {
                src_factor,
                dst_factor,
            } => {
                blend.operation(GpuBlendOperation::Subtract);
                blend.src_factor(src_factor.to_web_sys());
                blend.dst_factor(dst_factor.to_web_sys());
            }
            BlendComponent::ReverseSubtract {
                src_factor,
                dst_factor,
            } => {
                blend.operation(GpuBlendOperation::ReverseSubtract);
                blend.src_factor(src_factor.to_web_sys());
                blend.dst_factor(dst_factor.to_web_sys());
            }
            BlendComponent::Min => {
                blend.operation(GpuBlendOperation::Min);
                blend.src_factor(GpuBlendFactor::One);
                blend.dst_factor(GpuBlendFactor::One);
            }
            BlendComponent::Max => {
                blend.operation(GpuBlendOperation::Max);
                blend.src_factor(GpuBlendFactor::One);
                blend.dst_factor(GpuBlendFactor::One);
            }
        }

        blend
    }
}

impl Default for BlendComponent {
    fn default() -> Self {
        BlendComponent::Add {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
        }
    }
}

pub struct BlendState {
    pub color: BlendComponent,
    pub alpha: BlendComponent,
}

impl BlendState {
    pub(crate) fn to_web_sys(&self) -> GpuBlendState {
        GpuBlendState::new(&self.alpha.to_web_sys(), &self.color.to_web_sys())
    }
}

// Modified from wgpu::ColorWrites
bitflags::bitflags! {
    /// Color write mask.
    ///
    /// Disabled color channels will not be written to.
    #[repr(transparent)]
    pub struct ColorWriteMask: u32 {
        /// Enable red channel writes
        const RED = 1 << 0;
        /// Enable green channel writes
        const GREEN = 1 << 1;
        /// Enable blue channel writes
        const BLUE = 1 << 2;
        /// Enable alpha channel writes
        const ALPHA = 1 << 3;
        /// Enable red, green, and blue channel writes
        const COLOR = Self::RED.bits | Self::GREEN.bits | Self::BLUE.bits;
        /// Enable writes to all channels.
        const ALL = Self::RED.bits | Self::GREEN.bits | Self::BLUE.bits | Self::ALPHA.bits;
    }
}

pub struct ColorOutput<F>
where
    F: ColorRenderable,
{
    pub format: F,
    pub write_mask: ColorWriteMask,
}

pub struct BlendedColorOutput<F>
where
    F: Blendable,
{
    pub format: F,
    pub blend_state: BlendState,
    pub write_mask: ColorWriteMask,
}

pub struct ColorOutputEncoding {
    inner: GpuColorTargetState,
}

mod typed_color_output_seal {
    pub trait Seal {}
}

pub trait TypedColorOutput: typed_color_output_seal::Seal {
    type Format: ColorRenderable;

    fn to_encoding(&self) -> ColorOutputEncoding;
}

impl<F> typed_color_output_seal::Seal for ColorOutput<F> where F: ColorRenderable {}
impl<F> TypedColorOutput for ColorOutput<F>
where
    F: ColorRenderable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorOutputEncoding {
        let mut inner = GpuColorTargetState::new(F::FORMAT_ID.to_web_sys());

        inner.write_mask(self.write_mask.bits());

        ColorOutputEncoding { inner }
    }
}

impl<F> typed_color_output_seal::Seal for BlendedColorOutput<F> where F: Blendable {}
impl<F> TypedColorOutput for BlendedColorOutput<F>
where
    F: Blendable,
{
    type Format = F;

    fn to_encoding(&self) -> ColorOutputEncoding {
        let mut inner = GpuColorTargetState::new(F::FORMAT_ID.to_web_sys());

        inner.blend(&self.blend_state.to_web_sys());
        inner.write_mask(self.write_mask.bits());

        ColorOutputEncoding { inner }
    }
}

mod typed_color_outputs_seal {
    pub trait Seal {}
}

pub trait TypedColorOutputs: typed_color_outputs_seal::Seal {
    type Layout: TypedColorLayout;

    type Encodings: Iterator<Item = ColorOutputEncoding>;

    fn encodings(&self) -> Self::Encodings;
}

macro_rules! impl_typed_color_outputs {
    ($n:literal, $($color:ident),*) => {
        #[allow(unused_parens)]
        impl<$($color),*> typed_color_outputs_seal::Seal for ($($color),*) where $($color: TypedColorOutput),* {}

        #[allow(unused_parens)]
        impl<$($color),*> TypedColorOutputs for ($($color),*) where $($color: TypedColorOutput),* {
            type Layout = ($($color::Format),*);

            type Encodings = <[ColorOutputEncoding; $n] as IntoIterator>::IntoIter;

            fn encodings(&self) -> Self::Encodings {
                #[allow(non_snake_case)]
                let ($($color),*) = self;

                [$($color.to_encoding()),*].into_iter()
            }
        }
    }
}

impl_typed_color_outputs!(1, C0);
impl_typed_color_outputs!(2, C0, C1);
impl_typed_color_outputs!(3, C0, C1, C2);
impl_typed_color_outputs!(4, C0, C1, C2, C3);
impl_typed_color_outputs!(5, C0, C1, C2, C3, C4);
impl_typed_color_outputs!(6, C0, C1, C2, C3, C4, C5);
impl_typed_color_outputs!(7, C0, C1, C2, C3, C4, C5, C6);
impl_typed_color_outputs!(8, C0, C1, C2, C3, C4, C5, C6, C7);

pub struct FragmentStage<O> {
    pub(crate) inner: GpuFragmentState,
    pub(crate) shader_meta: ShaderSourceInternal,
    entry_index: usize,
    _marker: marker::PhantomData<*const O>,
}

pub struct FragmentStageBuilder<O> {
    inner: GpuFragmentState,
    shader_meta: ShaderSourceInternal,
    entry_index: usize,
    has_constants: bool,
    _marker: marker::PhantomData<*const O>,
}

impl FragmentStageBuilder<()> {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let inner = GpuFragmentState::new(entry_point, &shader_module.inner, &JsValue::null());
        let shader_meta = shader_module.meta.clone();

        let entry_index = shader_meta
            .resolve_entry_point_index(entry_point)
            .expect("could not find entry point in shader module");
        let stage = shader_meta.entry_point_stage(entry_index);

        assert!(
            stage == Some(ShaderStage::Fragment),
            "entry point is not a fragment stage"
        );

        FragmentStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants: false,
            _marker: Default::default(),
        }
    }

    pub fn color_outputs<O: TypedColorOutputs>(
        self,
        color_outputs: O,
    ) -> FragmentStageBuilder<O::Layout> {
        let FragmentStageBuilder {
            mut inner,
            shader_meta,
            entry_index,
            has_constants,
            ..
        } = self;

        let layout = O::Layout::COLOR_FORMATS;

        let output_bindings = shader_meta
            .entry_point_output_bindings(entry_index)
            .unwrap();

        for binding in output_bindings {
            let location = binding.location();
            let binding_type = binding.binding_type();

            if let Some(format) = layout.get(location as usize) {
                // TODO: it's not clear from the spec what it means for a format to be compatible
                // with an output. Assuming for now that compatibility is solely about the main
                // component type (float, half-float, uint, sint) and not the number of components
                // (as this is how it works in OpenGL); needs confirmation.
                if binding_type.is_float() && !format.is_float() {
                    panic!(
                        "shader expects a float format binding for location `{}`",
                        location
                    );
                }

                if binding_type.is_half_float() && !format.is_half_float() {
                    panic!(
                        "shader expects a half-float format binding for location `{}`",
                        location
                    );
                }

                if binding_type.is_signed_integer() && !format.is_signed_integer() {
                    panic!(
                        "shader expects a signed integer format binding for location `{}`",
                        location
                    );
                }

                if binding_type.is_unsigned_integer() && !format.is_unsigned_integer() {
                    panic!(
                        "shader expects an unsigned integer format binding for location `{}`",
                        location
                    );
                }
            } else {
                panic!(
                    "shader expects an output binding for location `{}`",
                    location
                );
            }
        }

        let js_array: js_sys::Array = js_sys::Array::new();

        for encoding in color_outputs.encodings() {
            js_array.push(encoding.inner.as_ref());
        }

        inner.targets(js_array.as_ref());

        FragmentStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants,
            _marker: Default::default(),
        }
    }
}

impl<O> FragmentStageBuilder<O> {
    pub fn pipeline_constants<C: PipelineConstants>(
        self,
        pipeline_constants: &C,
    ) -> FragmentStageBuilder<O> {
        let FragmentStageBuilder {
            inner,
            shader_meta,
            entry_index,
            ..
        } = self;

        let record = shader_meta.build_constants(pipeline_constants);

        // TODO: get support for WebIDL record types in wasm bindgen
        js_sys::Reflect::set(inner.as_ref(), &JsValue::from("constants"), &record).unwrap_throw();

        FragmentStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants: true,
            _marker: Default::default(),
        }
    }
}

impl<O> FragmentStageBuilder<O>
where
    O: TypedColorLayout,
{
    pub fn finish(self) -> FragmentStage<O> {
        let FragmentStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants,
            ..
        } = self;

        if !has_constants && shader_meta.has_required_constants() {
            panic!("the shader declares pipeline constants without fallback values, but no pipeline constants were set");
        }

        FragmentStage {
            inner,
            shader_meta,
            entry_index,
            _marker: Default::default(),
        }
    }
}
