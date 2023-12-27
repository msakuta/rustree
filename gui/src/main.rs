use ::rustree::{Point, RTree};
use eframe::{
    egui::{self, Context, Ui},
    epaint::{pos2, Color32, Pos2, Rect},
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

struct RustreeApp {
    rtree: RTree<Point>,
    offset: Pos2,
}

impl RustreeApp {
    fn new() -> Self {
        let mut rtree = RTree::new();
        let mut try_add = |x, y| {
            let pt = Point { x, y };
            rtree.insert_entry(pt, BoundingBox { min: pt, max: pt });
            println!("inserted: {rtree:?}");
            println!("now bb: {:?}", rtree.bounding_box());
        };
        try_add(2., 0.);
        try_add(-2., 0.);
        try_add(1., 7.);
        try_add(1., 5.);
        try_add(-1., -5.);
        Self {
            rtree,
            offset: pos2(100., 100.),
        }
    }

    fn paint(&mut self, ui: &mut Ui) {
        let (_response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click());

        self.rtree.walk(&mut |_id, _level, bb| {
            if bb.get_area() == 0. {
                painter.circle(
                    pos2(bb.min.x as f32 * 10., bb.min.y as f32 * 10.) + self.offset.to_vec2(),
                    3.,
                    Color32::RED,
                    (1., Color32::GREEN),
                );
            } else {
                painter.rect_stroke(
                    Rect {
                        min: pos2(bb.min.x as f32 * 10., bb.min.y as f32 * 10.)
                            + self.offset.to_vec2(),
                        max: pos2(bb.max.x as f32 * 10., bb.max.y as f32 * 10.)
                            + self.offset.to_vec2(),
                    },
                    0.,
                    (1., Color32::BLUE),
                );
            }
        });
    }
}

impl eframe::App for RustreeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        eframe::egui::CentralPanel::default()
            // .resizable(true)
            // .min_height(100.)
            .show(ctx, |ui| {
                eframe::egui::Frame::canvas(ui.style()).show(ui, |ui| self.paint(ui));
            });
    }
}
