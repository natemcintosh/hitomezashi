use nannou::{
    prelude::*,
    rand::{Rng, SeedableRng},
};
use nannou_egui::{egui, Egui};
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::{BufReader, BufWriter};

struct RectSettings {
    spacing: f32,
    horz_selectors: Vec<bool>,
    vert_selectors: Vec<bool>,
    horz_seed: u8,
    vert_seed: u8,
}

struct Model {
    settings: RectSettings,
    egui: Egui,
    save_requested: bool,
    hide_ui_for_save: bool,
    save_path: Option<std::path::PathBuf>,
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
        settings: RectSettings {
            spacing: 25.0,
            horz_selectors: vec![false; 10],
            vert_selectors: vec![false; 10],
            horz_seed: 0,
            vert_seed: 0,
        },
        save_requested: false,
        hide_ui_for_save: false,
        save_path: None,
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn update(app: &App, model: &mut Model, update: Update) {
    // This destructures the model, giving mutable references to the settings and egui
    // but without having to prefix them with `model` every time they are accessed.
    let Model {
        ref mut settings,
        ref mut egui,
        ref mut save_requested,
        ref mut hide_ui_for_save,
        ref mut save_path,
    } = *model;

    // Reset flags after save is complete
    if *hide_ui_for_save && *save_requested {
        *hide_ui_for_save = false;
        *save_requested = false;
        *save_path = None;
        app.set_loop_mode(LoopMode::Wait);
        return; // Skip UI update this frame
    }

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    // Settings window
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
            let mut rng = nannou::rand::rngs::StdRng::seed_from_u64(settings.horz_seed as u64);
            for selector in &mut settings.horz_selectors {
                *selector = rng.gen_bool(0.5);
            }

            let mut rng = nannou::rand::rngs::StdRng::seed_from_u64(settings.vert_seed as u64);
            for selector in &mut settings.vert_selectors {
                *selector = rng.gen_bool(0.5);
            }
        }
    });

    // Save Image window
    egui::Window::new("Save Image").show(&ctx, |ui| {
        if ui.button("Save as PNG").clicked() {
            // Open file dialog immediately
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG", &["png"])
                .set_file_name("hitomezashi_pattern.png")
                .save_file()
            {
                *save_path = Some(path);
                *save_requested = true;
                *hide_ui_for_save = true;
                app.set_loop_mode(LoopMode::RefreshSync);
            }
        }
    });

    // Handle save request
    if *save_requested && *hide_ui_for_save {
        // Save will happen in view function
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Prepare to draw.
    let draw = app.draw();

    // Draw the pattern
    draw_pattern(&draw, app.window_rect(), &model.settings);

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();

    // If we need to save without UI, do it now
    if model.hide_ui_for_save && model.save_requested {
        if let Some(ref path) = model.save_path {
            save_frame_with_metadata(app, &frame, path, &model.settings);
        }
    } else {
        model.egui.draw_to_frame(&frame).unwrap();
    }
}

