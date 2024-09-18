use ferion::{from_bytes, to_bytes};

fn main() {
    let mut input = String::new();

    loop {
        println!("Please enter a JSON object (or 'exit' to quit):");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if input.trim().eq_ignore_ascii_case("exit") {
            break;
        }

        if let Some(path) = input.strip_prefix("file:") {
            let path = path.trim();
            let Ok(content) = std::fs::read_to_string(path) else {
                println!("Failed to read file: {}", path);
                input.clear(); // Clear the input for the next iteration
                continue;
            };
            input = content;
        }

        let json = match serde_json::from_str::<serde_json::Value>(input.trim()) {
            Ok(json) => json,
            Err(e) => {
                println!("Invalid JSON: {}", e);
                input.clear(); // Clear the input for the next iteration
                continue;
            }
        };

        let bytes = match to_bytes(&json) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Failed to convert JSON to bytes: {}", e);
                input.clear(); // Clear the input for the next iteration
                continue;
            }
        };
        println!("Converted bytes: {:?}", bytes);
        let decoded: serde_json::Value = match from_bytes(&bytes) {
            Ok(decoded) => decoded,
            Err(e) => {
                println!("Failed to decode bytes: {}", e);
                input.clear(); // Clear the input for the next iteration
                continue;
            }
        };
        println!("Decoded JSON: {:?}", decoded);
        if decoded != json {
            println!("Warning: Decoded JSON does not match the original");
        }
    }

    input.clear(); // Clear the input for the next iteration
}
