use std::{error::Error, ffi::OsStr, path::Path};

use ferion::*;
use serde_json::Value;
fn main() -> Result<(), Box<dyn Error>> {
    // Create a sample RION object
    let mut input = String::new();

    loop {
        println!("Enter a JSON object to be shown as RION (or 'exit' to quit):");
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|_| "Failed to read line")?;
        // let trim_inp = input.trim();
        if input.trim().eq_ignore_ascii_case("exit") {
            break;
        }

        let path = Path::new(input.trim());
        println!("{:?}", path.extension().and_then(OsStr::to_str));
        let is_json = path.extension().and_then(OsStr::to_str) == Some("json");
        if is_json {
            println!("This is json file");
        }
        if is_json && path.try_exists().unwrap() {
            println!("Found file!");
            let contents = std::fs::read_to_string(path).map_err(|_| "Failed to read file")?;
            input = contents;
        }
        let input_len = input.len();
        println!(
            "Input (len {input_len}) : {}{}",
            input.lines().take(10).collect::<Vec<_>>().join("\n"),
            if input.lines().count() > 10 {
                "\n..."
            } else {
                ""
            }
        );
        // Convert the input JSON string to RION
        let value: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => {
                println!("Invalid JSON. Please try again.");
                input.clear();
                continue;
            }
        };

        let rion_bytes = match crate::to_bytes(&value) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Failed to convert to RION {e}. Please try again.");
                input.clear();
                continue;
            }
        };

        // Display the RION bytes
        println!("RION bytes: {:x?}", &rion_bytes);
        let decoded_value: Value = crate::from_bytes(&rion_bytes)
            .map_err(|e| format!("Failed to convert back from RION: {e}"))?;
        println!("Decoded!");
        assert_eq!(value, decoded_value);
        input.clear();
    }

    Ok(())
}
