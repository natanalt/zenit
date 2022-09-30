/// Type-safe wrapper for an angle specified in radians, so that you don't
/// make the mistake of passing in degrees.
///
/// Included is [`AngleExt`] which allows to use `.radians()` and `.degrees()`
/// on f32 values and literals.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Radians(pub f32);

impl Radians {
    #[inline]
    pub fn from_radians(r: f32) -> Self {
        Self(r)
    }

    #[inline]
    pub fn from_degrees(r: f32) -> Self {
        Self(r.to_radians())
    }

    #[inline]
    pub fn to_radians(self) -> f32 {
        self.0
    }

    #[inline]
    pub fn to_degrees(self) -> f32 {
        self.0.to_degrees()
    }
}

pub trait AngleExt {
    /// Interprets this float as a radian angle value
    fn radians(self) -> Radians;
    /// Interprets this float as a degree angle value
    fn degrees(self) -> Radians;
}

impl AngleExt for f32 {
    #[inline]
    fn radians(self) -> Radians {
        Radians::from_degrees(self)
    }

    #[inline]
    fn degrees(self) -> Radians {
        Radians::from_degrees(self)
    }
}
