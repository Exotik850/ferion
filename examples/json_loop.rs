use ferion::{from_bytes, to_bytes};

fn main() {
    let mut input = String::new();

    loop {
        input.clear(); // Clear the input for the next iteration
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
                continue;
            };
            input = content;
        }

        let json = match serde_json::from_str::<serde_json::Value>(input.trim()) {
            Ok(json) => json,
            Err(e) => {
                println!("Invalid JSON: {}", e);
                continue;
            }
        };

        let bytes = match to_bytes(&json) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Failed to convert JSON to bytes: {}", e);
                continue;
            }
        };
        // println!("Converted bytes (len: {}): [", bytes.len());
        // for byte in &bytes {
        //     print!("{:02x} ", byte);
        // }
        // println!("]");
        let decoded: serde_json::Value = match from_bytes(&bytes) {
            Ok(decoded) => decoded,
            Err(e) => {
                println!("Failed to decode bytes: {}", e);
                continue;
            }
        };
        println!("Decoded JSON: {:?}", decoded);
        if decoded != json {
            println!("Warning: Decoded JSON does not match the original");
            continue;
        }
        let json_byte_len = input.trim().len();
        let rion_byte_len = bytes.len();
        let ratio = rion_byte_len as f64 / json_byte_len as f64;
        println!(
            "RION byte length: {}, JSON byte length: {}, Ratio: {:.2}",
            rion_byte_len, json_byte_len, ratio
        );
    }
}
