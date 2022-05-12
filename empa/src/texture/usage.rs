use crate::type_flag::{TypeFlag, X};

mod usage_flags_seal {
    pub trait Seal {
        #[doc(hidden)]
        const BITS: u32;
    }
}

pub trait UsageFlags: usage_flags_seal::Seal {}

pub struct Usages<
    RenderAttachment: TypeFlag,
    StorageBinding: TypeFlag,
    TextureBinding: TypeFlag,
    CopyDst: TypeFlag,
    CopySrc: TypeFlag,
> {
    _marker: std::marker::PhantomData<(
        RenderAttachment,
        StorageBinding,
        TextureBinding,
        CopyDst,
        CopySrc,
    )>,
}

impl<
        RenderAttachment: TypeFlag,
        StorageBinding: TypeFlag,
        TextureBinding: TypeFlag,
        CopyDst: TypeFlag,
        CopySrc: TypeFlag,
    > usage_flags_seal::Seal
    for Usages<RenderAttachment, StorageBinding, TextureBinding, CopyDst, CopySrc>
{
    const BITS: u32 = {
        let mut flags = 0u32;

        if CopySrc::IS_ENABLED {
            flags |= 1 << 0;
        }

        if CopyDst::IS_ENABLED {
            flags |= 1 << 1;
        }

        if TextureBinding::IS_ENABLED {
            flags |= 1 << 2;
        }

        if StorageBinding::IS_ENABLED {
            flags |= 1 << 3;
        }

        if RenderAttachment::IS_ENABLED {
            flags |= 1 << 4;
        }

        flags
    };
}

impl<
        RenderAttachment: TypeFlag,
        StorageBinding: TypeFlag,
        TextureBinding: TypeFlag,
        CopyDst: TypeFlag,
        CopySrc: TypeFlag,
    > UsageFlags for Usages<RenderAttachment, StorageBinding, TextureBinding, CopyDst, CopySrc>
{
}

mod copy_src_seal {
    pub trait Seal {}
}

pub trait CopySrc: copy_src_seal::Seal {}

impl<U0: TypeFlag, U1: TypeFlag, U2: TypeFlag, U3: TypeFlag> copy_src_seal::Seal
    for Usages<U0, U1, U2, U3, X>
{
}

impl<U0: TypeFlag, U1: TypeFlag, U2: TypeFlag, U3: TypeFlag> CopySrc for Usages<U0, U1, U2, U3, X> {}

mod copy_dst_seal {
    pub trait Seal {}
}

pub trait CopyDst: copy_dst_seal::Seal {}

impl<U0: TypeFlag, U1: TypeFlag, U2: TypeFlag, U4: TypeFlag> copy_dst_seal::Seal
    for Usages<U0, U1, U2, X, U4>
{
}

impl<U0: TypeFlag, U1: TypeFlag, U2: TypeFlag, U4: TypeFlag> CopyDst for Usages<U0, U1, U2, X, U4> {}

mod texture_binding_seal {
    pub trait Seal {}
}

pub trait TextureBinding: texture_binding_seal::Seal {}

impl<U0: TypeFlag, U1: TypeFlag, U3: TypeFlag, U4: TypeFlag> texture_binding_seal::Seal
    for Usages<U0, U1, X, U3, U4>
{
}

impl<U0: TypeFlag, U1: TypeFlag, U3: TypeFlag, U4: TypeFlag> TextureBinding
    for Usages<U0, U1, X, U3, U4>
{
}

mod storage_binding_seal {
    pub trait Seal {}
}

pub trait StorageBinding: storage_binding_seal::Seal {}

impl<U0: TypeFlag, U2: TypeFlag, U3: TypeFlag, U4: TypeFlag> storage_binding_seal::Seal
    for Usages<U0, X, U2, U3, U4>
{
}

impl<U0: TypeFlag, U2: TypeFlag, U3: TypeFlag, U4: TypeFlag> StorageBinding
    for Usages<U0, X, U2, U3, U4>
{
}

mod render_attachment_seal {
    pub trait Seal {}
}

pub trait RenderAttachment: render_attachment_seal::Seal {}

impl<U1: TypeFlag, U2: TypeFlag, U3: TypeFlag, U4: TypeFlag> render_attachment_seal::Seal
    for Usages<X, U1, U2, U3, U4>
{
}

impl<U1: TypeFlag, U2: TypeFlag, U3: TypeFlag, U4: TypeFlag> RenderAttachment
    for Usages<X, U1, U2, U3, U4>
{
}
