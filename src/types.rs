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

    pub fn byte(self) -> u8 {
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
    pub fn to_byte(self) -> u8 {
        match self {
            ShortRionType::Int64Positive => 0x2,
            ShortRionType::Int64Negative => 0x3,
            ShortRionType::Float => 0x4,
            ShortRionType::UTF8 => 0x6,
            ShortRionType::UTCDateTime => 0x7,
            ShortRionType::Key => 0xE,
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
    pub fn to_byte(self) -> u8 {
        match self {
            NormalRionType::Bytes => 0x0,
            NormalRionType::UTF8 => 0x5,
            NormalRionType::Array => 0xA,
            NormalRionType::Table => 0xB,
            NormalRionType::Object => 0xC,
            NormalRionType::Key => 0xD,
        }
    }
}

impl RionFieldType {
    pub fn to_byte(self) -> u8 {
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
