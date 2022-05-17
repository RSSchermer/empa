#![allow(non_camel_case_types)]

use crate::render_pipeline::StaticEntryPointBindingType;
use web_sys::GpuVertexFormat;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VertexAttributeFormatId {
    inner: GpuVertexFormat,
}

impl VertexAttributeFormatId {
    pub(crate) fn is_compatible(&self, binding_type: StaticEntryPointBindingType) -> bool {
        match binding_type {
            StaticEntryPointBindingType::SignedInteger => match self.inner {
                GpuVertexFormat::Sint32 => true,
                _ => false,
            },
            StaticEntryPointBindingType::SignedIntegerVector2 => match self.inner {
                GpuVertexFormat::Sint8x2
                | GpuVertexFormat::Sint16x2
                | GpuVertexFormat::Sint32x2 => true,
                _ => false,
            },
            StaticEntryPointBindingType::SignedIntegerVector3 => match self.inner {
                GpuVertexFormat::Sint32x3 => true,
                _ => false,
            },
            StaticEntryPointBindingType::SignedIntegerVector4 => match self.inner {
                GpuVertexFormat::Sint8x4
                | GpuVertexFormat::Sint16x4
                | GpuVertexFormat::Sint32x4 => true,
                _ => false,
            },
            StaticEntryPointBindingType::UnsignedInteger => match self.inner {
                GpuVertexFormat::Uint32 => true,
                _ => false,
            },
            StaticEntryPointBindingType::UnsignedIntegerVector2 => match self.inner {
                GpuVertexFormat::Uint8x2
                | GpuVertexFormat::Uint16x2
                | GpuVertexFormat::Uint32x2 => true,
                _ => false,
            },
            StaticEntryPointBindingType::UnsignedIntegerVector3 => match self.inner {
                GpuVertexFormat::Uint32x3 => true,
                _ => false,
            },
            StaticEntryPointBindingType::UnsignedIntegerVector4 => match self.inner {
                GpuVertexFormat::Uint8x4
                | GpuVertexFormat::Uint16x4
                | GpuVertexFormat::Uint32x4 => true,
                _ => false,
            },
            StaticEntryPointBindingType::Float => match self.inner {
                GpuVertexFormat::Float32 => true,
                _ => false,
            },
            StaticEntryPointBindingType::FloatVector2 => match self.inner {
                GpuVertexFormat::Unorm8x2
                | GpuVertexFormat::Snorm8x2
                | GpuVertexFormat::Unorm16x2
                | GpuVertexFormat::Snorm16x2
                | GpuVertexFormat::Float32x2 => true,
                _ => false,
            },
            StaticEntryPointBindingType::FloatVector3 => match self.inner {
                GpuVertexFormat::Float32x3 => true,
                _ => false,
            },
            StaticEntryPointBindingType::FloatVector4 => match self.inner {
                GpuVertexFormat::Unorm8x4
                | GpuVertexFormat::Snorm8x4
                | GpuVertexFormat::Unorm16x4
                | GpuVertexFormat::Snorm16x4
                | GpuVertexFormat::Float32x4 => true,
                _ => false,
            },
            StaticEntryPointBindingType::HalfFloat => false,
            StaticEntryPointBindingType::HalfFloatVector2 => match self.inner {
                GpuVertexFormat::Float16x2 => true,
                _ => false,
            },
            StaticEntryPointBindingType::HalfFloatVector3 => false,
            StaticEntryPointBindingType::HalfFloatVector4 => match self.inner {
                GpuVertexFormat::Float16x4 => true,
                _ => false,
            },
        }
    }
}

mod vertex_attribute_format_seal {
    pub trait Seal {}
}

pub trait VertexAttributeFormat: vertex_attribute_format_seal::Seal {
    const FORMAT_ID: VertexAttributeFormatId;
}

pub struct uint8x2 {}

impl vertex_attribute_format_seal::Seal for uint8x2 {}
impl VertexAttributeFormat for uint8x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Uint8x2,
    };
}

pub struct uint8x4 {}

