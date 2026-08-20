use std::hash::Hash;
use std::ops::Add;
use std::ops::Deref;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

pub trait Number:
    Copy
    + Eq
    + PartialOrd
    + Ord
    + Hash
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Default
    + Deref<Target = f64>
{
}

impl Number for Num {}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct Num(pub f64);

impl Eq for Num {}

impl PartialOrd for Num {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Num {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for Num {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Add for Num {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Num(self.0 + other.0)
    }
}

impl Sub for Num {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Num(self.0 - other.0)
    }
}

impl Mul for Num {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for Num {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Deref for Num {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
