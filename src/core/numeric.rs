use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use std::fmt::Debug;
use num_traits::{Bounded, One, Signed, Zero};

pub trait FlowNum:
    Copy + Ord + Zero + Bounded + Add<Output = Self> + Sub<Output = Self> + AddAssign + SubAssign + Debug
{
}
impl<T> FlowNum for T where
    T: Copy + Ord + Zero + Bounded + Add<Output = T> + Sub<Output = T> + AddAssign + SubAssign + Debug
{
}

pub trait CostNum:
    Copy
    + Ord
    + Zero
    + One
    + Signed
    + Add<Output = Self>
    + Sub<Output = Self>
    + AddAssign
    + SubAssign
    + Neg<Output = Self>
{
}
impl<T> CostNum for T where
    T: Copy
        + Ord
        + Zero
        + One
        + Signed
        + Add<Output = T>
        + Sub<Output = T>
        + AddAssign
        + SubAssign
        + Neg<Output = T>
        + Default
{
}

#[inline]
pub fn inf<T: Bounded>() -> T {
    T::max_value()
}

pub trait GainNum: Copy + PartialOrd + One + Mul<Output = Self> + Div<Output = Self> {}
impl<T> GainNum for T where T: Copy + PartialOrd + One + Mul<Output = T> + Div<Output = T> {}