impl vertex_attribute_format_seal::Seal for uint8x4 {}
impl VertexAttributeFormat for uint8x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Uint8x4,
    };
}

pub struct sint8x2 {}

impl vertex_attribute_format_seal::Seal for sint8x2 {}
impl VertexAttributeFormat for sint8x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Sint8x2,
    };
}

pub struct sint8x4 {}

impl vertex_attribute_format_seal::Seal for sint8x4 {}
impl VertexAttributeFormat for sint8x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Sint8x4,
    };
}

pub struct unorm8x2 {}

impl vertex_attribute_format_seal::Seal for unorm8x2 {}
impl VertexAttributeFormat for unorm8x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Unorm8x2,
    };
}

pub struct unorm8x4 {}

impl vertex_attribute_format_seal::Seal for unorm8x4 {}
impl VertexAttributeFormat for unorm8x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Unorm8x4,
    };
}

pub struct snorm8x2 {}

impl vertex_attribute_format_seal::Seal for snorm8x2 {}
impl VertexAttributeFormat for snorm8x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Snorm8x2,
    };
}

pub struct snorm8x4 {}

impl vertex_attribute_format_seal::Seal for snorm8x4 {}
impl VertexAttributeFormat for snorm8x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Snorm8x4,
    };
}

pub struct uint16x2 {}

impl vertex_attribute_format_seal::Seal for uint16x2 {}
impl VertexAttributeFormat for uint16x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Uint16x2,
    };
}

pub struct uint16x4 {}

impl vertex_attribute_format_seal::Seal for uint16x4 {}
impl VertexAttributeFormat for uint16x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Uint16x4,
    };
}

pub struct sint16x2 {}

impl vertex_attribute_format_seal::Seal for sint16x2 {}
impl VertexAttributeFormat for sint16x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Sint16x2,
    };
}

pub struct sint16x4 {}

impl vertex_attribute_format_seal::Seal for sint16x4 {}
impl VertexAttributeFormat for sint16x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Sint16x4,
    };
}

pub struct unorm16x2 {}

impl vertex_attribute_format_seal::Seal for unorm16x2 {}
impl VertexAttributeFormat for unorm16x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Unorm16x2,
    };
}

pub struct unorm16x4 {}

impl vertex_attribute_format_seal::Seal for unorm16x4 {}
impl VertexAttributeFormat for unorm16x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Unorm16x4,
    };
}

pub struct snorm16x2 {}

impl vertex_attribute_format_seal::Seal for snorm16x2 {}
impl VertexAttributeFormat for snorm16x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Snorm16x2,
    };
}

pub struct snorm16x4 {}

impl vertex_attribute_format_seal::Seal for snorm16x4 {}
impl VertexAttributeFormat for snorm16x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Snorm16x4,
    };
}

pub struct float16x2 {}

impl vertex_attribute_format_seal::Seal for float16x2 {}
impl VertexAttributeFormat for float16x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Float16x2,
    };
}

pub struct float16x4 {}

impl vertex_attribute_format_seal::Seal for float16x4 {}
impl VertexAttributeFormat for float16x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Float16x4,
    };
}

pub struct float32 {}

impl vertex_attribute_format_seal::Seal for float32 {}
impl VertexAttributeFormat for float32 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Float32,
    };
}

pub struct float32x2 {}

impl vertex_attribute_format_seal::Seal for float32x2 {}
impl VertexAttributeFormat for float32x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Float32x2,
    };
}

pub struct float32x3 {}

impl vertex_attribute_format_seal::Seal for float32x3 {}
impl VertexAttributeFormat for float32x3 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Float32x3,
    };
}

pub struct float32x4 {}

impl vertex_attribute_format_seal::Seal for float32x4 {}
impl VertexAttributeFormat for float32x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Float32x4,
    };
}

pub struct uint32 {}

impl vertex_attribute_format_seal::Seal for uint32 {}
impl VertexAttributeFormat for uint32 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Uint32,
    };
}

pub struct uint32x2 {}

impl vertex_attribute_format_seal::Seal for uint32x2 {}
impl VertexAttributeFormat for uint32x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Uint32x2,
    };
}

