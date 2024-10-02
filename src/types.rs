use std::error::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LeadByte(pub(crate) u8); // (field type, length)

impl LeadByte {
    pub fn from_type(field_type: RionFieldType, length: u8) -> Self {
        LeadByte(field_type.to_byte() << 4 | length)
    }

    pub fn field_type(self) -> RionFieldType {
        RionFieldType::try_from(self.0).unwrap()
    }

    pub fn length(self) -> u8 {
        match self.field_type() {
            RionFieldType::Tiny(_) => 0,
            _ => self.0 & 0x0F,
        }
    }

    pub fn is_null(self) -> bool {
        match self.field_type() {
            RionFieldType::Tiny(lead) => lead.byte() & 0x0F == 0,
            _ => self.length() == 0,
        }
    }

    pub fn is_short(self) -> bool {
        self.length() < 15
    }

    pub const fn byte(self) -> u8 {
        self.0
    }

    pub fn as_bool(self) -> Option<bool> {
        match self.field_type() {
            RionFieldType::Tiny(lead) if lead.byte() & 0x0F != 0 => Some(lead.byte() & 0x0F == 2),
            _ => None,
        }
    }
}

impl TryFrom<u8> for LeadByte {
    type Error = Box<dyn Error>;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        RionFieldType::try_from(value)?;
        Ok(LeadByte(value))
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RionFieldType {
    Short(ShortRionType),
    Normal(NormalRionType),
    Extended,
    Tiny(LeadByte),
}

impl From<ShortRionType> for RionFieldType {
    fn from(value: ShortRionType) -> Self {
        RionFieldType::Short(value)
    }
}

impl From<NormalRionType> for RionFieldType {
    fn from(value: NormalRionType) -> Self {
        RionFieldType::Normal(value)
    }
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
    pub const fn to_byte(self) -> u8 {
        match self {
            ShortRionType::Int64Positive => RionFieldType::INT64_POSITIVE,
            ShortRionType::Int64Negative => RionFieldType::INT64_NEGATIVE,
            ShortRionType::Float => RionFieldType::FLOAT,
            ShortRionType::UTF8 => RionFieldType::UTF8_SHORT,
            ShortRionType::UTCDateTime => RionFieldType::UTC_DATE_TIME,
            ShortRionType::Key => RionFieldType::KEY_SHORT,
        }
    }
}

impl TryFrom<u8> for ShortRionType {
    type Error = Box<dyn Error>;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let out = match (value & 0xF0) >> 4 {
            0x2 => ShortRionType::Int64Positive,
            0x3 => ShortRionType::Int64Negative,
            0x4 => ShortRionType::Float,
            0x6 => ShortRionType::UTF8,
            0x7 => ShortRionType::UTCDateTime,
            0xE => ShortRionType::Key,
            _ => return Err(format!("Invalid short field type: {value:#X}").into()),
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
    type Error = Box<dyn Error>;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let out = match (value & 0xF0) >> 4 {
            0x0 => NormalRionType::Bytes,
            0x5 => NormalRionType::UTF8,
            0xA => NormalRionType::Array,
            0xB => NormalRionType::Table,
            0xC => NormalRionType::Object,
            0xD => NormalRionType::Key,
            _ => return Err(format!("Invalid normal field type: {value:#X}").into()),
        };
        Ok(out)
    }
}

impl NormalRionType {
    pub const fn to_byte(self) -> u8 {
        match self {
            NormalRionType::Bytes => RionFieldType::BYTES,
            NormalRionType::UTF8 => RionFieldType::UTF8,
            NormalRionType::Array => RionFieldType::ARRAY,
            NormalRionType::Table => RionFieldType::TABLE,
            NormalRionType::Object => RionFieldType::OBJECT,
            NormalRionType::Key => RionFieldType::KEY,
        }
    }
}

impl RionFieldType {
    pub const OBJECT: u8 = 0xC;
    pub const TABLE: u8 = 0xB;
    pub const ARRAY: u8 = 0xA;
    pub const UTF8: u8 = 0x5;
    pub const BYTES: u8 = 0x0;
    pub const KEY: u8 = 0xD;
    pub const FLOAT: u8 = 0x4;
    pub const UTC_DATE_TIME: u8 = 0x7;
    pub const INT64_POSITIVE: u8 = 0x2;
    pub const INT64_NEGATIVE: u8 = 0x3;
    pub const UTF8_SHORT: u8 = 0x6;
    pub const KEY_SHORT: u8 = 0xE;

    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Short(short) => short.to_byte(),
            Self::Normal(normal) => normal.to_byte(),
            Self::Extended => 0xF,
            Self::Tiny(lead) => lead.byte(),
        }
    }

    pub fn is_key(&self) -> bool {
        matches!(
            self,
            RionFieldType::Short(ShortRionType::Key) | RionFieldType::Normal(NormalRionType::Key)
        )
    }
}

impl TryFrom<u8> for RionFieldType {
    type Error = Box<dyn Error>;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let type_bits = value & 0xF0;
        match type_bits >> 4 {
            0xF => Ok(RionFieldType::Extended),
            0x1 => Ok(RionFieldType::Tiny(LeadByte(value))),
            0x0 | 0x5 | 0xA..=0xD => {
                Ok(RionFieldType::Normal(NormalRionType::try_from(type_bits)?))
            }
            0x0..=0x7 | 0xE => Ok(RionFieldType::Short(ShortRionType::try_from(type_bits)?)),
            _ => Err(format!("Invalid field type: {value:#X}").into()),
        }
    }
}
