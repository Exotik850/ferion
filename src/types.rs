use std::error::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LeadByte(pub(crate) u8); // (field type, length)

impl LeadByte {
    pub fn from_type(field_type: RionFieldType, length: u8) -> Self {
        LeadByte(field_type.to_byte() | length)
    }

    pub fn field_type(self) -> RionFieldType {
        RionFieldType::try_from(self.0).unwrap()
    }

    pub fn length(self) -> u8 {
        self.0 & 0x0F
    }

    pub fn is_null(self) -> bool {
        self.length() == 0
    }

    pub fn is_short(self) -> bool {
        self.length() < 15
    }

    pub fn byte(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for LeadByte {
    type Error = Box<dyn Error>;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        RionFieldType::try_from(value)?;
        Ok(LeadByte(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RionFieldType {
    Short(ShortRionType),
    Normal(NormalRionType),
    Extended,
    Tiny(LeadByte),
}

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum ShortRionType {
    Int64Positive,
    Int64Negative,
    UTF8,
    UTCDateTime,
    Float,
    Key,
}

impl ShortRionType {
    pub fn to_byte(self) -> u8 {
        match self {
            ShortRionType::Int64Positive => 0x20,
            ShortRionType::Int64Negative => 0x30,
            ShortRionType::UTF8 => 0x50,
            ShortRionType::UTCDateTime => 0x70,
            ShortRionType::Float => 0x40,
            ShortRionType::Key => 0xD0,
        }
    }
}

impl TryFrom<u8> for ShortRionType {
    type Error = &'static str;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let out = match value {
            0x20 => ShortRionType::Int64Positive,
            0x30 => ShortRionType::Int64Negative,
            0x50 => ShortRionType::UTF8,
            0x70 => ShortRionType::UTCDateTime,
            0x40 => ShortRionType::Float,
            0xD0 => ShortRionType::Key,
            _ => return Err("Invalid short field type"),
        };
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum NormalRionType {
    Bytes,
    UTF8,
    Array,
    Table,
    Object,
    Key,
}

impl TryFrom<u8> for NormalRionType {
    type Error = &'static str;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let out = match value & 0xF0 {
            0x00 => NormalRionType::Bytes,
            0x50 => NormalRionType::UTF8,
            0xA0 => NormalRionType::Array,
            0xB0 => NormalRionType::Table,
            0xC0 => NormalRionType::Object,
            0xD0 => NormalRionType::Key,
            _ => return Err("Invalid normal field type"),
        };
        Ok(out)
    }
}

impl NormalRionType {
    pub fn to_byte(self) -> u8 {
        match self {
            NormalRionType::Bytes => 0x00,
            NormalRionType::UTF8 => 0x50,
            NormalRionType::Array => 0xA0,
            NormalRionType::Table => 0xB0,
            NormalRionType::Object => 0xC0,
            NormalRionType::Key => 0xD0,
        }
    }
}

impl RionFieldType {
    pub fn to_byte(self) -> u8 {
        match self {
            Self::Short(short) => short.to_byte(),
            Self::Normal(normal) => normal.to_byte(),
            Self::Extended => 0xF0,
            Self::Tiny(lead) => lead.byte(),
        }
    }
}

impl TryFrom<u8> for RionFieldType {
    type Error = &'static str;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let type_bits = value & 0xF0;
        match type_bits {
            0xF0 => Ok(RionFieldType::Extended),
            0x10 => Ok(RionFieldType::Tiny(LeadByte(value))),
            0x80..=0xC0 => Ok(RionFieldType::Normal(NormalRionType::try_from(type_bits)?)),
            0x00..=0x70 | 0xD0 => Ok(RionFieldType::Short(ShortRionType::try_from(type_bits)?)),
            _ => Err("Invalid field type"),
        }
    }
}
