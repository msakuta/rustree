mod data;

use ::rustree::{Point, RTree, RTreeNode};
use data::{ConvexHull, ConvexHulls};
use eframe::{
    egui::{self, Context, Ui},
    emath::Align2,
    epaint::{pos2, vec2, Color32, FontId, PathShape, Pos2, Rect},
};
use rustree::BoundingBox;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut native_options = eframe::NativeOptions::default();

    // We insist to use light theme, because the canvas color is designed to work with light background.
    native_options.follow_system_theme = false;
    native_options.default_theme = eframe::Theme::Light;

    eframe::run_native(
        "rustree GUI",
        native_options,
        Box::new(|_cc| Box::new(RustreeApp::new())),
    )
    .unwrap();
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    AddPoint,
    Query,
}

struct RustreeApp {
    mode: Mode,
    query_radius: f64,
    rtree: RTree<ConvexHull>,
    offset: Pos2,
    scale: f32,
}

impl RustreeApp {
    fn new() -> Self {
        Self {
            mode: Mode::AddPoint,
            query_radius: 2.,
            rtree: Self::reset(),
            offset: pos2(300., 300.),
            scale: 5.,
        }
    }

    fn paint(&mut self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click());

        if self.mode == Mode::AddPoint && response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let screen_pos = (pos - self.offset.to_vec2()) / self.scale;
                let pt = Point {
                    x: screen_pos.x as f64,
                    y: screen_pos.y as f64,
                };
                // self.rtree
                //     .insert_entry(pt, BoundingBox::from_minmax(pt, pt));
            }
        }

        self.rtree.walk(&mut |id, level, entry| {
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
                (1., Color32::BLUE),
            );
        });

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
        let json = std::fs::read("convex_hulls.json").unwrap();
        let deserialized: ConvexHulls =
            serde_json::from_str(std::str::from_utf8(&json).unwrap()).unwrap();
        let mut rtree = RTree::new();
        for c_hull in deserialized.convex_hulls {
            if let Some(bbox) = c_hull.bounding_box() {
                rtree.insert_entry(c_hull, bbox);
            }
        }
        // let mut rtree = RTree::new();
        // let mut try_add = |x, y| {
        //     let pt = Point { x, y };
        //     rtree.insert_entry(pt, BoundingBox { min: pt, max: pt });
        // };
        // try_add(2., 0.);
        // try_add(-2., 1.);
        // try_add(1., 7.);
        // try_add(0., 5.);
        // try_add(-1., -5.);
        rtree
    }

    fn show_side_panel(&mut self, ui: &mut Ui) {
        if ui.button("Reset").clicked() {
            // self.rtree = Self::reset();
        }

        ui.group(|ui| {
            ui.label("Click mode:");
            ui.radio_value(&mut self.mode, Mode::AddPoint, "Add point");
            ui.radio_value(&mut self.mode, Mode::Query, "Query");
        });

        ui.label("Query radius:");
        ui.add(egui::widgets::Slider::new(
            &mut self.query_radius,
            (0.1)..=10.,
        ));

        let mut s = "id, level\n".to_string();
        self.rtree.walk(&mut |id, level, _bb| {
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
