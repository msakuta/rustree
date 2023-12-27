use crate::point::Point;

#[derive(Copy, Clone, Debug)]
pub struct BoundingBox {
    pub min: Point,
    pub max: Point,
}

impl BoundingBox {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self {
            min: Point { x: x0, y: y0 },
            max: Point { x: x1, y: y1 },
        }
    }
    pub fn from_minmax(min: Point, max: Point) -> Self {
        Self { min, max }
    }

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

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && other.min.x <= self.max.x
            && self.min.y <= other.max.y
            && other.min.y <= self.max.y
    }
}

impl std::fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}, {})",
            self.min.x, self.min.y, self.max.x, self.max.y
        )
    }
}

#[test]
fn test_intersects() {
    let bb1 = BoundingBox::new(-2., -2., 1., 1.);
    let bb2 = BoundingBox::new(-1., -1., 2., 2.);
    assert!(bb1.intersects(&bb2));
    let bb3 = BoundingBox::new(-2., -2., -1., -1.);
    let bb4 = BoundingBox::new(1., 1., 2., 2.);
    assert!(!bb3.intersects(&bb4));
}
