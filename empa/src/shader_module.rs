use crate::render_pipeline::{
    PipelineConstantIdentifier, PipelineConstantValue, PipelineConstants,
};
use crate::resource_binding::{BindGroupLayoutEntry, BindingType};
use empa_reflect::ShaderSource as DynamicShaderSource;
use std::sync::Arc;
use web_sys::GpuShaderModule;

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticConstantDescriptor {
    pub identifier: PipelineConstantIdentifier<'static>,
    pub constant_type: StaticConstantType,
    pub required: bool,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StaticConstantType {
    Float,
    Bool,
    SignedInteger,
    UnsignedInteger,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StaticShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticResourceBinding {
    pub group: u32,
    pub binding: u32,
    pub binding_type: BindingType,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticEntryPointBinding {
    pub location: u32,
    pub binding_type: StaticEntryPointBindingType,
    pub interpolation: Option<StaticInterpolation>,
    pub sampling: Option<StaticSampling>,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StaticEntryPointBindingType {
    SignedInteger,
    SignedIntegerVector2,
    SignedIntegerVector3,
    SignedIntegerVector4,
    UnsignedInteger,
    UnsignedIntegerVector2,
    UnsignedIntegerVector3,
    UnsignedIntegerVector4,
    Float,
    FloatVector2,
    FloatVector3,
    FloatVector4,
    HalfFloat,
    HalfFloatVector2,
    HalfFloatVector3,
    HalfFloatVector4,
}

impl StaticEntryPointBindingType {
    pub(crate) fn is_float(&self) -> bool {
        match self {
            StaticEntryPointBindingType::Float
            | StaticEntryPointBindingType::FloatVector2
            | StaticEntryPointBindingType::FloatVector3
            | StaticEntryPointBindingType::FloatVector4 => true,
            _ => false,
        }
    }

    pub(crate) fn is_half_float(&self) -> bool {
        match self {
            StaticEntryPointBindingType::HalfFloat
            | StaticEntryPointBindingType::HalfFloatVector2
            | StaticEntryPointBindingType::HalfFloatVector3
            | StaticEntryPointBindingType::HalfFloatVector4 => true,
            _ => false,
        }
    }

    pub(crate) fn is_signed_integer(&self) -> bool {
        match self {
            StaticEntryPointBindingType::SignedInteger
            | StaticEntryPointBindingType::SignedIntegerVector2
            | StaticEntryPointBindingType::SignedIntegerVector3
            | StaticEntryPointBindingType::SignedIntegerVector4 => true,
            _ => false,
        }
    }

    pub(crate) fn is_unsigned_integer(&self) -> bool {
        match self {
            StaticEntryPointBindingType::UnsignedInteger
            | StaticEntryPointBindingType::UnsignedIntegerVector2
            | StaticEntryPointBindingType::UnsignedIntegerVector3
            | StaticEntryPointBindingType::UnsignedIntegerVector4 => true,
            _ => false,
        }
    }
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StaticInterpolation {
    Perspective,
    Linear,
    Flat,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StaticSampling {
    Center,
    Centroid,
    Sample,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticShaderSource {
    pub source: &'static str,
    pub resource_bindings: &'static [StaticResourceBinding],
    pub constants: &'static [StaticConstantDescriptor],
    pub entry_points: &'static [StaticEntryPoint],
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticEntryPoint {
    pub name: &'static str,
    pub stage: StaticShaderStage,
    pub input_bindings: &'static [StaticEntryPointBinding],
    pub output_bindings: &'static [StaticEntryPointBinding],
}

pub(crate) enum ShaderSourceInternal {
    Static(StaticShaderSource),
    Dynamic(DynamicShaderSource),
}

impl ShaderSourceInternal {
    pub(crate) fn source(&self) -> &str {
        match self {
            ShaderSourceInternal::Static(source) => source.source,
            ShaderSourceInternal::Dynamic(source) => source.raw_str(),
        }
    }

    pub(crate) fn resource_bindings(&self) -> &[StaticResourceBinding] {
        match self {
            ShaderSourceInternal::Static(source) => source.resource_bindings,
            ShaderSourceInternal::Dynamic(_) => todo!(),
        }
    }

    pub(crate) fn constants(&self) -> &[StaticConstantDescriptor] {
        match self {
            ShaderSourceInternal::Static(source) => source.constants,
            ShaderSourceInternal::Dynamic(_) => todo!(),
        }
    }

    pub(crate) fn entry_points(&self) -> &[StaticEntryPoint] {
        match self {
            ShaderSourceInternal::Static(source) => source.entry_points,
            ShaderSourceInternal::Dynamic(_) => todo!(),
        }
    }

    pub(crate) fn build_constants<C: PipelineConstants>(
        &self,
        pipeline_constants: &C,
    ) -> js_sys::Object {
        let shader_constants = self.constants();
        let record = js_sys::Object::new();

        for constant in shader_constants {
            if let Some(supplied_value) = pipeline_constants.lookup(constant.identifier) {
                if (supplied_value.constant_type() != constant.constant_type) {
                    panic!("supplied value for pipeline constant `{}` does not match the type expected by the shader", constant.identifier)
                }

                js_sys::Reflect::set(
                    record.as_ref(),
                    &constant.identifier.to_js_value(),
                    &supplied_value.to_js_value(),
                );
            } else {
                if constant.required {
                    panic!(
                        "could not find a value for the required constant `{}`",
                        constant.identifier
                    );
                }
            }
        }

        record
    }
}

pub struct ShaderSource {
    inner: ShaderSourceInternal,
}

impl ShaderSource {
    /// Internal function for `shader_source` macro.
    #[doc(hidden)]
    pub fn from_static(shader_source: StaticShaderSource) -> Self {
        ShaderSource {
            inner: ShaderSourceInternal::Static(shader_source),
        }
    }
}

pub struct ShaderModule {
    pub(crate) inner: GpuShaderModule,
    pub(crate) meta: Arc<ShaderSourceInternal>,
}