/// Draw the hitomezashi pattern without any UI elements
fn draw_pattern(draw: &Draw, bounds: Rect, settings: &RectSettings) {
    // Clear the background to white
    draw.background().color(WHITE);

    // Draw the pattern as specified by the settings
    draw_hito_horizontal(draw, bounds, settings.spacing, &settings.horz_selectors);

    draw_hito_vertical(draw, bounds, settings.spacing, &settings.vert_selectors);
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

/// Save the current frame directly with metadata
fn save_frame_with_metadata(
    app: &App,
    _frame: &Frame,
    path: &std::path::Path,
    settings: &RectSettings,
) {
    println!("Saving frame with metadata to: {}", path.display());

    // Since capture_frame is unreliable, use manual PNG creation directly
    match create_image_with_pattern(app, path, settings) {
        Ok(()) => println!("Image saved with metadata to: {}", path.display()),
        Err(e) => eprintln!("Failed to create image: {e}"),
    }
}

/// Create PNG with actual hitomezashi pattern and metadata
fn create_image_with_pattern(
    app: &App,
    path: &std::path::Path,
    settings: &RectSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use png::{BitDepth, ColorType, Encoder};
    use std::fs::File;
    use std::io::BufWriter;

    // Get window dimensions
    let window_rect = app.window_rect();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = window_rect.w() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = window_rect.h() as u32;

    // Create image data with white background
    let mut image_data = Vec::with_capacity((width * height * 4) as usize);

    // Initialize with white background
    for _ in 0..(width * height) {
        image_data.push(255); // R
        image_data.push(255); // G
        image_data.push(255); // B
        image_data.push(255); // A
    }

    // Draw the hitomezashi pattern onto the image data
    draw_pattern_to_image(&mut image_data, width, height, settings);

    // Create PNG with metadata
    let file = File::create(path)?;
    let mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(&mut w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    // Add metadata directly during creation
    let settings_json = serde_json::json!({
        "spacing": settings.spacing,
        "horz_seed": settings.horz_seed,
        "vert_seed": settings.vert_seed,
        "horz_selectors": settings.horz_selectors,
        "vert_selectors": settings.vert_selectors
    });

    encoder.add_text_chunk("Description".to_string(), "Hitomezashi Pattern".to_string())?;
    encoder.add_text_chunk(
        "Software".to_string(),
        "Hitomezashi Pattern Generator".to_string(),
    )?;
    encoder.add_text_chunk("Settings".to_string(), settings_json.to_string())?;

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&image_data)?;
    writer.finish()?;

    Ok(())
}

/// Draw the hitomezashi pattern directly onto image data
fn draw_pattern_to_image(image_data: &mut [u8], width: u32, height: u32, settings: &RectSettings) {
    let spacing = settings.spacing;

    // Draw horizontal lines
    let mut current_y = 0.0;
    let mut selector_idx = 0;

    while current_y < height as f32 {
        let y = current_y as u32;
        if y >= height {
            break;
        }

        let start_with_dash = settings.horz_selectors[selector_idx % settings.horz_selectors.len()];
        selector_idx += 1;

        draw_horizontal_dashed_line(image_data, width, height, y, spacing, start_with_dash);
        current_y += spacing;
    }

    // Draw vertical lines
    let mut current_x = 0.0;
    selector_idx = 0;

    while current_x < width as f32 {
        let x = current_x as u32;
        if x >= width {
            break;
        }

        let start_with_dash = settings.vert_selectors[selector_idx % settings.vert_selectors.len()];
        selector_idx += 1;

        draw_vertical_dashed_line(image_data, width, height, x, spacing, start_with_dash);
        current_x += spacing;
    }
}

fn draw_horizontal_dashed_line(
    image_data: &mut [u8],
    width: u32,
    height: u32,
    y: u32,
    dash_length: f32,
    start_with_dash: bool,
) {
    if y >= height {
        return;
    }

    let mut x = 0.0;
    let mut drawing = start_with_dash;

    while x < width as f32 {
        let end_x = (x + dash_length).min(width as f32);

        if drawing {
            for pixel_x in (x as u32)..(end_x as u32) {
                if pixel_x < width {
                    set_pixel_black(image_data, width, pixel_x, y);
                }
            }
        }

        x = end_x;
        drawing = !drawing;
    }
}

fn draw_vertical_dashed_line(
    image_data: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    dash_length: f32,
    start_with_dash: bool,
) {
    if x >= width {
        return;
    }

    let mut y = 0.0;
    let mut drawing = start_with_dash;

    while y < height as f32 {
        let end_y = (y + dash_length).min(height as f32);

        if drawing {
            for pixel_y in (y as u32)..(end_y as u32) {
                if pixel_y < height {
                    set_pixel_black(image_data, width, x, pixel_y);
                }
            }
        }

        y = end_y;
        drawing = !drawing;
    }
}

fn set_pixel_black(image_data: &mut [u8], width: u32, x: u32, y: u32) {
    let index = ((y * width + x) * 4) as usize;
    if index + 3 < image_data.len() {
        image_data[index] = 0; // R
        image_data[index + 1] = 0; // G
        image_data[index + 2] = 0; // B
                                   // Keep alpha at 255
    }
}

/// Add metadata containing `RectSettings` to the PNG file as text chunks
#[cfg(test)]
fn add_metadata_to_png(
    path: &std::path::Path,
    settings: &RectSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if file exists and is readable
    if !path.exists() {
        return Err(format!("PNG file does not exist: {}", path.display()).into());
    }

    // Additional check for file size to ensure it's fully written
    let file_size = std::fs::metadata(path)?.len();
    if file_size == 0 {
        return Err(format!("PNG file is empty: {}", path.display()).into());
    }

    // Read the original PNG file
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut decoder = png::Decoder::new(reader);
    decoder.set_transformations(png::Transformations::IDENTITY);

    let mut reader = decoder.read_info()?;

    // Read the image data
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;

    // Create new PNG file with metadata
    let temp_path = path.with_extension("tmp.png");

    // Create settings JSON
    let settings_json = serde_json::json!({
        "spacing": settings.spacing,
        "horz_seed": settings.horz_seed,
        "vert_seed": settings.vert_seed,
        "horz_selectors": settings.horz_selectors,
        "vert_selectors": settings.vert_selectors
    });

    {
        let file = File::create(&temp_path)?;
        let mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(&mut w, info.width, info.height);
        encoder.set_color(info.color_type);
        encoder.set_depth(info.bit_depth);

        // Add text chunks with our metadata
        encoder.add_text_chunk("Description".to_string(), "Hitomezashi Pattern".to_string())?;
        encoder.add_text_chunk(
            "Software".to_string(),
            "Hitomezashi Pattern Generator".to_string(),
        )?;
        encoder.add_text_chunk("Settings".to_string(), settings_json.to_string())?;
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&buf)?;
        writer.finish()?;
    } // File is closed here

    // Replace original file with the new one
    std::fs::rename(&temp_path, path)?;
    println!("Added metadata successfully: {settings_json}");
    Ok(())
}

