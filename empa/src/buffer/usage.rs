use crate::type_flag::{TypeFlag, O, X};

mod usage_flags_seal {
    pub trait Seal {
        #[doc(hidden)]
        const BITS: u32;
    }
}

pub trait UsageFlags: usage_flags_seal::Seal {}

pub trait ValidUsageFlags: UsageFlags {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
    > ValidUsageFlags for Usages<U0, U1, U2, U3, U4, U5, U6, U7, O, O>
{
}
impl ValidUsageFlags for Usages<O, O, O, O, O, O, X, O, O, X> {}
impl ValidUsageFlags for Usages<O, O, O, O, O, O, O, X, X, O> {}

pub struct Usages<
    QueryResolve: TypeFlag,
    Indirect: TypeFlag,
    StorageBinding: TypeFlag,
    UniformBinding: TypeFlag,
    Vertex: TypeFlag,
    Index: TypeFlag,
    CopyDst: TypeFlag,
    CopySrc: TypeFlag,
    MapWrite: TypeFlag,
    MapRead: TypeFlag,
> {
    _marker: std::marker::PhantomData<(
        QueryResolve,
        Indirect,
        StorageBinding,
        UniformBinding,
        Vertex,
        Index,
        CopyDst,
        CopySrc,
        MapWrite,
        MapRead,
    )>,
}

impl<
        QueryResolve: TypeFlag,
        Indirect: TypeFlag,
        StorageBinding: TypeFlag,
        UniformBinding: TypeFlag,
        Vertex: TypeFlag,
        Index: TypeFlag,
        CopyDst: TypeFlag,
        CopySrc: TypeFlag,
        MapWrite: TypeFlag,
        MapRead: TypeFlag,
    > usage_flags_seal::Seal
    for Usages<
        QueryResolve,
        Indirect,
        StorageBinding,
        UniformBinding,
        Vertex,
        Index,
        CopyDst,
        CopySrc,
        MapWrite,
        MapRead,
    >
{
    const BITS: u32 = {
        let mut flags = 0u32;

        if MapRead::IS_ENABLED {
            flags |= 1 << 0;
        }

        if MapWrite::IS_ENABLED {
            flags |= 1 << 1;
        }

        if CopySrc::IS_ENABLED {
            flags |= 1 << 2;
        }

        if CopyDst::IS_ENABLED {
            flags |= 1 << 3;
        }

        if Index::IS_ENABLED {
            flags |= 1 << 4;
        }

        if Vertex::IS_ENABLED {
            flags |= 1 << 5;
        }

        if UniformBinding::IS_ENABLED {
            flags |= 1 << 6;
        }

        if StorageBinding::IS_ENABLED {
            flags |= 1 << 7;
        }

        if Indirect::IS_ENABLED {
            flags |= 1 << 8;
        }

        if QueryResolve::IS_ENABLED {
            flags |= 1 << 9;
        }

        flags
    };
}

impl<
        QueryResolve: TypeFlag,
        Indirect: TypeFlag,
        StorageBinding: TypeFlag,
        UniformBinding: TypeFlag,
        Vertex: TypeFlag,
        Index: TypeFlag,
        CopyDst: TypeFlag,
        CopySrc: TypeFlag,
        MapWrite: TypeFlag,
        MapRead: TypeFlag,
    > UsageFlags
    for Usages<
        QueryResolve,
        Indirect,
        StorageBinding,
        UniformBinding,
        Vertex,
        Index,
        CopyDst,
        CopySrc,
        MapWrite,
        MapRead,
    >
{
}

mod map_read_seal {
    pub trait Seal {}
}

pub trait MapRead: map_read_seal::Seal {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
    > map_read_seal::Seal for Usages<U0, U1, U2, U3, U4, U5, U6, U7, U8, X>
{
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
    > MapRead for Usages<U0, U1, U2, U3, U4, U5, U6, U7, U8, X>
{
}

mod map_write_seal {
    pub trait Seal {}
}

pub trait MapWrite: map_write_seal::Seal {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U9: TypeFlag,
    > map_write_seal::Seal for Usages<U0, U1, U2, U3, U4, U5, U6, U7, X, U9>
{
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U9: TypeFlag,
    > MapWrite for Usages<U0, U1, U2, U3, U4, U5, U6, U7, X, U9>
{
}

mod copy_src_seal {
    pub trait Seal {}
}

pub trait CopySrc: copy_src_seal::Seal {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > copy_src_seal::Seal for Usages<U0, U1, U2, U3, U4, U5, U6, X, U8, U9>
{
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > CopySrc for Usages<U0, U1, U2, U3, U4, U5, U6, X, U8, U9>
{
}

mod copy_dst_seal {
    pub trait Seal {}
}

pub trait CopyDst: copy_dst_seal::Seal {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > copy_dst_seal::Seal for Usages<U0, U1, U2, U3, U4, U5, X, U7, U8, U9>
{
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > CopyDst for Usages<U0, U1, U2, U3, U4, U5, X, U7, U8, U9>
{
}

mod index_seal {
    pub trait Seal {}
}

pub trait Index: index_seal::Seal {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > index_seal::Seal for Usages<U0, U1, U2, U3, U4, X, U6, U7, U8, U9>
{
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Index for Usages<U0, U1, U2, U3, U4, X, U6, U7, U8, U9>
{
}

mod vertex_seal {
    pub trait Seal {}
}

pub trait Vertex: vertex_seal::Seal {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > vertex_seal::Seal for Usages<U0, U1, U2, U3, X, U5, U6, U7, U8, U9>
{
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Vertex for Usages<U0, U1, U2, U3, X, U5, U6, U7, U8, U9>
{
}

mod uniform_binding_seal {
    pub trait Seal {}
}

pub trait UniformBinding: uniform_binding_seal::Seal {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > uniform_binding_seal::Seal for Usages<U0, U1, U2, X, U4, U5, U6, U7, U8, U9>
{
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > UniformBinding for Usages<U0, U1, U2, X, U4, U5, U6, U7, U8, U9>
{
}

mod storage_binding_seal {
    pub trait Seal {}
}

pub trait StorageBinding: storage_binding_seal::Seal {}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > storage_binding_seal::Seal for Usages<U0, U1, X, U3, U4, U5, U6, U7, U8, U9>
{
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > StorageBinding for Usages<U0, U1, X, U3, U4, U5, U6, U7, U8, U9>
{
}

mod indirect_seal {
    pub trait Seal {}
}

pub trait Indirect: indirect_seal::Seal {}

impl<
        U0: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > indirect_seal::Seal for Usages<U0, X, U2, U3, U4, U5, U6, U7, U8, U9>
{
}

impl<
        U0: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Indirect for Usages<U0, X, U2, U3, U4, U5, U6, U7, U8, U9>
{
}

mod query_resolve_seal {
    pub trait Seal {}
}

pub trait QueryResolve: query_resolve_seal::Seal {}

impl<
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > query_resolve_seal::Seal for Usages<X, U1, U2, U3, U4, U5, U6, U7, U8, U9>
{
}

impl<
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > QueryResolve for Usages<X, U1, U2, U3, U4, U5, U6, U7, U8, U9>
{
}

impl Usages<O, O, O, O, O, O, O, O, O, O> {
    pub fn query_resolve() -> Usages<X, O, O, O, O, O, O, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn indirect() -> Usages<O, X, O, O, O, O, O, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn storage_binding() -> Usages<O, O, X, O, O, O, O, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn uniform_binding() -> Usages<O, O, O, X, O, O, O, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn vertex() -> Usages<O, O, O, O, X, O, O, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn index() -> Usages<O, O, O, O, O, X, O, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn copy_dst() -> Usages<O, O, O, O, O, O, X, O, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn copy_src() -> Usages<O, O, O, O, O, O, O, X, O, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn map_write() -> Usages<O, O, O, O, O, O, O, O, X, O> {
        Usages {
            _marker: Default::default(),
        }
    }

    pub fn map_read() -> Usages<O, O, O, O, O, O, O, O, O, X> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, U1, U2, U3, U4, U5, U6, U7, U8, U9>
{
    pub fn and_render_attachment(self) -> Usages<U0, U1, U2, U3, U4, U5, U6, U7, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<O, U1, U2, U3, U4, U5, U6, U7, U8, U9>
{
    pub fn and_query_resolve(self) -> Usages<X, U1, U2, U3, U4, U5, U6, U7, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, O, U2, U3, U4, U5, U6, U7, U8, U9>
{
    pub fn and_indirect(self) -> Usages<U0, X, U2, U3, U4, U5, U6, U7, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, U1, O, U3, U4, U5, U6, U7, U8, U9>
{
    pub fn and_storage_binding(self) -> Usages<U0, U1, X, U3, U4, U5, U6, U7, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, U1, U2, O, U4, U5, U6, U7, U8, U9>
{
    pub fn and_uniform_binding(self) -> Usages<U0, U1, U2, X, U4, U5, U6, U7, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, U1, U2, U3, O, U5, U6, U7, U8, U9>
{
    pub fn and_vertex(self) -> Usages<U0, U1, U2, U3, X, U5, U6, U7, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, U1, U2, U3, U4, O, U6, U7, U8, U9>
{
    pub fn and_index(self) -> Usages<U0, U1, U2, U3, U4, X, U6, U7, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, U1, U2, U3, U4, U5, O, U7, U8, U9>
{
    pub fn and_copy_dst(self) -> Usages<U0, U1, U2, U3, U4, U5, X, U7, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U8: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, U1, U2, U3, U4, U5, U6, O, U8, U9>
{
    pub fn and_copy_src(self) -> Usages<U0, U1, U2, U3, U4, U5, U6, X, U8, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U9: TypeFlag,
    > Usages<U0, U1, U2, U3, U4, U5, U6, U7, O, U9>
{
    pub fn and_map_write(self) -> Usages<U0, U1, U2, U3, U4, U5, U6, U7, X, U9> {
        Usages {
            _marker: Default::default(),
        }
    }
}

impl<
        U0: TypeFlag,
        U1: TypeFlag,
        U2: TypeFlag,
        U3: TypeFlag,
        U4: TypeFlag,
        U5: TypeFlag,
        U6: TypeFlag,
        U7: TypeFlag,
        U8: TypeFlag,
    > Usages<U0, U1, U2, U3, U4, U5, U6, U7, U8, O>
{
    pub fn and_map_read(self) -> Usages<U0, U1, U2, U3, U4, U5, U6, U7, U8, X> {
        Usages {
            _marker: Default::default(),
        }
    }
}
