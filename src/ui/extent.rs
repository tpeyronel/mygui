#[derive(Debug, Clone, Copy, PartialEq)]

pub enum Extent {
    Extrinsic(ExtrinsicExtent),
    FitContent,
}

impl Extent {
    pub fn resolve_if_extrinsic(self, boundary_extent: f32, absolute_offset: f32, dp_factor: f32) -> Option<f32> {
        match self {
            Extent::FitContent => None,
            Extent::Extrinsic(extrinsic_extent) => {
                Some(extrinsic_extent.resolve(boundary_extent, absolute_offset, dp_factor))
            }
        }
    }

    pub fn fill_parent() -> Self {
        Self::Extrinsic(ExtrinsicExtent::fill_parent())
    }
}

impl Default for Extent {
    fn default() -> Self {
        Self::fill_parent()
    }
}

impl From<ExtrinsicExtent> for Extent {
    fn from(value: ExtrinsicExtent) -> Self {
        Self::Extrinsic(value)
    }
}

impl From<ExtrinsicExtent> for Option<Extent> {
    fn from(value: ExtrinsicExtent) -> Self {
        Some(Extent::Extrinsic(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtrinsicExtent {
    Parent(f32), // Relative
    Px(f32),     // Absolute
    Dp(f32),     // Absolute
}

impl ExtrinsicExtent {
    pub fn resolve(self, boundary_extent: f32, absolute_offset: f32, dp_factor: f32) -> f32 {
        match self {
            Self::Parent(r) => (r.max(0.0) * boundary_extent).round(),
            Self::Px(px) => px.round().max(0.0) + absolute_offset,
            Self::Dp(dp) => (dp_factor * dp).round().max(0.0) + absolute_offset,
        }
    }

    pub fn fill_parent() -> Self {
        Self::Parent(1.0)
    }
}

impl Default for ExtrinsicExtent {
    fn default() -> Self {
        Self::fill_parent()
    }
}

pub trait ExtentExt {
    fn px(&self) -> ExtrinsicExtent;
    fn dp(&self) -> ExtrinsicExtent;
    fn percent(&self) -> ExtrinsicExtent;
}

impl ExtentExt for f32 {
    fn px(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Px(*self as f32)
    }

    fn dp(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Dp(*self as f32)
    }

    fn percent(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for f64 {
    fn px(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Px(*self as f32)
    }

    fn dp(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Dp(*self as f32)
    }

    fn percent(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for i32 {
    fn px(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Px(*self as f32)
    }

    fn dp(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Dp(*self as f32)
    }

    fn percent(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for i64 {
    fn px(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Px(*self as f32)
    }

    fn dp(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Dp(*self as f32)
    }

    fn percent(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for u32 {
    fn px(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Px(*self as f32)
    }

    fn dp(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Dp(*self as f32)
    }

    fn percent(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Parent(*self as f32 * 0.01)
    }
}

impl ExtentExt for u64 {
    fn px(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Px(*self as f32)
    }

    fn dp(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Dp(*self as f32)
    }

    fn percent(&self) -> ExtrinsicExtent {
        ExtrinsicExtent::Parent(*self as f32 * 0.01)
    }
}
