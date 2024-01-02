use crate::data::{ConvexHull, ConvexHulls};
use ::rustree::{Point, RTree, RTreeNode, WalkCallbackPayload};
use eframe::{
    egui::{self, Context, Ui},
    emath::Align2,
    epaint::{pos2, vec2, Color32, FontId, PathShape, Pos2, Rect},
};
use rustree::BoundingBox;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    AddPoint,
    AddPolygon,
    Query,
}

pub struct RustreeApp {
    mode: Mode,
    query_radius: f64,
    adding_polygon: Vec<Point>,
    rtree: RTree<ConvexHull>,
    offset: Pos2,
    scale: f32,
}

impl RustreeApp {
    pub fn new() -> Self {
        Self {
            mode: Mode::AddPoint,
            query_radius: 2.,
            adding_polygon: vec![],
            rtree: Self::reset(),
            offset: pos2(300., 300.),
            scale: 5.,
        }
    }

    fn paint(&mut self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click());

        match self.mode {
            Mode::AddPolygon => {
                if response.clicked_by(egui::PointerButton::Primary) {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pos = (pos - self.offset.to_vec2()) / self.scale;
                        let pt = Point {
                            x: screen_pos.x as f64,
                            y: screen_pos.y as f64,
                        };
                        self.adding_polygon.push(pt);
                    }
                } else if response.clicked_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pos = (pos - self.offset.to_vec2()) / self.scale;
                        let pt = Point {
                            x: screen_pos.x as f64,
                            y: screen_pos.y as f64,
                        };
                        self.adding_polygon.push(pt);
                        if 3 <= self.adding_polygon.len() {
                            let c_hull = ConvexHull {
                                id: 0,
                                apexes: std::mem::take(&mut self.adding_polygon),
                            };
                            if let Some(bb) = c_hull.bounding_box() {
                                self.rtree.insert_entry(c_hull, bb);
                            }
                        }
                    }
                }
            }
            Mode::AddPoint => {
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pos = (pos - self.offset.to_vec2()) / self.scale;
                        let pt = Point {
                            x: screen_pos.x as f64,
                            y: screen_pos.y as f64,
                        };
                        use std::f64::consts::PI;
                        let theta0 = rand::random::<f64>() * PI * 2.;
                        let r = rand::random::<f64>() * 3. + 3.;
                        let apexes = (0..5)
                            .map(|i| {
                                let theta = i as f64 * PI * 2. / 5. + theta0;
                                let x = r * theta.cos();
                                let y = r * theta.sin();
                                Point::new(x, y) + pt
                            })
                            .collect();
                        let c_hull = ConvexHull { id: 0, apexes };
                        if let Some(bb) = c_hull.bounding_box() {
                            self.rtree.insert_entry(c_hull, bb);
                        }
                    }
                }
            }
            _ => {}
        }

        const PALETTE: [Color32; 3] = [
            Color32::from_rgb(127, 127, 255),
            Color32::from_rgb(255, 127, 255),
            Color32::from_rgb(255, 127, 127),
        ];

        let max_depth = self.rtree.max_depth();

        self.rtree.walk(&mut |payload| {
            let WalkCallbackPayload {
                id, level, entry, ..
            } = payload;
            let bb = entry.bounding_box();
            match entry.node() {
                RTreeNode::Node(_) => {
                    let pos = pos2(bb.min.x as f32 * self.scale, bb.min.y as f32 * self.scale)
                        + self.offset.to_vec2();
                    // painter.circle(pos, 3., Color32::GRAY, (1., Color32::from_rgb(0, 127, 0)));
                    painter.text(
                        pos,
                        Align2::LEFT_CENTER,
                        format!("{}, {}", id, level),
                        FontId::new(16., egui::FontFamily::Proportional),
                        Color32::BLACK,
                    );
                }
                RTreeNode::Leaf(c_hull) => {
                    painter.add(PathShape::convex_polygon(
                        c_hull
                            .apexes
                            .iter()
                            .map(|pt| {
                                pos2(pt.x as f32 * self.scale, pt.y as f32 * self.scale)
                                    + self.offset.to_vec2()
                            })
                            .collect(),
                        Color32::from_rgba_premultiplied(127, 255, 0, 63),
                        (1., Color32::from_rgb(63, 95, 0)),
                    ));
                }
            }

            painter.rect_stroke(
                Rect {
                    min: pos2(bb.min.x as f32 * self.scale, bb.min.y as f32 * self.scale)
                        + self.offset.to_vec2(),
                    max: pos2(bb.max.x as f32 * self.scale, bb.max.y as f32 * self.scale)
                        + self.offset.to_vec2(),
                },
                0.,
                (
                    (max_depth - level + 1) as f32,
                    PALETTE[level % PALETTE.len()],
                ),
            );
        });

        if self.mode == Mode::AddPolygon {
            let transform_point = |pt: &Point| {
                pos2(pt.x as f32 * self.scale, pt.y as f32 * self.scale) + self.offset.to_vec2()
            };

            if 2 <= self.adding_polygon.len() {
                painter.add(PathShape::line(
                    self.adding_polygon
                        .iter()
                        .map(|pt| transform_point(pt))
                        .collect(),
                    (1., Color32::from_rgb(127, 41, 41)),
                ));
            }

            if let Some(cursor) = response.hover_pos() {
                if let Some(point) = self.adding_polygon.last() {
                    painter.line_segment(
                        [transform_point(point), cursor],
                        (2., Color32::from_rgb(95, 31, 31)),
                    );
                }
            }
        }

        if self.mode == Mode::Query {
            if let Some(pos) = response.hover_pos() {
                let point_pos = (pos - self.offset.to_vec2()) / self.scale;
                let pt = Point::new(point_pos.x as f64, point_pos.y as f64);
                for c_hull in self.rtree.find_multi(&BoundingBox::from_center_size(
                    pt,
                    Point::new(self.query_radius, self.query_radius),
                )) {
                    painter.add(PathShape::convex_polygon(
                        c_hull
                            .apexes
                            .iter()
                            .map(|pt| {
                                pos2(pt.x as f32 * self.scale, pt.y as f32 * self.scale)
                                    + self.offset.to_vec2()
                            })
                            .collect(),
                        Color32::from_rgb(63, 95, 0),
                        (2., Color32::RED),
                    ));
                    // let pos =
                    //     pos2(node.x as f32 * self.scale, node.y as f32 * self.scale) + self.offset.to_vec2();
                    // // println!("node: {node:?}")
                    // painter.circle(
                    //     pos,
                    //     5.,
                    //     Color32::RED,
                    //     (2., Color32::GREEN),
                    // );
                }

                // Query rectangle
                let screen_radius = (self.query_radius * self.scale as f64) as f32;
                let screen_offset = vec2(screen_radius, screen_radius);
                painter.rect(
                    Rect {
                        min: pos - screen_offset,
                        max: pos + screen_offset,
                    },
                    0.,
                    Color32::from_rgba_unmultiplied(0, 255, 0, 63),
                    (2., Color32::from_rgb(0, 255, 0)),
                );
            }
        }
    }

    fn reset() -> RTree<ConvexHull> {
        let json = std::fs::read("convex_hulls.json").unwrap_or_else(|_| include_str!("../../convex_hulls.json").as_bytes().to_vec());
        let deserialized: ConvexHulls =
            serde_json::from_str(std::str::from_utf8(&json).unwrap()).unwrap();
        let mut rtree = RTree::new();
        for c_hull in deserialized.convex_hulls {
            if let Some(bbox) = c_hull.bounding_box() {
                rtree.insert_entry(c_hull, bbox);
            }
        }
        rtree
    }

    fn show_side_panel(&mut self, ui: &mut Ui) {
        if ui.button("Reset").clicked() {
            self.rtree = Self::reset();
        }

        ui.group(|ui| {
            ui.label("Click mode:");
            ui.radio_value(&mut self.mode, Mode::AddPoint, "Add point");
            ui.radio_value(
                &mut self.mode,
                Mode::AddPolygon,
                "Add polygon (right click to close polygon)",
            );
            ui.radio_value(&mut self.mode, Mode::Query, "Query");
        });

        ui.label("Query radius:");
        ui.add(egui::widgets::Slider::new(
            &mut self.query_radius,
            (0.1)..=10.,
        ));

        let mut s = "id, level\n".to_string();
        self.rtree
            .walk(&mut |WalkCallbackPayload { id, level, .. }| {
                s += &format!("{:width$}{id}, {level}\n", " ", width = level * 2);
            });
        ui.label(s);
    }
}

impl eframe::App for RustreeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        eframe::egui::SidePanel::right("side_panel")
            .min_width(200.)
            .show(ctx, |ui| {
                eframe::egui::ScrollArea::vertical().show(ui, |ui| self.show_side_panel(ui));
            });

        eframe::egui::CentralPanel::default()
            // .resizable(true)
            // .min_height(100.)
            .show(ctx, |ui| {
                eframe::egui::Frame::canvas(ui.style()).show(ui, |ui| self.paint(ui));
            });
    }
}
