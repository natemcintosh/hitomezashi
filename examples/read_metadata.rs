use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <png_file>", args[0]);
        std::process::exit(1);
    }

    let png_path = &args[1];
    read_png_metadata(png_path)?;

    Ok(())
}

fn read_png_metadata(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut png_data = Vec::new();
    file.read_to_end(&mut png_data)?;

    println!("PNG File: {}", path);

    // Parse PNG chunks manually to find text chunks
    let mut pos = 8; // Skip PNG signature
    let mut found_text_chunks = false;

    println!("\nSearching for text chunks...");

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
        if chunk_type == "tEXt" || chunk_type == "zTXt" || chunk_type == "iTXt" {
            found_text_chunks = true;

            // Extract text data
            let data_start = pos + 8;
            let data_end = data_start + length as usize;

            if data_end <= png_data.len() {
                let text_data = &png_data[data_start..data_end];

                if chunk_type == "tEXt" {
                    // tEXt format: keyword\0text
                    if let Some(null_pos) = text_data.iter().position(|&b| b == 0) {
                        let keyword = String::from_utf8_lossy(&text_data[..null_pos]);
                        let text = String::from_utf8_lossy(&text_data[null_pos + 1..]);
                        println!("  {}: {}", keyword, text);
                    }
                }
            }
        }

        // Move to next chunk
        pos += 12 + length as usize; // 4 bytes length + 4 bytes type + data + 4 bytes CRC
    }

    if !found_text_chunks {
        println!("  No text chunks found in PNG file");
    }

    Ok(())
}
