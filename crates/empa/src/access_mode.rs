mod access_mode_seal {
    pub trait Seal {}
}

pub enum AccessModeKind {
    Read,
    Write,
    ReadWrite,
}

pub trait AccessMode: access_mode_seal::Seal {
    const KIND: AccessModeKind;
}

pub struct Read {}

impl access_mode_seal::Seal for Read {}
impl AccessMode for Read {
    const KIND: AccessModeKind = AccessModeKind::Read;
}

pub struct Write {}

impl access_mode_seal::Seal for Write {}
impl AccessMode for Write {
    const KIND: AccessModeKind = AccessModeKind::Write;
}

pub struct ReadWrite {}

impl access_mode_seal::Seal for ReadWrite {}
impl AccessMode for ReadWrite {
    const KIND: AccessModeKind = AccessModeKind::ReadWrite;
}
