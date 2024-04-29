use web_sys::GpuCompareFunction;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompareFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl CompareFunction {
    pub(crate) fn to_web_sys(&self) -> GpuCompareFunction {
        match self {
            CompareFunction::Never => GpuCompareFunction::Never,
            CompareFunction::Less => GpuCompareFunction::Less,
            CompareFunction::Equal => GpuCompareFunction::Equal,
            CompareFunction::LessEqual => GpuCompareFunction::LessEqual,
            CompareFunction::Greater => GpuCompareFunction::Greater,
            CompareFunction::NotEqual => GpuCompareFunction::NotEqual,
            CompareFunction::GreaterEqual => GpuCompareFunction::GreaterEqual,
            CompareFunction::Always => GpuCompareFunction::Always,
        }
    }
}
