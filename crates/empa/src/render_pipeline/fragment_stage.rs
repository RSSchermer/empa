use std::collections::HashMap;
use std::marker;

use empa_smi::{IoBindingType, ShaderStage};
use flagset::{FlagSet, flags};

use crate::driver::ColorTargetState;
use crate::pipeline_constants::PipelineConstants;
use crate::render_target::TypedColorLayout;
use crate::shader_module::{EntryPointExt, ShaderModule, ShaderModuleData};
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
    pub(crate) shader_module_data: ShaderModuleData,
    pub(crate) entry_point_index: usize,
    pub(crate) pipeline_constants: HashMap<String, f64>,
    pub(crate) targets: Vec<ColorTargetState>,
}

impl FragmentState {
    pub(crate) fn entry_point(&self) -> EntryPointExt<'_> {
        self.shader_module_data
            .entry_point_ext(self.entry_point_index)
    }

    pub(crate) fn entry_point_name(&self) -> &str {
        self.shader_module_data.smi.entry_points[self.entry_point_index]
            .name
            .as_ref()
    }
}

pub struct FragmentStage<O> {
    pub(crate) state: FragmentState,
    _marker: marker::PhantomData<*const O>,
}

pub struct FragmentStageBuilder<O> {
    inner: FragmentStage<O>,
    has_constants: bool,
}

impl FragmentStageBuilder<()> {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let shader_module_data = shader_module.data.clone();

        let entry_point_index = shader_module_data
            .resolve_entry_point_index(entry_point)
            .expect("could not find entry point in shader module");
        let stage = shader_module_data.smi.entry_points[entry_point_index].stage;

        assert_eq!(
            stage,
            ShaderStage::Fragment,
            "entry point is not a fragment stage"
        );

        FragmentStageBuilder {
            inner: FragmentStage {
                state: FragmentState {
                    shader_module_data,
                    entry_point_index,
                    pipeline_constants: Default::default(),
                    targets: vec![],
                },
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

        for binding in self.inner.state.entry_point().output_bindings.as_ref() {
            let location = binding.location;
            let binding_type = binding.binding_type;

            if let Some(format) = layout.get(location as usize) {
                // TODO: it's not clear from the spec what it means for a format to be compatible
                // with an output. Assuming for now that compatibility is solely about the main
                // component type (float, half-float, uint, sint) and not the number of components
                // (as this is how it works in OpenGL); needs confirmation.
                if io_binding_type_is_float(&binding_type) && !format.is_float() {
                    panic!(
                        "shader expects a float format binding for location `{}`",
                        location
                    );
                }

                if io_binding_type_is_half_float(&binding_type) && !format.is_half_float() {
                    panic!(
                        "shader expects a half-float format binding for location `{}`",
                        location
                    );
                }

                if io_binding_type_is_signed_integer(&binding_type) && !format.is_signed_integer() {
                    panic!(
                        "shader expects a signed integer format binding for location `{}`",
                        location
                    );
                }

                if io_binding_type_is_unsigned_integer(&binding_type)
                    && !format.is_unsigned_integer()
                {
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

        self.inner.state.targets = color_outputs.targets().collect();

        FragmentStageBuilder {
            inner: FragmentStage {
                state: self.inner.state,
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
        self.inner.state.pipeline_constants = self
            .inner
            .state
            .entry_point()
            .build_constants(pipeline_constants);

        self.has_constants = true;

        self
    }
}

impl<O> FragmentStageBuilder<O>
where
    O: TypedColorLayout,
{
    pub fn finish(self) -> FragmentStage<O> {
        if !self.has_constants && self.inner.state.entry_point().has_required_constants() {
            panic!(
                "the shader declares pipeline constants without fallback values, but no pipeline \
                constants were set"
            );
        }

        self.inner
    }
}

fn io_binding_type_is_float(io_binding_type: &IoBindingType) -> bool {
    match io_binding_type {
        IoBindingType::Float
        | IoBindingType::FloatVector2
        | IoBindingType::FloatVector3
        | IoBindingType::FloatVector4 => true,
        _ => false,
    }
}

fn io_binding_type_is_half_float(io_binding_type: &IoBindingType) -> bool {
    match io_binding_type {
        IoBindingType::HalfFloat
        | IoBindingType::HalfFloatVector2
        | IoBindingType::HalfFloatVector3
        | IoBindingType::HalfFloatVector4 => true,
        _ => false,
    }
}

fn io_binding_type_is_signed_integer(io_binding_type: &IoBindingType) -> bool {
    match io_binding_type {
        IoBindingType::SignedInteger
        | IoBindingType::SignedIntegerVector2
        | IoBindingType::SignedIntegerVector3
        | IoBindingType::SignedIntegerVector4 => true,
        _ => false,
    }
}

fn io_binding_type_is_unsigned_integer(io_binding_type: &IoBindingType) -> bool {
    match io_binding_type {
        IoBindingType::UnsignedInteger
        | IoBindingType::UnsignedIntegerVector2
        | IoBindingType::UnsignedIntegerVector3
        | IoBindingType::UnsignedIntegerVector4 => true,
        _ => false,
    }
}
