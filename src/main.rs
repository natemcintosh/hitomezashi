use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run();
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
fn draw_equal_dashed_line(draw: &Draw, start: Point2, end: Point2, dash_length: f32) {
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
        draw_equal_dashed_line(
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
        draw_equal_dashed_line(
            draw,
            pt2(current_x_pos, start_y),
            pt2(current_x_pos, bounds.bottom()),
            dash_length,
        );

        // Update x position
        current_x_pos += dash_length;
    }
}

fn view(app: &App, frame: Frame) {
    // Prepare to draw.
    let draw = app.draw();

    // Clear the background to purple.
    draw.background().color(WHITE);

    // Draw a dashed line every 10 units
    draw_hito_horizontal(
        &draw,
        app.window_rect(),
        10.0,
        &[true, false, true, true, false],
    );
    draw_hito_vertical(
        &draw,
        app.window_rect(),
        10.0,
        &[false, true, false, false, true],
    );

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}