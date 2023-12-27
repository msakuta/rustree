use std::f64::EPSILON;

use serde::Deserialize;

#[derive(Debug, Deserialize, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
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
