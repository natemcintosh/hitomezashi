use nannou::{prelude::*, rand::Rng};
use nannou_egui::{egui, Egui};
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

enum ShapeSettings {
    Rectangle {
        spacing: f32,
        horz_selectors: Vec<bool>,
        vert_selectors: Vec<bool>,
        horz_seed: u8,
        vert_seed: u8,
    },
    Triangle {
        spacing: f32,
        s1_selectors: Vec<bool>,
        s2_selectors: Vec<bool>,
        s3_selectors: Vec<bool>,
        s1_seed: u8,
        s2_seed: u8,
        s3_seed: u8,
    },
}

impl ShapeSettings {
    fn new(spacing: f32) -> Self {
        // By default, create a rectangle
        ShapeSettings::Rectangle {
            spacing,
            horz_selectors: vec![false; 10],
            vert_selectors: vec![false; 10],
            horz_seed: 0,
            vert_seed: 0,
        }
    }

    fn display(&self, draw: &Draw, bounds: Rect) {
        match self {
            ShapeSettings::Rectangle {
                spacing,
                horz_selectors,
                vert_selectors,
                horz_seed,
                vert_seed,
            } => {
                draw_hito_vertical(draw, bounds, *spacing, vert_selectors);
                draw_hito_horizontal(draw, bounds, *spacing, horz_selectors);
            }
            ShapeSettings::Triangle {
                spacing,
                s1_selectors,
                s2_selectors,
                s3_selectors,
                s1_seed,
                s2_seed,
                s3_seed,
            } => todo!(),
        }
    }
}

struct RectSettings {
    spacing: f32,
    horz_selectors: Vec<bool>,
    vert_selectors: Vec<bool>,
    horz_seed: u8,
    vert_seed: u8,
}

struct Model {
    settings: ShapeSettings,
    egui: Egui,
}

