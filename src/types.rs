use std::{
    fmt,
    ops::{Add, Div, Mul, Rem, Sub},
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

impl Num for Vector2 {
    type FromStrRadixErr = ParseVectorError;

    // TODO: Change to only accept `0.0 0.0` or `0 0`
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, ParseVectorError> {
        if radix != 10 {
            return Err(ParseVectorError {
                kind: VectorErrorKind::Invalid,
            });
        }

        let res = f32::from_str_radix(str, radix);

        match res {
            Ok(res) => Ok(Vector2 { x: res, y: res }),
            Err(_) => Err(ParseVectorError {
                kind: VectorErrorKind::Invalid,
            }),
        }
    }
}
