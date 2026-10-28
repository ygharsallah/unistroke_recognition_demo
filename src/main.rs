mod shapes;

use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use unistroke_recognition::prelude::{Point, Recognizer, Shape};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .add_plugins(EguiPlugin::default())
        .init_resource::<Demo>()
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        .add_systems(EguiPrimaryContextPass, drawing_system)
        .run();
}

#[derive(Resource)]
struct Demo {
    recognizer: Recognizer,
    points: Vec<egui::Vec2>,
    scores: Vec<(Shape, f32)>,
}

impl Default for Demo {
    fn default() -> Self {
        let mut recognizer = Recognizer::new(None);
        for shape in shapes::templates() {
            recognizer.add_shape(shape);
        }
        Self {
            recognizer,
            points: Vec::new(),
            scores: Vec::new(),
        }
    }
}

fn drawing_system(mut contexts: EguiContexts, mut demo: ResMut<Demo>) -> Result {
    let context = contexts.ctx_mut()?;
    let mut viewport = create_viewport(context);

    show_scores(&mut viewport, &demo);
    show_drawing_canvas(&mut viewport, &mut demo);

    Ok(())
}

fn create_viewport(context: &egui::Context) -> egui::Ui {
    egui::Ui::new(
        context.clone(),
        egui::Id::new("viewport"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(context.viewport_rect()),
    )
}

fn show_scores(viewport: &mut egui::Ui, demo: &Demo) {
    egui::Panel::right("shapes")
        .default_size(250.0)
        .show(viewport, |ui| {
            ui.heading("Confidence scores");

            if demo.scores.is_empty() {
                for shape in demo.recognizer.get_shapes() {
                    ui.monospace(format!("{}: --", shape.id()));
                }
            } else {
                for (shape, score) in &demo.scores {
                    ui.monospace(format!("{}: {:.1}%", shape.id(), score * 100.0));
                }
            }

            ui.separator();
        });
}

fn show_drawing_canvas(viewport: &mut egui::Ui, demo: &mut Demo) {
    egui::CentralPanel::default().show(viewport, |ui| {
        if ui.button("Clear painting").clicked() {
            clear_drawing(demo);
        }

        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());
        let rect = response.rect;

        if response.drag_started_by(egui::PointerButton::Primary) {
            clear_drawing(demo);
        }

        record_pointer_input(&response, rect, demo);
        paint_canvas(ui, painter, rect, demo);
    });
}

fn clear_drawing(demo: &mut Demo) {
    demo.points.clear();
    demo.scores.clear();
}

fn record_pointer_input(response: &egui::Response, rect: egui::Rect, demo: &mut Demo) {
    let is_drawing = response.dragged_by(egui::PointerButton::Primary)
        || response.drag_stopped_by(egui::PointerButton::Primary);

    if !is_drawing {
        return;
    }

    let Some(pos) = response.interact_pointer_pos() else {
        return;
    };

    let point = rect.clamp(pos) - rect.min;
    if demo.points.last() == Some(&point) {
        return;
    }

    demo.points.push(point);
    if demo.points.len() < 2 {
        return;
    }

    // Point requires [-1, 1]; the library handles normalization.
    let scale = rect.width().max(rect.height()).max(1.0);
    let points: Vec<_> = demo
        .points
        .iter()
        .map(|p| Point::new(p.x / scale, -p.y / scale).unwrap())
        .collect();
    demo.scores = demo.recognizer.parse(&points);
}

fn paint_canvas(ui: &egui::Ui, painter: egui::Painter, rect: egui::Rect, demo: &Demo) {
    painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);

    if demo.points.len() < 2 {
        return;
    }

    painter.add(egui::Shape::line(
        demo.points.iter().map(|p| rect.min + *p).collect(),
        egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE),
    ));
}
