#![allow(non_camel_case_types)]

use empa_reflect::EntryPointBindingType;

use crate::render_pipeline::VertexFormat;

pub(crate) fn vertex_format_is_compatible(
    format: VertexFormat,
    binding_type: EntryPointBindingType,
) -> bool {
    match binding_type {
        EntryPointBindingType::SignedInteger => match format {
            VertexFormat::sint32 => true,
            _ => false,
        },
        EntryPointBindingType::SignedIntegerVector2 => match format {
            VertexFormat::sint8x2 | VertexFormat::sint16x2 | VertexFormat::sint32x2 => true,
            _ => false,
        },
        EntryPointBindingType::SignedIntegerVector3 => match format {
            VertexFormat::sint32x3 => true,
            _ => false,
        },
        EntryPointBindingType::SignedIntegerVector4 => match format {
            VertexFormat::sint8x4 | VertexFormat::sint16x4 | VertexFormat::sint32x4 => true,
            _ => false,
        },
        EntryPointBindingType::UnsignedInteger => match format {
            VertexFormat::uint32 => true,
            _ => false,
        },
        EntryPointBindingType::UnsignedIntegerVector2 => match format {
            VertexFormat::uint8x2 | VertexFormat::uint16x2 | VertexFormat::uint32x2 => true,
            _ => false,
        },
        EntryPointBindingType::UnsignedIntegerVector3 => match format {
            VertexFormat::uint32x3 => true,
            _ => false,
        },
        EntryPointBindingType::UnsignedIntegerVector4 => match format {
            VertexFormat::uint8x4 | VertexFormat::uint16x4 | VertexFormat::uint32x4 => true,
            _ => false,
        },
        EntryPointBindingType::Float => match format {
            VertexFormat::float32 => true,
            _ => false,
        },
        EntryPointBindingType::FloatVector2 => match format {
            VertexFormat::unorm8x2
            | VertexFormat::snorm8x2
            | VertexFormat::unorm16x2
            | VertexFormat::snorm16x2
            | VertexFormat::float32x2 => true,
            _ => false,
        },
        EntryPointBindingType::FloatVector3 => match format {
            VertexFormat::float32x3 => true,
            _ => false,
        },
        EntryPointBindingType::FloatVector4 => match format {
            VertexFormat::unorm8x4
            | VertexFormat::snorm8x4
            | VertexFormat::unorm16x4
            | VertexFormat::snorm16x4
            | VertexFormat::float32x4 => true,
            _ => false,
        },
        EntryPointBindingType::HalfFloat => false,
        EntryPointBindingType::HalfFloatVector2 => match format {
            VertexFormat::float16x2 => true,
            _ => false,
        },
        EntryPointBindingType::HalfFloatVector3 => false,
        EntryPointBindingType::HalfFloatVector4 => match format {
            VertexFormat::float16x4 => true,
            _ => false,
        },
    }
}

mod vertex_attribute_format_seal {
    pub trait Seal {}
}

pub trait VertexAttributeFormat: vertex_attribute_format_seal::Seal {
    const FORMAT: VertexFormat;
}

pub struct uint8x2 {}

impl vertex_attribute_format_seal::Seal for uint8x2 {}
impl VertexAttributeFormat for uint8x2 {
    const FORMAT: VertexFormat = VertexFormat::uint8x2;
}

pub struct uint8x4 {}

impl vertex_attribute_format_seal::Seal for uint8x4 {}
impl VertexAttributeFormat for uint8x4 {
    const FORMAT: VertexFormat = VertexFormat::uint8x4;
}

pub struct sint8x2 {}

impl vertex_attribute_format_seal::Seal for sint8x2 {}
impl VertexAttributeFormat for sint8x2 {
    const FORMAT: VertexFormat = VertexFormat::sint8x2;
}

pub struct sint8x4 {}

impl vertex_attribute_format_seal::Seal for sint8x4 {}
impl VertexAttributeFormat for sint8x4 {
    const FORMAT: VertexFormat = VertexFormat::sint8x4;
}

pub struct unorm8x2 {}

impl vertex_attribute_format_seal::Seal for unorm8x2 {}
impl VertexAttributeFormat for unorm8x2 {
    const FORMAT: VertexFormat = VertexFormat::unorm8x2;
}

pub struct unorm8x4 {}

impl vertex_attribute_format_seal::Seal for unorm8x4 {}
impl VertexAttributeFormat for unorm8x4 {
    const FORMAT: VertexFormat = VertexFormat::unorm8x4;
}

pub struct snorm8x2 {}

impl vertex_attribute_format_seal::Seal for snorm8x2 {}
impl VertexAttributeFormat for snorm8x2 {
    const FORMAT: VertexFormat = VertexFormat::snorm8x2;
}

pub struct snorm8x4 {}

impl vertex_attribute_format_seal::Seal for snorm8x4 {}
impl VertexAttributeFormat for snorm8x4 {
    const FORMAT: VertexFormat = VertexFormat::snorm8x4;
}

pub struct uint16x2 {}

