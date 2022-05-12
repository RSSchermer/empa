#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MipmapLevels {
    Complete,
    Partial(u8),
}

impl MipmapLevels {
    pub(crate) fn to_u32(&self, size: u32) -> u32 {
        let max_levels = (size as f64).log2() as u32 + 1;

        match *self {
            MipmapLevels::Complete => max_levels,
            MipmapLevels::Partial(levels) => {
                let levels = levels as u32;

                assert!(
                    levels <= max_levels,
                    "partial mipmap level count cannot the maximum level count({})",
                    max_levels
                );

                levels
            }
        }
    }
}
