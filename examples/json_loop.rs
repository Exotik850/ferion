use ferion::{from_bytes, to_bytes};
use flate2::{write::ZlibEncoder, Compression};
use std::{io::Write, time::Instant};

fn main() {
    let mut input = String::new();
    loop {
        input.clear();
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
        let json: serde_json::Value = match serde_json::from_str(input.trim()) {
            Ok(json) => json,
            Err(e) => {
                println!("Invalid JSON: {}", e);
                continue;
            }
        };

        // Measure RION serialization and deserialization
        let rion_ser_start = Instant::now();
        let rion_bytes = match to_bytes(&json) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Failed to convert JSON to RION bytes: {}", e);
                continue;
            }
        };
        let rion_ser_time = rion_ser_start.elapsed();

        let rion_de_start = Instant::now();
        let rion_decoded: serde_json::Value = match from_bytes(&rion_bytes) {
            Ok(decoded) => decoded,
            Err(e) => {
                println!("Failed to decode RION bytes: {}", e);
                continue;
            }
        };
        if rion_decoded != json {
            println!("Warning: Decoded RION does not match the original JSON");
            println!("  RION: {:?}", rion_decoded);
            println!("  JSON: {:?}", json);
            continue;
        }
        let rion_de_time = rion_de_start.elapsed();

        // Measure JSON serialization and deserialization
        let json_ser_start = Instant::now();
        let json_string = serde_json::to_string(&json).unwrap();
        let json_ser_time = json_ser_start.elapsed();

        let json_de_start = Instant::now();
        let _: serde_json::Value = serde_json::from_str(&json_string).unwrap();
        let json_de_time = json_de_start.elapsed();

        // Measure POT serialization and deserialization
        let pot_ser_start = Instant::now();
        let pot_bytes = pot::to_vec(&json).unwrap();
        let pot_ser_time = pot_ser_start.elapsed();

        let pot_de_start = Instant::now();
        let _: serde_json::Value = pot::from_slice(&pot_bytes).unwrap();
        let pot_de_time = pot_de_start.elapsed();

        let json_byte_len = json_string.len();
        let rion_byte_len = rion_bytes.len();
        let pot_byte_len = pot_bytes.len();

        println!("Byte lengths:");
        println!("  RION: {}", rion_byte_len);
        println!("  JSON: {}", json_byte_len);
        println!("  POT:  {}", pot_byte_len);

        println!("\nSerialization times:");
        println!("  RION: {:?}", rion_ser_time);
        println!("  JSON: {:?}", json_ser_time);
        println!("  POT:  {:?}", pot_ser_time);

        println!("\nDeserialization times:");
        println!("  RION: {:?}", rion_de_time);
        println!("  JSON: {:?}", json_de_time);
        println!("  POT:  {:?}", pot_de_time);

        println!("\nRatios (compared to JSON):");
        println!("  RION: {:.2}", rion_byte_len as f64 / json_byte_len as f64);
        println!("  POT:  {:.2}", pot_byte_len as f64 / json_byte_len as f64);

        let mut zlib_json = ZlibEncoder::new(Vec::new(), Compression::best());
        zlib_json.write_all(json_string.as_bytes()).unwrap();
        let zlib_json = zlib_json.finish().unwrap();

        let mut zlib_rion = ZlibEncoder::new(Vec::new(), Compression::best());
        zlib_rion.write_all(&rion_bytes).unwrap();
        let zlib_rion = zlib_rion.finish().unwrap();

        let mut zlib_pot = ZlibEncoder::new(Vec::new(), Compression::best());
        zlib_pot.write_all(&pot_bytes).unwrap();
        let zlib_pot = zlib_pot.finish().unwrap();

        println!("\nZlib compressed byte lengths:");
        println!("  JSON: {}", zlib_json.len());
        println!("  RION: {}", zlib_rion.len());
        println!("  POT:  {}", zlib_pot.len());
    }
}