impl vertex_attribute_format_seal::Seal for uint16x2 {}
impl VertexAttributeFormat for uint16x2 {
    const FORMAT: VertexFormat = VertexFormat::uint16x2;
}

pub struct uint16x4 {}

impl vertex_attribute_format_seal::Seal for uint16x4 {}
impl VertexAttributeFormat for uint16x4 {
    const FORMAT: VertexFormat = VertexFormat::uint16x4;
}

pub struct sint16x2 {}

impl vertex_attribute_format_seal::Seal for sint16x2 {}
impl VertexAttributeFormat for sint16x2 {
    const FORMAT: VertexFormat = VertexFormat::sint16x2;
}

pub struct sint16x4 {}

impl vertex_attribute_format_seal::Seal for sint16x4 {}
impl VertexAttributeFormat for sint16x4 {
    const FORMAT: VertexFormat = VertexFormat::sint16x4;
}

pub struct unorm16x2 {}

impl vertex_attribute_format_seal::Seal for unorm16x2 {}
impl VertexAttributeFormat for unorm16x2 {
    const FORMAT: VertexFormat = VertexFormat::unorm16x2;
}

pub struct unorm16x4 {}

impl vertex_attribute_format_seal::Seal for unorm16x4 {}
impl VertexAttributeFormat for unorm16x4 {
    const FORMAT: VertexFormat = VertexFormat::unorm16x4;
}

pub struct snorm16x2 {}

impl vertex_attribute_format_seal::Seal for snorm16x2 {}
impl VertexAttributeFormat for snorm16x2 {
    const FORMAT: VertexFormat = VertexFormat::snorm16x2;
}

pub struct snorm16x4 {}

impl vertex_attribute_format_seal::Seal for snorm16x4 {}
impl VertexAttributeFormat for snorm16x4 {
    const FORMAT: VertexFormat = VertexFormat::snorm16x4;
}

pub struct float16x2 {}

impl vertex_attribute_format_seal::Seal for float16x2 {}
impl VertexAttributeFormat for float16x2 {
    const FORMAT: VertexFormat = VertexFormat::float16x2;
}

pub struct float16x4 {}

impl vertex_attribute_format_seal::Seal for float16x4 {}
impl VertexAttributeFormat for float16x4 {
    const FORMAT: VertexFormat = VertexFormat::float16x4;
}

pub struct float32 {}

impl vertex_attribute_format_seal::Seal for float32 {}
impl VertexAttributeFormat for float32 {
    const FORMAT: VertexFormat = VertexFormat::float32;
}

pub struct float32x2 {}

impl vertex_attribute_format_seal::Seal for float32x2 {}
impl VertexAttributeFormat for float32x2 {
    const FORMAT: VertexFormat = VertexFormat::float32x2;
}

pub struct float32x3 {}

impl vertex_attribute_format_seal::Seal for float32x3 {}
impl VertexAttributeFormat for float32x3 {
    const FORMAT: VertexFormat = VertexFormat::float32x3;
}

pub struct float32x4 {}

impl vertex_attribute_format_seal::Seal for float32x4 {}
impl VertexAttributeFormat for float32x4 {
    const FORMAT: VertexFormat = VertexFormat::float32x4;
}

pub struct uint32 {}

impl vertex_attribute_format_seal::Seal for uint32 {}
impl VertexAttributeFormat for uint32 {
    const FORMAT: VertexFormat = VertexFormat::uint32;
}

pub struct uint32x2 {}

impl vertex_attribute_format_seal::Seal for uint32x2 {}
impl VertexAttributeFormat for uint32x2 {
    const FORMAT: VertexFormat = VertexFormat::uint32x2;
}

pub struct uint32x3 {}

impl vertex_attribute_format_seal::Seal for uint32x3 {}
impl VertexAttributeFormat for uint32x3 {
    const FORMAT: VertexFormat = VertexFormat::uint32x3;
}

pub struct uint32x4 {}

impl vertex_attribute_format_seal::Seal for uint32x4 {}
impl VertexAttributeFormat for uint32x4 {
    const FORMAT: VertexFormat = VertexFormat::uint32x4;
}

pub struct sint32 {}

impl vertex_attribute_format_seal::Seal for sint32 {}
impl VertexAttributeFormat for sint32 {
    const FORMAT: VertexFormat = VertexFormat::sint32;
}

pub struct sint32x2 {}

impl vertex_attribute_format_seal::Seal for sint32x2 {}
impl VertexAttributeFormat for sint32x2 {
    const FORMAT: VertexFormat = VertexFormat::sint32x2;
}

pub struct sint32x3 {}

impl vertex_attribute_format_seal::Seal for sint32x3 {}
impl VertexAttributeFormat for sint32x3 {
    const FORMAT: VertexFormat = VertexFormat::sint32x3;
}

pub struct sint32x4 {}

impl vertex_attribute_format_seal::Seal for sint32x4 {}
impl VertexAttributeFormat for sint32x4 {
    const FORMAT: VertexFormat = VertexFormat::sint32x4;
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
