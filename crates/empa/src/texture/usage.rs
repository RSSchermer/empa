use flagset::FlagSet;

use crate::driver::TextureUsage;
use crate::type_flag::{TypeFlag, O, X};

mod usage_flags_seal {
    use flagset::FlagSet;

    use crate::driver::TextureUsage;

    pub trait Seal {
        #[doc(hidden)]
        const FLAG_SET: FlagSet<TextureUsage>;
    }
}

pub trait UsageFlags: usage_flags_seal::Seal + Clone + Copy {}

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
    > Clone for Usages<RenderAttachment, StorageBinding, TextureBinding, CopyDst, CopySrc>
{
    fn clone(&self) -> Self {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        RenderAttachment: TypeFlag,
        StorageBinding: TypeFlag,
        TextureBinding: TypeFlag,
        CopyDst: TypeFlag,
        CopySrc: TypeFlag,
    > Copy for Usages<RenderAttachment, StorageBinding, TextureBinding, CopyDst, CopySrc>
{
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
    const FLAG_SET: FlagSet<TextureUsage> = {
        let mut bits = TextureUsage::None as u32;

        if CopySrc::IS_ENABLED {
            bits |= 0x0001;
        }

        if CopyDst::IS_ENABLED {
            bits |= 0x0002;
        }

        if TextureBinding::IS_ENABLED {
            bits |= 0x0004;
        }

        if StorageBinding::IS_ENABLED {
            bits |= 0x0008;
        }

        if RenderAttachment::IS_ENABLED {
            bits |= 0x0010;
        }

        unsafe { FlagSet::new_unchecked(bits) }
    };

    // TODO when const traits
    // const FLAG_SET: FlagSet<TextureUsage> = {
    //     let mut flags = FlagSet::from(TextureUsage::None);
    //
    //     if CopySrc::IS_ENABLED {
    //         flags |= TextureUsage::CopySrc;
    //     }
    //
    //     if CopyDst::IS_ENABLED {
    //         flags |= TextureUsage::CopyDst;
    //     }
    //
    //     if TextureBinding::IS_ENABLED {
    //         flags |= TextureUsage::TextureBinding;
    //     }
    //
    //     if StorageBinding::IS_ENABLED {
    //         flags |= TextureUsage::StorageBinding;
    //     }
    //
    //     if RenderAttachment::IS_ENABLED {
    //         flags |= TextureUsage::RenderAttachment;
    //     }
    //
    //     flags
    // };
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

impl Usages<O, O, O, O, O> {
    pub fn render_attachment() -> Usages<X, O, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn storage_binding() -> Usages<O, X, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn texture_binding() -> Usages<O, O, X, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn copy_dst() -> Usages<O, O, O, X, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn copy_src() -> Usages<O, O, O, O, X> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<U1: TypeFlag, U2: TypeFlag, U3: TypeFlag, U4: TypeFlag> Usages<O, U1, U2, U3, U4> {
    pub fn and_render_attachment(self) -> Usages<X, U1, U2, U3, U4> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<U0: TypeFlag, U2: TypeFlag, U3: TypeFlag, U4: TypeFlag> Usages<U0, O, U2, U3, U4> {
    pub fn and_storage_binding(self) -> Usages<U0, X, U2, U3, U4> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<U0: TypeFlag, U1: TypeFlag, U3: TypeFlag, U4: TypeFlag> Usages<U0, U1, O, U3, U4> {
    pub fn and_texture_binding(self) -> Usages<U0, U1, X, U3, U4> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<U0: TypeFlag, U1: TypeFlag, U2: TypeFlag, U4: TypeFlag> Usages<U0, U1, U2, O, U4> {
    pub fn and_copy_dst(self) -> Usages<U0, U1, U2, X, U4> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<U0: TypeFlag, U1: TypeFlag, U2: TypeFlag, U3: TypeFlag> Usages<U0, U1, U2, U3, O> {
    pub fn and_copy_src(self) -> Usages<U0, U1, U2, U3, X> {
        Usages {
            _marker: Default::default(),
        }
    }
}
