use std::f64::EPSILON;

use serde::Deserialize;

#[derive(Debug, Deserialize, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Clone for Point {
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        f64::abs(self.x - other.x) < EPSILON && f64::abs(self.y - other.y) < EPSILON
    }
}

impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, other: Point) -> Point {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Point {
    type Output = Self;
    fn sub(self, other: Point) -> Point {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
