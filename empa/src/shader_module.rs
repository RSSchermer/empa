use crate::render_pipeline::{
    PipelineConstantIdentifier, PipelineConstantValue, PipelineConstants,
};
use crate::resource_binding::{BindGroupLayoutEntry, BindingType};
use std::sync::Arc;
use web_sys::GpuShaderModule;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PipelineConstantDescriptor {
    pub identifier: PipelineConstantIdentifier,
    pub constant_type: PipelineConstantType,
    pub required: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PipelineConstantType {
    Float,
    Bool,
    SignedInteger,
    UnsignedInteger,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ShaderResourceBinding {
    pub group: u32,
    pub binding: u32,
    pub binding_type: BindingType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ShaderEntryPointBinding {
    pub location: u32,
    pub binding_type: ShaderEntryPointBindingType,
    pub interpolation: Option<ShaderEntryPointBindingInterpolation>,
    pub sampling: Option<ShaderEntryPointBindingSampling>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShaderEntryPointBindingType {
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

impl ShaderEntryPointBindingType {
    pub(crate) fn is_float(&self) -> bool {
        match self {
            ShaderEntryPointBindingType::Float
            | ShaderEntryPointBindingType::FloatVector2
            | ShaderEntryPointBindingType::FloatVector3
            | ShaderEntryPointBindingType::FloatVector4 => true,
            _ => false,
        }
    }

    pub(crate) fn is_half_float(&self) -> bool {
        match self {
            ShaderEntryPointBindingType::HalfFloat
            | ShaderEntryPointBindingType::HalfFloatVector2
            | ShaderEntryPointBindingType::HalfFloatVector3
            | ShaderEntryPointBindingType::HalfFloatVector4 => true,
            _ => false,
        }
    }

    pub(crate) fn is_signed_integer(&self) -> bool {
        match self {
            ShaderEntryPointBindingType::SignedInteger
            | ShaderEntryPointBindingType::SignedIntegerVector2
            | ShaderEntryPointBindingType::SignedIntegerVector3
            | ShaderEntryPointBindingType::SignedIntegerVector4 => true,
            _ => false,
        }
    }

    pub(crate) fn is_unsigned_integer(&self) -> bool {
        match self {
            ShaderEntryPointBindingType::UnsignedInteger
            | ShaderEntryPointBindingType::UnsignedIntegerVector2
            | ShaderEntryPointBindingType::UnsignedIntegerVector3
            | ShaderEntryPointBindingType::UnsignedIntegerVector4 => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShaderEntryPointBindingInterpolation {
    Perspective,
    Linear,
    Flat,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShaderEntryPointBindingSampling {
    Center,
    Centroid,
    Sample,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PreprocessedShaderSource {
    pub source: &'static str,
    pub resource_bindings: &'static [ShaderResourceBinding],
    pub constants: &'static [PipelineConstantDescriptor],
    pub entry_points: &'static [ShaderEntryPoint],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ShaderEntryPoint {
    pub name: &'static str,
    pub stage: ShaderStage,
    pub input_bindings: &'static [ShaderEntryPointBinding],
    pub output_bindings: &'static [ShaderEntryPointBinding],
}

pub(crate) enum ShaderMeta {
    Preprocessed(PreprocessedShaderSource),
}

impl ShaderMeta {
    pub(crate) fn source(&self) -> &str {
        match self {
            ShaderMeta::Preprocessed(source) => source.source,
        }
    }

    pub(crate) fn resource_bindings(&self) -> &[ShaderResourceBinding] {
        match self {
            ShaderMeta::Preprocessed(source) => source.resource_bindings,
        }
    }

    pub(crate) fn constants(&self) -> &[PipelineConstantDescriptor] {
        match self {
            ShaderMeta::Preprocessed(source) => source.constants,
        }
    }

    pub(crate) fn entry_points(&self) -> &[ShaderEntryPoint] {
        match self {
            ShaderMeta::Preprocessed(source) => source.entry_points,
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

pub struct ShaderModule {
    pub(crate) inner: GpuShaderModule,
    pub(crate) meta: Arc<ShaderMeta>,
}
