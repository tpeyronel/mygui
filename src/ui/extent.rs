#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extent {
    FillParent,
    FitContent,
    Px(f32),
    Dp(f32),
}

impl Default for Extent {
    fn default() -> Self {
        Self::FillParent
    }
}

pub trait ExtentExt {
    fn px(&self) -> Extent;
    fn dp(&self) -> Extent;
}

impl ExtentExt for f32 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }
}

impl ExtentExt for f64 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }
}

impl ExtentExt for i32 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }
}

impl ExtentExt for i64 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }
}

impl ExtentExt for u32 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }
}

impl ExtentExt for u64 {
    fn px(&self) -> Extent {
        Extent::Px(*self as f32)
    }

    fn dp(&self) -> Extent {
        Extent::Dp(*self as f32)
    }
}
