use std::borrow::Cow;

use crate::{
    types::{NormalRionType, RionFieldType},
    Result, RionField,
};

#[cfg(test)]
mod test {
    use super::*;
    use crate::RionField;

    fn create_test_table_data() -> Vec<u8> {
        // Create a sample table with 2 columns and 2 rows
        let data = vec![
            0xB1, 0x0C, // Table lead byte and length
            0x21, 0x02, // Number of rows (2)
            0xE2, b'i', b'd', // Column name "id"
            0xE4, b'n', b'a', b'm', b'e', // Column name "name"
            0x21, 0x01, // id: 1
            0x61, b'A', // name: "A"
            0x21, 0x02, // id: 2
            0x61, b'B', // name: "B"
        ];
        data
    }

    #[test]
    fn test_table_from_slice() {
        let data = create_test_table_data();
        let table = RionTable::from_slice(&data).unwrap();

        assert_eq!(table.column_names.len(), 2);
        assert_eq!(table.column_names[0], b"id".as_ref());
        assert_eq!(table.column_names[1], b"name".as_ref());

        assert_eq!(table.rows.len(), 4);
    }

    #[test]
    fn test_table_parse() {
        let data = create_test_table_data();
        let (table, rest) = RionTable::parse(&data).unwrap();

        assert!(rest.is_empty());
        assert_eq!(table.column_names.len(), 2);
        assert_eq!(table.rows.len(), 4);
    }

