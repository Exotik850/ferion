#![cfg_attr(feature = "specialization", feature(min_specialization))]

use std::error::Error;
mod array;
mod field;
mod object;
mod table;
mod types;

#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "serde")]
pub use serde::*;

pub use array::RionArray;
pub use object::RionObject;
pub use table::RionTable;

#[cfg(test)]
mod test;
pub use field::RionField;
use types::LeadByte;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn get_lead_byte(data: &[u8]) -> Result<(LeadByte, &[u8])> {
    let Some(lead) = data.first() else {
        return Err("Data is empty".into());
    };
    Ok((LeadByte::try_from(*lead)?, &data[1..]))
}

// Get the header of a RION object
fn get_header(data: &[u8]) -> Result<(LeadByte, &[u8], &[u8])> {
    let (lead, rest) = get_lead_byte(data)?;
    let length_length = lead.length() as usize;
    if length_length > rest.len() {
        return Err(
            format!("Not enough data in {rest:x?} for length_length {length_length}").into(),
        );
    }
    Ok((lead, &rest[..length_length], &rest[length_length..]))
}

fn bytes_to_int(bytes: &[u8]) -> Result<u64> {
    match bytes.len() {
        0..=8 => Ok(bytes.iter().fold(0u64, |acc, &b| acc << 8 | b as u64)),
        _ => Err("Too many bytes to convert to u64".into()),
    }
}

// fn bytes_to_float

// Casts the int to a slice of integers (big endian)
// If the int is 0, nothing is written
fn int_to_bytes(int: &u64, w: &mut impl std::io::Write) -> std::io::Result<()> {
    if *int == 0 {
        return Ok(());
    }
    let bytes = int.to_be_bytes();
    let first_non_zero = bytes.iter().position(|&b| b != 0).unwrap();
    w.write_all(&bytes[first_non_zero..])
}

/// Get the header of a RION object
/// Returns the lead byte, the length of the data, and the remaining data
fn get_normal_header(data: &[u8]) -> Result<(LeadByte, usize, &[u8])> {
    let (lead, length, rest) = get_header(data)?;
    let types::RionFieldType::Normal(_) = lead.field_type() else {
        return Err("Expected a Normal encoded field".into());
    };
    let data_len = bytes_to_int(length)?;
    let data_len: usize = data_len.try_into()?;
    if data_len > rest.len() {
        return Err(format!(
            "Not enough data in {data:x?} (len: (rest) {} + (header) {}) for length {data_len}",
            rest.len(),
            1 + length.len()
        )
        .into());
    }
    Ok((lead, data_len, rest))
}

fn needed_bytes(length: u64) -> u32 {
    if length == 0 {
        return 0;
    }
    if length == 1 {
        return 1;
    }
    length.ilog2() / 8 + 1
}

fn needed_bytes_usize(length: usize) -> usize {
    needed_bytes(length as u64) as usize
}

#[cfg(test)]
mod int_cast_tests {
    use crate::needed_bytes;

    // Test the bytes_to_int and int_to_bytes functions
    #[test]
    fn test_bytes_to_int() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        assert_eq!(super::bytes_to_int(&bytes).unwrap(), 0x01020304);
    }

    #[test]
    fn test_int_to_bytes() {
        let int = 0x01020304;
        let mut encoder = Vec::new();
        super::int_to_bytes(&int, &mut encoder).unwrap();
        assert_eq!(&encoder, &[0x01, 0x02, 0x03, 0x04]);
    }

    // Test they work to and from each other
    #[test]
    fn test_int_to_bytes_to_int() {
        let int = 0x01020304;
        let mut encoder = Vec::new();
        super::int_to_bytes(&int, &mut encoder).unwrap();
        assert_eq!(super::bytes_to_int(&encoder).unwrap(), int);
    }

    // Test that the int_to_bytes function writes exactly needed bytes amount of bytes
    #[test]
    fn test_int_to_bytes_needed_bytes() {
        let int = 0x01020304;
        let mut encoder = Vec::new();
        super::int_to_bytes(&int, &mut encoder).unwrap();
        assert_eq!(encoder.len(), needed_bytes(int) as usize);
    }
}
