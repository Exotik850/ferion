use std::error::Error;
mod field;
mod object;
mod types;

pub use object::RionObject;

#[cfg(test)]
mod test;
pub use field::RionField;
use field::{LeadByte};
use num_bigint::BigUint;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn get_lead_byte(data: &[u8]) -> Result<(LeadByte, &[u8])> {
    let Some(lead) = data.first() else {
        return Err("Data is empty".into());
    };
    Ok((LeadByte::try_from(*lead)?, &data[1..]))
}

/// Get the header of a RION object
/// Returns the lead byte, the length of the data, and the remaining data
fn get_normal_header(data: &[u8]) -> Result<(LeadByte, usize, &[u8])> {
    let (lead, rest) = get_lead_byte(data)?;
    let length_length = lead.length() as usize;
    let length = BigUint::from_bytes_be(&rest[..length_length]);
    let data_len: usize = length
        .try_into()
        .map_err(|_| "Data too large for this system!")?;
    Ok((lead, data_len, &rest[length_length..]))
}

