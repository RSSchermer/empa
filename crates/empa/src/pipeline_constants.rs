use std::fmt;

use crate::shader_module::StaticConstantType;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PipelineConstantIdentifier<'a> {
    Number(u32),
    Name(&'a str),
}

impl fmt::Display for PipelineConstantIdentifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineConstantIdentifier::Number(n) => n.fmt(f),
            PipelineConstantIdentifier::Name(n) => n.fmt(f),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PipelineConstantValue {
    Bool(bool),
    Float(f32),
    SignedInteger(i32),
    UnsignedInteger(u32),
}

impl PipelineConstantValue {
    pub(crate) fn constant_type(&self) -> StaticConstantType {
        match self {
            PipelineConstantValue::Bool(_) => StaticConstantType::Bool,
            PipelineConstantValue::Float(_) => StaticConstantType::Float,
            PipelineConstantValue::SignedInteger(_) => StaticConstantType::SignedInteger,
            PipelineConstantValue::UnsignedInteger(_) => StaticConstantType::UnsignedInteger,
        }
    }

    pub(crate) fn to_f64(&self) -> f64 {
        match *self {
            PipelineConstantValue::Bool(v) => v as u32 as f64,
            PipelineConstantValue::Float(v) => v as f64,
            PipelineConstantValue::SignedInteger(v) => v as f64,
            PipelineConstantValue::UnsignedInteger(v) => v as f64,
        }
    }
}

pub trait PipelineConstants {
    fn lookup(&self, identifier: PipelineConstantIdentifier) -> Option<PipelineConstantValue>;
}

mod pipeline_constant_seal {
    pub trait Seal {}
}

pub trait PipelineConstant: pipeline_constant_seal::Seal {
    fn into_value(self) -> PipelineConstantValue;
}

impl pipeline_constant_seal::Seal for bool {}
impl PipelineConstant for bool {
    fn into_value(self) -> PipelineConstantValue {
        PipelineConstantValue::Bool(self)
    }
}

impl pipeline_constant_seal::Seal for f32 {}
impl PipelineConstant for f32 {
    fn into_value(self) -> PipelineConstantValue {
        PipelineConstantValue::Float(self)
    }
}

impl pipeline_constant_seal::Seal for u32 {}
impl PipelineConstant for u32 {
    fn into_value(self) -> PipelineConstantValue {
        PipelineConstantValue::UnsignedInteger(self)
    }
}

impl pipeline_constant_seal::Seal for i32 {}
impl PipelineConstant for i32 {
    fn into_value(self) -> PipelineConstantValue {
        PipelineConstantValue::SignedInteger(self)
    }
}
