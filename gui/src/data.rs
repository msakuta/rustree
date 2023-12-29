use rustree::{BoundingBox, Point};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ConvexHull {
    #[serde(rename = "ID")]
    pub id: usize,
    pub apexes: Vec<Point>,
}

impl ConvexHull {
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        let mut min: Option<Point> = None;
        let mut max: Option<Point> = None;
        for apex in &self.apexes {
            min = min
                .map(|min| Point::new(min.x.min(apex.x), min.y.min(apex.y)))
                .or(Some(*apex));
            max = max
                .map(|max| Point::new(max.x.max(apex.x), max.y.max(apex.y)))
                .or(Some(*apex));
        }
        min.zip(max)
            .map(|(min, max)| BoundingBox::from_minmax(min, max))
    }
}

#[derive(Deserialize)]
pub struct ConvexHulls {
    #[serde(rename = "convex hulls")]
    pub convex_hulls: Vec<ConvexHull>,
}
