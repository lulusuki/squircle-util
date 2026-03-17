use std::{
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

use num::{Num, One, Zero};

#[derive(Debug)]
pub enum VectorErrorKind {
    Empty,
    Invalid,
    Radix,
}

#[derive(Debug)]
pub struct ParseVectorError {
    kind: VectorErrorKind,
}

impl fmt::Display for ParseVectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.kind {
            VectorErrorKind::Empty => "cannot parse vector from empty string",
            VectorErrorKind::Invalid => "invalid vector literal",
            VectorErrorKind::Radix => "invalid radix literal",
        };

        description.fmt(f)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let mut res = self;

        res.x -= rhs.x;
        res.y -= rhs.y;

        res
    }
}

impl Div for Vector2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let mut res = self;
        res.x /= rhs.x;
        res.y /= rhs.y;

        res
    }
}

impl Mul for Vector2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut res = self;
        res.x *= rhs.x;
        res.y *= rhs.y;

        res
    }
}

impl Rem for Vector2 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        let mut res = self;
        res.x %= rhs.x;
        res.y %= rhs.y;

        res
    }
}

impl One for Vector2 {
    fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    fn is_one(&self) -> bool {
        self.x == 1.0 && self.y == 1.0
    }

    fn set_one(&mut self) {
        self.x = 1.0;
        self.y = 1.0;
    }
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let mut res = self;
        res.x += rhs.x;
        res.y += rhs.y;

        res
    }
}

impl Zero for Vector2 {
    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    fn set_zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }
}

impl AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vector2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign for Vector2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl DivAssign for Vector2 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl RemAssign for Vector2 {
    fn rem_assign(&mut self, rhs: Self) {
        self.x %= rhs.x;
        self.y %= rhs.y;
    }
}

impl Num for Vector2 {
    type FromStrRadixErr = ParseVectorError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, ParseVectorError> {
        let mut values = str.split("_");

        let x = values.next().ok_or(ParseVectorError {
            kind: VectorErrorKind::Invalid,
        })?;

        let y = values.next().ok_or(ParseVectorError {
            kind: VectorErrorKind::Invalid,
        })?;

        let res_x = f32::from_str_radix(x, radix).map_err(|_| ParseVectorError {
            kind: VectorErrorKind::Invalid,
        })?;

        let res_y = f32::from_str_radix(y, radix).map_err(|_| ParseVectorError {
            kind: VectorErrorKind::Invalid,
        })?;

        Ok(Vector2 { x: res_x, y: res_y })
    }
}