fn main() {
    nannou::app(model)
        .loop_mode(LoopMode::Wait)
        .update(update)
        .run();
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    Model {
        egui,
        settings: ShapeSettings::new(25.0),
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn update(_app: &App, model: &mut Model, update: Update) {
    // This destructures the model, giving mutable references to the settings and egui
    // but without having to prefix them with `model` every time they are accessed.
    let Model {
        ref mut settings,
        ref mut egui,
    } = *model;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui| {
        let mut changed = false;
        changed |= ui
            .add(egui::Slider::new(&mut settings.spacing, 10.0..=100.0).text("Spacing"))
            .changed();

        changed |= ui
            .add(egui::Slider::new(&mut settings.horz_seed, 0..=255).text("Horizontal Seed"))
            .changed();

        changed |= ui
            .add(egui::Slider::new(&mut settings.vert_seed, 0..=255).text("Vertical Seed"))
            .changed();

        if changed {
            let mut rng: Pcg64 = Seeder::from(settings.horz_seed).make_rng();
            rng.fill(&mut settings.horz_selectors[..]);

            let mut rng: Pcg64 = Seeder::from(settings.vert_seed).make_rng();
            rng.fill(&mut settings.vert_selectors[..]);
        }
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Prepare to draw.
    let draw = app.draw();

    // Set the background color
    draw.background().color(WHITE);

    // Draw the pattern as specified by the model settings
    draw_hito_horizontal(
        &draw,
        app.window_rect(),
        model.settings.spacing,
        &model.settings.horz_selectors,
    );

    draw_hito_vertical(
        &draw,
        app.window_rect(),
        model.settings.spacing,
        &model.settings.vert_selectors,
    );

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

#[allow(dead_code)]
fn draw_vertical_lines(draw: &Draw, bounds: Rect, spacing: f32) {
    let mut current_x_pos = bounds.left() + spacing;

    while current_x_pos < bounds.right() {
        // Draw the line from top to bottom
        draw.line()
            .start(pt2(current_x_pos, bounds.top()))
            .end(pt2(current_x_pos, bounds.bottom()))
            .weight(3.0);

        // Update drawing position
        current_x_pos += spacing;
    }
}

#[allow(dead_code)]
fn draw_horizontal_lines(draw: &Draw, bounds: Rect, spacing: f32) {
    // Start at the top and go down
    let mut current_y_pos = bounds.top() + spacing;

    while current_y_pos > bounds.bottom() {
        // Draw the line from left to right
        draw.line()
            .start(pt2(bounds.left(), current_y_pos))
            .end(pt2(bounds.right(), current_y_pos))
            .weight(3.0);

        // Move drawing position down
        current_y_pos -= spacing;
    }
}

/// Draws a dashed line from `start` to `end`. The length of each dash is the same as the
/// length of each gap.
fn draw_dashed_line(draw: &Draw, start: Point2, end: Point2, dash_length: f32) {
    // Create a vector poiting from `start` to `end`, of lengh `dash_length`
    let draw_direction = (end - start).normalize() * dash_length;

    // Create points at which to start and end drawing the line
    let mut sdraw = start;
    let mut edraw = start + draw_direction;

    // While the distance from `start` to `end` is longer than from `start` to `edraw`
    while start.distance(end) > start.distance(edraw) {
        // Draw the dash
        draw.line().start(sdraw).end(edraw).weight(3.0);

        // Increment `sdraw` and `edraw`
        sdraw = edraw + draw_direction;
        edraw = sdraw + draw_direction;
    }
}

/// Draw horizontal dashed lines with `dash_length` dashes and `dash_length` spacing between lines.
/// The `on_off_selectors` pair up with each line. If true, then it starts with a dash,
/// if false it starts with a space. If the bounds go farther than the `on_off_selectors`
/// then `idx % on_off_selectors.len()` is used to continue selecting bools from it.
fn draw_hito_horizontal(draw: &Draw, bounds: Rect, dash_length: f32, on_off_selectors: &[bool]) {
    // Start at the top and go down
    let mut current_y_pos = bounds.top();

    // What index of `on_off_selectors` are we on
    let mut selector_idx = 0;

    // Move down the window
    while current_y_pos > bounds.bottom() {
        let start_x = if on_off_selectors[selector_idx % on_off_selectors.len()] {
            bounds.left()
        } else {
            bounds.left() + dash_length
        };
        // bump `selector_idx`
        selector_idx += 1;

        // Draw the line
        draw_dashed_line(
            draw,
            pt2(start_x, current_y_pos),
            pt2(bounds.right(), current_y_pos),
            dash_length,
        );

        // Update y position
        current_y_pos -= dash_length;
    }
}

/// Draw vertical dashed lines with `dash_length` dashes and `dash_length` spacing between lines.
/// The `on_off_selectors` pair up with each line. If true, then it starts with a dash,
/// if false it starts with a space. If the bounds go farther than the `on_off_selectors`
/// then `idx % on_off_selectors.len()` is used to continue selecting bools from it.
fn draw_hito_vertical(draw: &Draw, bounds: Rect, dash_length: f32, on_off_selectors: &[bool]) {
    // Move from left to right
    let mut current_x_pos = bounds.left();

    // What index of `on_off_selectors` are we on
    let mut selector_idx = 0;

    // Move right across the window
    while current_x_pos < bounds.right() {
        let start_y = if on_off_selectors[selector_idx % on_off_selectors.len()] {
            bounds.top()
        } else {
            bounds.top() + dash_length
        };
        // bump `selector_idx`
        selector_idx += 1;

        // Draw the line
        draw_dashed_line(
            draw,
            pt2(current_x_pos, start_y),
            pt2(current_x_pos, bounds.bottom()),
            dash_length,
        );

        // Update x position
        current_x_pos += dash_length;
    }
}

/// Draw angled dashed lines with `dash_length` dashes and `dash_length` spacing between lines.
/// The `on_off_selectors` pair up with each line. If true, then it starts with a dash,
/// if false it starts with a space. If the bounds go farther than the `on_off_selectors`
/// then `idx % on_off_selectors.len()` is used to continue selecting bools from it.
/// `degs` is the number of degrees the lines should be angled: [0, 90] where 0 is horizontal
/// and 90 is vertical.
fn draw_hito_angled(
    draw: &Draw,
    bounds: Rect,
    dash_length: f32,
    on_off_selectors: &[bool],
    degs: f32,
) {
    /*
    I think that perhaps we could calculate the vertical spacing between lines, figure out
    the equation of the line in the form `x + y + value = 0`, then use the `.x()` and
    `.y()` methods to set the line.
    */
}
