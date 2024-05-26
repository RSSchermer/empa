use std::collections::HashMap;
use std::marker;

use empa_reflect::ShaderStage;
use flagset::{flags, FlagSet};

use crate::driver::{ColorTargetState, Driver, Dvr};
use crate::pipeline_constants::PipelineConstants;
use crate::render_target::TypedColorLayout;
use crate::shader_module::{ShaderModule, ShaderSourceInternal};
use crate::texture::format::{Blendable, ColorRenderable};

flags! {
    pub enum ColorWrite: u32 {
        Red   = 0x0001,
        Green = 0x0002,
        Blue  = 0x0004,
        Alpha = 0x0008,
        Color = (ColorWrite::Red | ColorWrite::Green | ColorWrite::Blue).bits(),
        All   = (ColorWrite::Color | ColorWrite::Alpha).bits(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl Default for BlendComponent {
    fn default() -> Self {
        BlendComponent::Add {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlendState {
    pub color: BlendComponent,
    pub alpha: BlendComponent,
}

pub struct ColorOutput<F, W>
where
    F: ColorRenderable,
    W: Into<FlagSet<ColorWrite>> + Copy,
{
    pub format: F,
    pub write_mask: W,
}

pub struct BlendedColorOutput<F, W>
where
    F: Blendable,
    W: Into<FlagSet<ColorWrite>> + Copy,
{
    pub format: F,
    pub blend_state: BlendState,
    pub write_mask: W,
}

mod typed_color_output_seal {
    pub trait Seal {}
}

pub trait TypedColorOutput: typed_color_output_seal::Seal {
    type Format: ColorRenderable;

    fn to_color_target_state(&self) -> ColorTargetState;
}

impl<F, W> typed_color_output_seal::Seal for ColorOutput<F, W>
where
    F: ColorRenderable,
    W: Into<FlagSet<ColorWrite>> + Copy,
{
}
impl<F, W> TypedColorOutput for ColorOutput<F, W>
where
    F: ColorRenderable,
    W: Into<FlagSet<ColorWrite>> + Copy,
{
    type Format = F;

    fn to_color_target_state(&self) -> ColorTargetState {
        ColorTargetState {
            format: F::FORMAT_ID,
            blend: None,
            write_mask: self.write_mask.into(),
        }
    }
}

impl<F, W> typed_color_output_seal::Seal for BlendedColorOutput<F, W>
where
    F: Blendable,
    W: Into<FlagSet<ColorWrite>> + Copy,
{
}
impl<F, W> TypedColorOutput for BlendedColorOutput<F, W>
where
    F: Blendable,
    W: Into<FlagSet<ColorWrite>> + Copy,
{
    type Format = F;

    fn to_color_target_state(&self) -> ColorTargetState {
        ColorTargetState {
            format: F::FORMAT_ID,
            blend: Some(self.blend_state),
            write_mask: self.write_mask.into(),
        }
    }
}

mod typed_color_outputs_seal {
    pub trait Seal {}
}

pub trait TypedColorOutputs: typed_color_outputs_seal::Seal {
    type Layout: TypedColorLayout;

    type Targets: Iterator<Item = ColorTargetState>;

    fn targets(&self) -> Self::Targets;
}

macro_rules! impl_typed_color_outputs {
    ($n:literal, $($color:ident),*) => {
        #[allow(unused_parens)]
        impl<$($color),*> typed_color_outputs_seal::Seal for ($($color),*) where $($color: TypedColorOutput),* {}

        #[allow(unused_parens)]
        impl<$($color),*> TypedColorOutputs for ($($color),*) where $($color: TypedColorOutput),* {
            type Layout = ($($color::Format),*);

            type Targets = <[ColorTargetState; $n] as IntoIterator>::IntoIter;

            fn targets(&self) -> Self::Targets {
                #[allow(non_snake_case)]
                let ($($color),*) = self;

                [$($color.to_color_target_state()),*].into_iter()
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

pub(crate) struct FragmentState {
    pub(crate) shader_module: <Dvr as Driver>::ShaderModuleHandle,
    pub(crate) entry_point: String,
    pub(crate) constants: HashMap<String, f64>,
    pub(crate) targets: Vec<ColorTargetState>,
}

pub struct FragmentStage<O> {
    pub(crate) fragment_state: FragmentState,
    pub(crate) shader_meta: ShaderSourceInternal,
    entry_index: usize,
    _marker: marker::PhantomData<*const O>,
}

pub struct FragmentStageBuilder<O> {
    inner: FragmentStage<O>,
    has_constants: bool,
}

impl FragmentStageBuilder<()> {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
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
            inner: FragmentStage {
                fragment_state: FragmentState {
                    shader_module: shader_module.handle.clone(),
                    entry_point: entry_point.to_string(),
                    constants: Default::default(),
                    targets: vec![],
                },
                shader_meta,
                entry_index,
                _marker: Default::default(),
            },
            has_constants: false,
        }
    }

    pub fn color_outputs<O: TypedColorOutputs>(
        mut self,
        color_outputs: O,
    ) -> FragmentStageBuilder<O::Layout> {
        let layout = O::Layout::COLOR_FORMATS;

        let output_bindings = self
            .inner
            .shader_meta
            .entry_point_output_bindings(self.inner.entry_index)
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

        self.inner.fragment_state.targets = color_outputs.targets().collect();

        FragmentStageBuilder {
            inner: FragmentStage {
                fragment_state: self.inner.fragment_state,
                shader_meta: self.inner.shader_meta,
                entry_index: self.inner.entry_index,
                _marker: Default::default(),
            },
            has_constants: self.has_constants,
        }
    }
}

impl<O> FragmentStageBuilder<O> {
    pub fn pipeline_constants<C: PipelineConstants>(
        mut self,
        pipeline_constants: &C,
    ) -> FragmentStageBuilder<O> {
        self.inner.fragment_state.constants =
            self.inner.shader_meta.build_constants(pipeline_constants);

        self.has_constants = true;

        self
    }
}

impl<O> FragmentStageBuilder<O>
where
    O: TypedColorLayout,
{
    pub fn finish(self) -> FragmentStage<O> {
        if !self.has_constants && self.inner.shader_meta.has_required_constants() {
            panic!("the shader declares pipeline constants without fallback values, but no pipeline constants were set");
        }

        self.inner
    }
}
