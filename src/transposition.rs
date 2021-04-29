use crate::score::ScoreTy;

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum Flag {
    Exact,
    LowerBound,
    UpperBound,
}

impl Default for Flag {
    fn default() -> Self {
        Flag::LowerBound
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Default)]
pub struct CacheItem {
    pub depth: u8,
    pub flag: Flag,
    pub value: ScoreTy,
}
