use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

// ------------------------------------------------------------
// 0. Zero / One / Bounded – 基本数値ユーティリティ
// ------------------------------------------------------------
pub trait Zero {
    fn zero() -> Self;
}
pub trait One {
    fn one() -> Self;
}
pub trait Bounded {
    fn max_value() -> Self;
}

pub trait FromPrimitive: Sized {
    fn from_u8(n: u8) -> Self;
    fn from_i64(n: i64) -> Self;
}
macro_rules! impl_from_prim {
    ($($t:ty),*) => {$(
        impl FromPrimitive for $t {
            #[inline] fn from_u8(n: u8)  -> Self { n as $t }
            #[inline] fn from_i64(n: i64)-> Self { n as $t }
        }
    )*};
}
impl_from_prim!(u16, u32, u64, u128, usize, i16, i32, i64, i128, f32, f64);

macro_rules! impl_nums {
    ($($t:ty => $max:expr),* $(,)?) => { $(
        impl Zero for $t { #[inline] fn zero() -> Self { 0 as $t } }
        impl One  for $t { #[inline] fn one()  -> Self { 1 as $t } }
        impl Bounded for $t { #[inline] fn max_value() -> Self { $max } }
    )* };
}
impl_nums!(
    usize => usize::MAX,
    u16 => u16::MAX,
    u32 => u32::MAX,
    u64 => u64::MAX,
    u128 => u128::MAX,
    i16 => i16::MAX,
    i32 => i32::MAX,
    i64 => i64::MAX,
    i128 => i128::MAX,
    f32 => f32::INFINITY,
    f64 => f64::INFINITY,
);

// ------------------------------------------------------------
// 1. 数値カテゴリトレイト（整数 / 浮動小数）
// ------------------------------------------------------------
pub trait NumBase: Copy + Zero + Bounded + Add<Output = Self> + Sub<Output = Self> + AddAssign + SubAssign {}
impl<T> NumBase for T where T: Copy + Zero + Bounded + Add<Output = T> + Sub<Output = T> + AddAssign + SubAssign {}

pub trait IntNum: NumBase + Ord {}
impl<T: NumBase + Ord> IntNum for T {}

pub trait FloatNum: NumBase + PartialOrd + Neg<Output = Self> {}
impl FloatNum for f32 {}
impl FloatNum for f64 {}

// ------------------------------------------------------------
// 2. Edge 属性抽象トレイト
// ------------------------------------------------------------

pub trait HasCap {
    type Cap: NumBase;
    fn cap(&self) -> Self::Cap;
}

pub trait HasCost: HasCap {
    type Cost: NumBase;
    fn cost(&self) -> Self::Cost;
}