    #[test]
    fn test_table_parse_with_extra_data() {
        let mut data = create_test_table_data();
        data.extend_from_slice(&[0x00, 0x00]); // Add extra data

        let result = RionTable::from_slice(&data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Extra data after table");
    }

    #[test]
    fn test_table_parse_invalid_lead_byte() {
        let mut data = create_test_table_data();
        data[0] = 0xA0; // Change lead byte to Array instead of Table

        let result = RionTable::parse(&data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Expected a RION table, found Normal(Array)"
        );
    }

    #[test]
    fn test_table_parse_invalid_row_count() {
        let mut data = create_test_table_data();
        data[2] = 0x50; // Change row count to UTF8 instead of Int64Positive

        let result = RionTable::parse(&data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Expected a short field, found Normal(NormalField { field_type: UTF8, length_length: 0, data: [] })");
    }

    #[test]
    fn test_table_parse_zero_rows() {
        let mut data = create_test_table_data();
        data[3] = 0x00; // Change row count to 0

        let (table, rest) = RionTable::parse(&data).unwrap();

        assert_eq!(table.column_names.len(), 2);
        assert_eq!(table.rows.len(), 0);
        assert!(!rest.is_empty());
    }

    #[test]
    fn test_table_parse_no_columns() {
        let data = vec![
            0xB1, 0x02, // Table lead byte and length
            0x21, 0x00, // Number of rows (0)
        ];

        let (table, rest) = RionTable::parse(&data).unwrap();

        assert_eq!(table.column_names.len(), 0);
        assert_eq!(table.rows.len(), 0);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_table_parse_incomplete_data() {
        let data = vec![
            0xB0, 0x0A, // Table lead byte and length
            0x20, 0x02, // Number of rows (2)
            0xD1, b'i', b'd', // Column name "id"
        ];

        let result = RionTable::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_parse_mismatched_row_data() {
        let mut data = create_test_table_data();
        data[3] = 0x03; // Change row count to 3 (but only data for 2 rows)

        let result = RionTable::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_column_data_types() {
        let data = vec![
            0xB1, 0x0E, // Table lead byte and length
            0x21, 0x01, // Number of rows (1)
            0xE1, b'a', // Column name "a"
            0xE1, b'b', // Column name "b"
            0xE1, b'c', // Column name "c"
            0x21, 0x0A, // a: 10 (Int64Positive)
            0x31, 0x0A, // b: -11 (Int64Negative)
            0x61, b'x', // c: "x" (UTF8)
        ];

        let (table, _) = RionTable::parse(&data).unwrap();

        assert_eq!(table.column_names.len(), 3);
        assert_eq!(table.rows.len(), 3);

        if let RionField::Short(field) = &table.rows[0] {
            assert_eq!(field.as_pos_int(), Some(10));
        } else {
            panic!("Expected Short field for column 'a'");
        }

        if let RionField::Short(field) = &table.rows[1] {
            assert_eq!(field.as_neg_int(), Some(-11));
        } else {
            panic!("Expected Short field for column 'b'");
        }

        if let RionField::Short(field) = &table.rows[2] {
            assert_eq!(field.as_str(), Some("x"));
        } else {
            panic!("Expected Short field for column 'c'");
        }
    }

    #[test]
    fn test_table_with_null_values() {
        let data = vec![
            0xB1, 0x09, // Table lead byte and length
            0x21, 0x01, // Number of rows (1)
            0xE1, b'a', // Column name "a"
            0xE1, b'b', // Column name "b"
            0x21, 0x0A, // a: 10
            0x50, // b: null (UTF8 with length 0)
        ];

        let (table, _) = RionTable::parse(&data).unwrap();

        assert_eq!(table.column_names.len(), 2);
        assert_eq!(table.rows.len(), 2);

        if let RionField::Normal(field) = &table.rows[1] {
            assert!(field.is_null());
        } else {
            panic!("Expected Normal field for column 'b'");
        }
    }
}

#[derive(Debug, Clone)]
pub struct RionTable<'a> {
    pub column_names: Vec<Cow<'a, [u8]>>,
    pub rows: Vec<RionField<'a>>, // TODO Make better type
}

impl<'a> RionTable<'a> {
    pub fn from_slice(data: &'a [u8]) -> Result<Self> {
        let (table, rest) = Self::parse(data)?;
        if !rest.is_empty() {
            return Err("Extra data after table".into());
        }
        Ok(table)
    }

    fn parse(data: &'a [u8]) -> Result<(Self, &[u8])> {
        if data.is_empty() {
            return Err("Data is empty".into());
        }
        let (lead, length, rest) = crate::get_normal_header(data)?;
        let RionFieldType::Normal(NormalRionType::Table) = lead.field_type() else {
            return Err(format!("Expected a RION table, found {:?}", lead.field_type()).into());
        };
        // First field is Int64Positive = m = number of rows
        let (field, mut rest) = RionField::parse(rest)?;
        let RionField::Short(short) = field else {
            return Err(format!("Expected a short field, found {:?}", field).into());
        };
        let Some(m) = short.as_pos_int() else {
            return Err(format!("Expected a positive integer, found {:?}", short).into());
        };
        let mut column_names = Vec::new();
        // Next n Key/KeyShorts = Column names
        let first_object = loop {
            let Ok((field, new_rest)) = RionField::parse(rest) else {
                if m != 0 {
                    return Err("Not enough column names".into());
                }
                return Ok((
                    RionTable {
                        column_names,
                        rows: Vec::new(),
                    },
                    rest,
                ));
            };
            rest = new_rest;
            if !field.is_key() {
                break field;
            }
            column_names.push(field.to_data().unwrap());
        };
        println!("first_object: {:?}", first_object);
        if column_names.is_empty() || m == 0 {
            return Ok((
                RionTable {
                    column_names,
                    rows: Vec::new(),
                },
                rest,
            ));
        }

        // next m * n fields = data
        let data_len = m * column_names.len() as u64;
        if data_len > length as u64 {
            return Err(format!(
                "Not enough data for rows, expected {}, found {}",
                data_len, length
            )
            .into());
        }
        let mut rows = Vec::with_capacity((data_len) as usize);
        rows.push(first_object);
        for _ in 0..data_len - (!column_names.is_empty() as u64) {
            let (field, new_rest) = RionField::parse(rest)?;
            rest = new_rest;
            rows.push(field);
        }

        Ok((RionTable { column_names, rows }, rest))
    }
}
