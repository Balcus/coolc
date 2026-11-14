const NOGC_INIT: &str = "_NoGC_Init";
const NOGC_COLLECT: &str = "_NoGC_Collect";
const GENGC_INIT: &str = "_GenGC_Init";
const GENGC_COLLECT: &str = "_GenGC_Collect";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GcMode {
    NoGc,
    GenGc,
    GenGcTest,
}

impl GcMode {
    pub fn initializer(self) -> &'static str {
        match self {
            GcMode::NoGc => NOGC_INIT,
            _ => GENGC_INIT,
        }
    }

    pub fn collector(self) -> &'static str {
        match self {
            GcMode::NoGc => NOGC_COLLECT,
            _ => GENGC_COLLECT,
        }
    }

    pub fn test_flag(self) -> i32 {
        matches!(self, GcMode::GenGcTest) as i32
    }
}
