//! 精确有理数类型，所有时间运算必须使用此类型

use num::rational::Rational64;
use num::ToPrimitive;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// 有理数类型，封装 Rational64 以支持 From 实现
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Q(pub Rational64);

impl Q {
    pub fn from_integer(n: i64) -> Self {
        Q(Rational64::from_integer(n))
    }
}

impl From<i32> for Q {
    fn from(n: i32) -> Self {
        Q::from_integer(n as i64)
    }
}

impl From<u32> for Q {
    fn from(n: u32) -> Self {
        Q::from_integer(n as i64)
    }
}

impl From<i64> for Q {
    fn from(n: i64) -> Self {
        Q::from_integer(n)
    }
}

impl From<usize> for Q {
    fn from(n: usize) -> Self {
        Q::from_integer(n as i64)
    }
}

impl Add for Q {
    type Output = Q;
    fn add(self, rhs: Self) -> Self::Output {
        Q(self.0 + rhs.0)
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, rhs: Self) -> Self::Output {
        Q(self.0 - rhs.0)
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, rhs: Self) -> Self::Output {
        Q(self.0 * rhs.0)
    }
}

impl Div for Q {
    type Output = Q;
    fn div(self, rhs: Self) -> Self::Output {
        Q(self.0 / rhs.0)
    }
}

impl Add for &Q {
    type Output = Q;
    fn add(self, rhs: Self) -> Self::Output {
        Q(&self.0 + &rhs.0)
    }
}

impl Sub for &Q {
    type Output = Q;
    fn sub(self, rhs: Self) -> Self::Output {
        Q(&self.0 - &rhs.0)
    }
}

impl Mul for &Q {
    type Output = Q;
    fn mul(self, rhs: Self) -> Self::Output {
        Q(&self.0 * &rhs.0)
    }
}

impl Div for &Q {
    type Output = Q;
    fn div(self, rhs: Self) -> Self::Output {
        Q(&self.0 / &rhs.0)
    }
}

impl Div<Q> for &Q {
    type Output = Q;
    fn div(self, rhs: Q) -> Self::Output {
        Q(&self.0 / rhs.0)
    }
}

impl std::ops::AddAssign for Q {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign for Q {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::SubAssign<&Q> for Q {
    fn sub_assign(&mut self, rhs: &Q) {
        self.0 -= &rhs.0;
    }
}

impl Neg for Q {
    type Output = Q;
    fn neg(self) -> Self::Output {
        Q(-self.0)
    }
}

impl Neg for &Q {
    type Output = Q;
    fn neg(self) -> Self::Output {
        Q(-&self.0)
    }
}

/// 将 Q 转为 f64（用于 LP 求解器等）
pub fn q_to_f64(q: &Q) -> f64 {
    q.0.to_f64().unwrap_or(0.0)
}

/// 从 f64 近似转换为 Q（用于 LP 结果）
pub fn f64_to_q(f: f64) -> Option<Q> {
    Rational64::approximate_float::<f64>(f).map(Q)
}