pub struct uint32x3 {}

impl vertex_attribute_format_seal::Seal for uint32x3 {}
impl VertexAttributeFormat for uint32x3 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Uint32x3,
    };
}

pub struct uint32x4 {}

impl vertex_attribute_format_seal::Seal for uint32x4 {}
impl VertexAttributeFormat for uint32x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Uint32x4,
    };
}

pub struct sint32 {}

impl vertex_attribute_format_seal::Seal for sint32 {}
impl VertexAttributeFormat for sint32 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Sint32,
    };
}

pub struct sint32x2 {}

impl vertex_attribute_format_seal::Seal for sint32x2 {}
impl VertexAttributeFormat for sint32x2 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Sint32x2,
    };
}

pub struct sint32x3 {}

impl vertex_attribute_format_seal::Seal for sint32x3 {}
impl VertexAttributeFormat for sint32x3 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Sint32x3,
    };
}

pub struct sint32x4 {}

impl vertex_attribute_format_seal::Seal for sint32x4 {}
impl VertexAttributeFormat for sint32x4 {
    const FORMAT_ID: VertexAttributeFormatId = VertexAttributeFormatId {
        inner: GpuVertexFormat::Sint32x4,
    };
}

pub unsafe trait VertexAttributeFormatCompatible<F>
where
    F: VertexAttributeFormat,
{
}

unsafe impl VertexAttributeFormatCompatible<uint8x2> for [u8; 2] {}
unsafe impl VertexAttributeFormatCompatible<uint8x4> for [u8; 4] {}
unsafe impl VertexAttributeFormatCompatible<sint8x2> for [i8; 2] {}
unsafe impl VertexAttributeFormatCompatible<sint8x4> for [i8; 4] {}
unsafe impl VertexAttributeFormatCompatible<unorm8x2> for [u8; 2] {}
unsafe impl VertexAttributeFormatCompatible<unorm8x4> for [u8; 4] {}
unsafe impl VertexAttributeFormatCompatible<snorm8x2> for [i8; 2] {}
unsafe impl VertexAttributeFormatCompatible<snorm8x4> for [i8; 4] {}
unsafe impl VertexAttributeFormatCompatible<uint16x2> for [u16; 2] {}
unsafe impl VertexAttributeFormatCompatible<uint16x4> for [u16; 4] {}
unsafe impl VertexAttributeFormatCompatible<sint16x2> for [i16; 2] {}
unsafe impl VertexAttributeFormatCompatible<sint16x4> for [i16; 4] {}
unsafe impl VertexAttributeFormatCompatible<unorm16x2> for [u16; 2] {}
unsafe impl VertexAttributeFormatCompatible<unorm16x4> for [u16; 4] {}
unsafe impl VertexAttributeFormatCompatible<snorm16x2> for [i16; 2] {}
unsafe impl VertexAttributeFormatCompatible<snorm16x4> for [i16; 4] {}
unsafe impl VertexAttributeFormatCompatible<float32> for f32 {}
unsafe impl VertexAttributeFormatCompatible<float32x2> for [f32; 2] {}
unsafe impl VertexAttributeFormatCompatible<float32x3> for [f32; 3] {}
unsafe impl VertexAttributeFormatCompatible<float32x4> for [f32; 4] {}
unsafe impl VertexAttributeFormatCompatible<uint32> for u32 {}
unsafe impl VertexAttributeFormatCompatible<uint32x2> for [u32; 2] {}
unsafe impl VertexAttributeFormatCompatible<uint32x3> for [u32; 3] {}
unsafe impl VertexAttributeFormatCompatible<uint32x4> for [u32; 4] {}
unsafe impl VertexAttributeFormatCompatible<sint32> for i32 {}
unsafe impl VertexAttributeFormatCompatible<sint32x2> for [i32; 2] {}
unsafe impl VertexAttributeFormatCompatible<sint32x3> for [i32; 3] {}
unsafe impl VertexAttributeFormatCompatible<sint32x4> for [i32; 4] {}
