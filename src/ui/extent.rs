#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extent {
    Parent(f32),
    FitContent,
    Px(f32),
    Dp(f32),
}

impl Extent {
    pub fn fill_parent() -> Self {
        Self::Parent(1.0)
    }
}

impl Default for Extent {
    fn default() -> Self {
        Self::fill_parent()
    }
}

pub trait ExtentExt {
    fn px(&self) -> Extent;
    fn dp(&self) -> Extent;
    fn percent(&self) -> Extent;
}

impl ExtentExt for f32 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }

    fn percent(&self) -> Extent {
        Extent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for f64 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }

    fn percent(&self) -> Extent {
        Extent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for i32 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }

    fn percent(&self) -> Extent {
        Extent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for i64 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }

    fn percent(&self) -> Extent {
        Extent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for u32 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }

    fn percent(&self) -> Extent {
        Extent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for u64 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }

    fn percent(&self) -> Extent {
        Extent::Parent(*self as f32 * 0.01)
    }
}