/// Read and display metadata from a PNG file for verification

#[cfg(test)]
mod tests {
    use super::*;
    use png::{BitDepth, ColorType, Encoder};
    use std::fs::File;
    use std::io::{BufWriter, Read};
    use std::path::Path;

    fn create_test_settings() -> RectSettings {
        RectSettings {
            spacing: 25.0,
            horz_seed: 42,
            vert_seed: 84,
            horz_selectors: vec![true, false, true, false, true],
            vert_selectors: vec![false, true, false, true, false],
        }
    }

    fn create_test_png(
        path: &Path,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create a simple test image
        let mut image_data: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let r = if (x / 10) % 2 == 0 { 255 } else { 128 };
                let g = if (y / 10) % 2 == 0 { 255 } else { 128 };
                let b = 200;
                let a = 255;

                image_data.push(r);
                image_data.push(g);
                image_data.push(b);
                image_data.push(a);
            }
        }

        let file = File::create(path)?;
        let ref mut w = BufWriter::new(file);

        let mut encoder = Encoder::new(w, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&image_data)?;
        writer.finish()?;

        Ok(())
    }

    fn read_png_text_chunks(
        path: &Path,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut png_data = Vec::new();
        file.read_to_end(&mut png_data)?;

        let mut pos = 8; // Skip PNG signature
        let mut text_chunks = Vec::new();

        while pos < png_data.len() - 12 {
            if pos + 8 >= png_data.len() {
                break;
            }

            // Read chunk length (4 bytes, big endian)
            let length = u32::from_be_bytes([
                png_data[pos],
                png_data[pos + 1],
                png_data[pos + 2],
                png_data[pos + 3],
            ]);

            // Read chunk type (4 bytes)
            let chunk_type = String::from_utf8_lossy(&png_data[pos + 4..pos + 8]);

            // Check if this is a text chunk
            if chunk_type == "tEXt" {
                let data_start = pos + 8;
                let data_end = data_start + length as usize;

                if data_end <= png_data.len() {
                    let text_data = &png_data[data_start..data_end];

                    // tEXt format: keyword\0text
                    if let Some(null_pos) = text_data.iter().position(|&b| b == 0) {
                        let keyword = String::from_utf8_lossy(&text_data[..null_pos]).to_string();
                        let text = String::from_utf8_lossy(&text_data[null_pos + 1..]).to_string();
                        text_chunks.push((keyword, text));
                    }
                }
            }

            // Move to next chunk
            pos += 12 + length as usize; // 4 bytes length + 4 bytes type + data + 4 bytes CRC
        }

        Ok(text_chunks)
    }

    #[test]
    fn test_metadata_writing_and_reading() {
        let test_path = Path::new("test_metadata.png");
        let settings = create_test_settings();

        // Clean up any existing test file
        let _ = std::fs::remove_file(test_path);

        // Create a test PNG file
        create_test_png(test_path, 100, 100).expect("Failed to create test PNG");

        // Add metadata to the PNG
        add_metadata_to_png(test_path, &settings).expect("Failed to add metadata");

        // Read the metadata back
        let text_chunks = read_png_text_chunks(test_path).expect("Failed to read text chunks");

        // Verify the metadata was written correctly
        assert!(!text_chunks.is_empty(), "No text chunks found");

        let mut found_description = false;
        let mut found_software = false;
        let mut found_settings = false;

        for (keyword, text) in text_chunks {
            match keyword.as_str() {
                "Description" => {
                    assert_eq!(text, "Hitomezashi Pattern");
                    found_description = true;
                }
                "Software" => {
                    assert_eq!(text, "Hitomezashi Pattern Generator");
                    found_software = true;
                }
                "Settings" => {
                    // Parse JSON to verify it's valid
                    let parsed: serde_json::Value =
                        serde_json::from_str(&text).expect("Settings should be valid JSON");

                    assert_eq!(parsed["spacing"], 25.0);
                    assert_eq!(parsed["horz_seed"], 42);
                    assert_eq!(parsed["vert_seed"], 84);
                    assert_eq!(
                        parsed["horz_selectors"],
                        serde_json::json!([true, false, true, false, true])
                    );
                    assert_eq!(
                        parsed["vert_selectors"],
                        serde_json::json!([false, true, false, true, false])
                    );
                    found_settings = true;
                }
                _ => {}
            }
        }

        assert!(found_description, "Description chunk not found");
        assert!(found_software, "Software chunk not found");
        assert!(found_settings, "Settings chunk not found");

        // Clean up
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_metadata_with_different_settings() {
        let test_path = Path::new("test_metadata_different.png");
        let settings = RectSettings {
            spacing: 50.0,
            horz_seed: 123,
            vert_seed: 200,
            horz_selectors: vec![true, true, false, false, true, false],
            vert_selectors: vec![false, false, true, true, false, true],
        };

        // Clean up any existing test file
        let _ = std::fs::remove_file(test_path);

        // Create a test PNG file
        create_test_png(test_path, 200, 150).expect("Failed to create test PNG");

        // Add metadata to the PNG
        add_metadata_to_png(test_path, &settings).expect("Failed to add metadata");

        // Read the metadata back
        let text_chunks = read_png_text_chunks(test_path).expect("Failed to read text chunks");

        // Find the settings chunk
        let settings_chunk = text_chunks
            .iter()
            .find(|(keyword, _)| keyword == "Settings")
            .expect("Settings chunk not found");

        // Parse and verify the settings
        let parsed: serde_json::Value =
            serde_json::from_str(&settings_chunk.1).expect("Settings should be valid JSON");

        assert_eq!(parsed["spacing"], 50.0);
        assert_eq!(parsed["horz_seed"], 123);
        assert_eq!(parsed["vert_seed"], 200);
        assert_eq!(
            parsed["horz_selectors"],
            serde_json::json!([true, true, false, false, true, false])
        );
        assert_eq!(
            parsed["vert_selectors"],
            serde_json::json!([false, false, true, true, false, true])
        );

        // Clean up
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_metadata_nonexistent_file() {
        let nonexistent_path = Path::new("nonexistent.png");
        let settings = create_test_settings();

        // Ensure file doesn't exist
        let _ = std::fs::remove_file(nonexistent_path);

        // Try to add metadata to nonexistent file
        let result = add_metadata_to_png(nonexistent_path, &settings);
        assert!(result.is_err(), "Should fail for nonexistent file");
    }

    #[test]
    fn test_settings_json_serialization() {
        let settings = RectSettings {
            spacing: 37.5,
            horz_seed: 255,
            vert_seed: 0,
            horz_selectors: vec![true, false, true, true, false],
            vert_selectors: vec![false, true, false, false, true],
        };

        let settings_json = serde_json::json!({
            "spacing": settings.spacing,
            "horz_seed": settings.horz_seed,
            "vert_seed": settings.vert_seed,
            "horz_selectors": settings.horz_selectors,
            "vert_selectors": settings.vert_selectors
        });

        let json_string = settings_json.to_string();
        let parsed: serde_json::Value =
            serde_json::from_str(&json_string).expect("Should be able to parse back");

        assert_eq!(parsed["spacing"], 37.5);
        assert_eq!(parsed["horz_seed"], 255);
        assert_eq!(parsed["vert_seed"], 0);
        assert_eq!(
            parsed["horz_selectors"],
            serde_json::json!([true, false, true, true, false])
        );
        assert_eq!(
            parsed["vert_selectors"],
            serde_json::json!([false, true, false, false, true])
        );
    }

    #[test]
    fn test_png_file_replacement() {
        let test_path = Path::new("test_replacement.png");
        let settings = create_test_settings();

        // Clean up any existing test file
        let _ = std::fs::remove_file(test_path);

        // Create a test PNG file
        create_test_png(test_path, 50, 50).expect("Failed to create test PNG");

        // Get original file size
        let original_size = std::fs::metadata(test_path)
            .expect("Failed to get file metadata")
            .len();

        // Add metadata to the PNG
        add_metadata_to_png(test_path, &settings).expect("Failed to add metadata");

        // Get new file size
        let new_size = std::fs::metadata(test_path)
            .expect("Failed to get file metadata")
            .len();

        // File should be larger after adding metadata
        assert!(
            new_size > original_size,
            "File should be larger after adding metadata"
        );

        // Verify file still exists and is readable
        assert!(test_path.exists(), "File should still exist");

        // Clean up
        let _ = std::fs::remove_file(test_path);
    }
}
