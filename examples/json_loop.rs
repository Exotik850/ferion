use ferion::to_bytes;

fn main() {
    let mut input = String::new();

    loop {
        println!("Please enter a JSON object (or 'exit' to quit):");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimed = input.trim();
        if trimed.eq_ignore_ascii_case("exit") {
            break;
        }

        let json = match serde_json::from_str::<serde_json::Value>(trimed) {
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
    }

    input.clear(); // Clear the input for the next iteration
}
