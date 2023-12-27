use crate::point::Point;

#[derive(Copy, Clone, Debug)]
pub struct BoundingBox {
    pub min: Point,
    pub max: Point,
}

impl BoundingBox {
    pub fn get_union(&self, b: &BoundingBox) -> BoundingBox {
        return BoundingBox {
            min: Point {
                x: f64::min(self.min.x, b.min.x),
                y: f64::min(self.min.y, b.min.y),
            },
            max: Point {
                x: f64::max(self.max.x, b.max.x),
                y: f64::max(self.max.y, b.max.y),
            },
        };
    }

    pub fn get_area(&self) -> f64 {
        return (self.max.x - self.min.x) * (self.max.y - self.min.y);
    }
}
