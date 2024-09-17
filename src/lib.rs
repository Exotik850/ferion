#![feature(min_specialization)]

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
use num_bigint::BigUint;
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

fn bytes_to_num<T>(bytes: &[u8]) -> Result<T>
where
    T: TryFrom<BigUint>,
{
    let length = BigUint::from_bytes_be(bytes);
    let data_len: T = length
        .try_into()
        .map_err(|_| format!("Data too large for {}", std::any::type_name::<T>()))?;
    Ok(data_len)
}

/// Get the header of a RION object
/// Returns the lead byte, the length of the data, and the remaining data
fn get_normal_header(data: &[u8]) -> Result<(LeadByte, usize, &[u8])> {
    let (lead, length, rest) = get_header(data)?;
    let types::RionFieldType::Normal(_) = lead.field_type() else {
        return Err("Expected a Normal encoded field".into());
    };
    let data_len: usize = bytes_to_num(length)?;
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

fn num_needed_length(length: usize) -> usize {
    if length == 0 {
        return 0;
    }
    if length == 1 {
        return 1;
    }
    (length.ilog2() as usize).div_ceil(8)
}
