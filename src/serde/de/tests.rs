use super::*;
#[test]
fn test_deserialize_uint() {
    let data = vec![0x21, 0x0A]; // 10
    let value: u64 = from_bytes(&data).unwrap();
    assert_eq!(value, 10);
}

#[test]
fn test_deserialize_string() {
    let data = vec![0xD1, 0x05, b'A', b'l', b'i', b'c', b'e'];
    let name: String = from_bytes(&data).unwrap();
    assert_eq!(name, "Alice");
}

#[test]
fn test_deserialize_map() {
    let data = vec![
        0xC1, 0x0A, 0xE3, b'K', b'e', b'y', 0x65, b'V', b'a', b'l', b'u', b'e',
    ];
    let map: std::collections::HashMap<String, String> = from_bytes(&data).unwrap();
    println!("{:?}", map);
    assert_eq!(map.get("Key").unwrap(), "Value");
}

#[test]
fn test_deserialize_integers() {
    let data = vec![0x21, 0x7F]; // 127 (i8 max)
    let value: u8 = from_bytes(&data).unwrap();
    assert_eq!(value, 127);

    let data = vec![0x31, 0x7F]; // -128 (i8 min)
    let value: i8 = from_bytes(&data).unwrap();
    assert_eq!(value, i8::MIN);

    let data = vec![0x22, 0x7F, 0xFF]; // 32767 (i16 max)
    let value: i16 = from_bytes(&data).unwrap();
    assert_eq!(value, 32767);

    let data = vec![0x24, 0x7F, 0xFF, 0xFF, 0xFF]; // 2147483647 (i32 max)
    let value: i32 = from_bytes(&data).unwrap();
    assert_eq!(value, 2147483647);
}

#[test]
fn test_deserialize_float() {
    let data = vec![0x44, 0x40, 0x48, 0xF5, 0xC3]; // 3.14 (f32)
    let value: f32 = from_bytes(&data).unwrap();
    assert!((value - 3.14).abs() < f32::EPSILON);

    let data = vec![0x48, 0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x11]; // 3.14159265358979 (f64)
    let value: f64 = from_bytes(&data).unwrap();
    assert!((value - 3.14159265358979).abs() < f64::EPSILON);
}

#[test]
fn test_deserialize_char() {
    let data = vec![0x61, b'A'];
    let value: char = from_bytes(&data).unwrap();
    assert_eq!(value, 'A');
}

#[test]
fn test_deserialize_option() {
    let data = vec![0x00]; // null
    let value: Option<u32> = from_bytes(&data).unwrap();
    assert_eq!(value, None);

    let data = vec![0x21, 0x0A]; // Some(10)
    let value: Option<u32> = from_bytes(&data).unwrap();
    assert_eq!(value, Some(10));
}

use serde::Deserialize;
#[derive(Deserialize)]
struct Test {
    name: String,
    #[allow(dead_code)]
    age: u32,
}

#[test]
fn test_deserialize_struct() {
    let data = vec![
        0xC1, 0x11, // Start of object
        0xE4, b'n', b'a', b'm', b'e', 0x65, b'A', b'l', b'i', b'c', b'e', // name: "Alice"
        0xE3, b'a', b'g', b'e', 0x21, 0x1E, // age: 30
    ];
    let value: Test = from_bytes(&data).unwrap();
    assert_eq!(value.name, "Alice");
}

// Nested structs
#[derive(Deserialize, Debug, PartialEq)]
struct Address {
    street: String,
    city: String,
}

#[derive(Deserialize, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
    address: Address,
}

#[test]
fn test_deserialize_nested_struct() {
    let data = vec![
        0xC1, 0x35, // Start of object
        0xE4, b'n', b'a', b'm', b'e', 0x65, b'A', b'l', b'i', b'c', b'e', // name: "Alice"
        0xE3, b'a', b'g', b'e', 0x21, 0x1E, // age: 30
        0xE7, b'a', b'd', b'd', b'r', b'e', b's', b's', 0xC1, 0x1A, // address: { ... }
        0xE6, b's', b't', b'r', b'e', b'e', b't', 0x68, b'1', b'2', b'3', b' ', b'M', b'a', b'i',
        b'n', // street: "123 Main"
        0xE4, b'c', b'i', b't', b'y', 0x64, b'S', b'o', b'm', b'e', // city: "Some"
    ];
    println!("{:?}", data.len());
    let value: User = from_bytes(&data).unwrap();
    assert_eq!(value.name, "Alice");
    assert_eq!(value.age, 30);
    assert_eq!(value.address.street, "123 Main");
    assert_eq!(value.address.city, "Some");
}

#[test]
fn test_deserialize_tuple() {
    let data = vec![0xA1, 0x04, 0x21, 0x0A, 0x61, b'A']; // (10, 'A')
    let value: (u8, char) = from_bytes(&data).unwrap();
    assert_eq!(value, (10, 'A'));
}

#[test]
fn test_deserialize_bytes() {
    let data = vec![
        // 0xA1, 0x0A, 0x21, 0x01, 0x21, 0x02, 0x21, 0x03, 0x21, 0x04, 0x21, 0x05
        0x01, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05,
    ];
    let value: Vec<u8> = from_bytes(&data).unwrap();
    assert_eq!(value, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_deserialize_null_option() {
    let data = vec![0x50];
    let result: Option<String> = from_bytes(&data).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_deserialize_some_option() {
    let data = vec![0xE5, b'A', b'l', b'i', b'c', b'e'];
    let result: Option<String> = from_bytes(&data).unwrap();
    assert_eq!(result, Some("Alice".to_string()));
}

#[test]
fn test_deserialize_wrong_option() {
    let data = vec![0xE5, b'A', b'l', b'i', b'c', b'e'];
    let result: Result<Option<i32>, _> = from_bytes(&data);
    assert!(result.is_err())
}
