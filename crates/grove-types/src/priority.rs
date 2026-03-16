use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BeadPriority {
    P0,
    P1,
    P2,
    P3,
    P4,
}

impl BeadPriority {
    #[must_use]
    pub fn base_score(self) -> i32 {
        match self {
            Self::P0 => 100,
            Self::P1 => 70,
            Self::P2 => 40,
            Self::P3 => 20,
            Self::P4 => 5,
        }
    }
}
