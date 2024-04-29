mod type_flag_seal {
    pub trait Seal {}
}

pub trait TypeFlag: type_flag_seal::Seal {
    const IS_ENABLED: bool;
}

pub struct X {}

impl type_flag_seal::Seal for X {}

impl TypeFlag for X {
    const IS_ENABLED: bool = true;
}

pub struct O {}

impl type_flag_seal::Seal for O {}

impl TypeFlag for O {
    const IS_ENABLED: bool = false;
}
