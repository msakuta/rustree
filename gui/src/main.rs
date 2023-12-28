use ::rustree::{Point, RTree};
use eframe::{
    egui::{self, Context, Ui},
    emath::Align2,
    epaint::{pos2, Color32, FontId, Pos2, Rect},
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
        };
        try_add(2., 0.);
        try_add(-2., 1.);
        try_add(1., 7.);
        try_add(0., 5.);
        try_add(-1., -5.);
        Self {
            rtree,
            offset: pos2(100., 100.),
        }
    }

    fn paint(&mut self, ui: &mut Ui) {
        let (_response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click());

        if _response.clicked() {
            if let Some(pos) = _response.interact_pointer_pos() {
                let screen_pos = (pos - self.offset.to_vec2()) / 10.;
                let pt = Point {
                    x: screen_pos.x as f64,
                    y: screen_pos.y as f64,
                };
                self.rtree
                    .insert_entry(pt, BoundingBox::from_minmax(pt, pt));
            }
        }

        self.rtree.walk(&mut |id, level, bb| {
            if bb.get_area() == 0. {
                let pos =
                    pos2(bb.min.x as f32 * 10., bb.min.y as f32 * 10.) + self.offset.to_vec2();
                painter.circle(pos, 3., Color32::RED, (1., Color32::GREEN));
                painter.text(
                    pos,
                    Align2::LEFT_CENTER,
                    format!("{}, {}", id, level),
                    FontId::new(16., egui::FontFamily::Proportional),
                    Color32::BLACK,
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

    fn show_side_panel(&mut self, ui: &mut Ui) {
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
