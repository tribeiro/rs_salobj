extern crate serde;

#[derive(Deserialize, Debug)]
pub enum SalType {
    SalBool(bool),
    SalByte(u8),
    SalShort(i16),
    SalInt(isize),
    SalLong(i32),
    SalLongLong(i64),
    SalUnsignedShort(u16),
    SalUnsignedInt(usize),
    SalUnsignedLong(u32),
    SalFloat(f32),
    SalDouble(f64),
    SalString(String),
}

/// Information about one field of a topic.
#[derive(Deserialize, Debug)]
pub struct FieldInfo {
    /// Field name
    name: String,
    /// SAL data type.
    sal_type: String,
    /// For lists: the fixed list length.
    count: usize,
    /// Units, "unitless" if none.
    units: String,
    /// Description (arbitrary text)
    description: String,
    /// For a scalar: the default value.
    /// For an array: the default value of one element.
    default_scalar_value: SalType,
}

impl FieldInfo {

    /// Create new FieldInfo.
    pub fn new(
        name: &str,
        sal_type: &str,
        count: usize,
        units: &str,
        description: &str,
    ) -> FieldInfo {
        FieldInfo {
            name: String::from(name),
            sal_type: String::from(sal_type),
            count: count,
            units: String::from(units),
            description: String::from(description),
            default_scalar_value: FieldInfo::get_default_scalar_value(sal_type),
        }
    }

    /// Return the default value for the specified sal type.
    fn get_default_scalar_value(sal_type: &str) -> SalType {
        match sal_type {
            "boolean" => SalType::SalBool(false),
            "byte" => SalType::SalByte(0),
            "short" => SalType::SalShort(0),
            "int" => SalType::SalInt(0),
            "long" => SalType::SalLong(0),
            "long long" => SalType::SalLongLong(0),
            "unsigned short" => SalType::SalUnsignedShort(0),
            "unsigned int" => SalType::SalUnsignedInt(0),
            "unsigned long" => SalType::SalUnsignedLong(0),
            "float" => SalType::SalFloat(0.0),
            "double" => SalType::SalDouble(0.0),
            "string" => SalType::SalString(String::from("")),
            _ => panic!("Unrecognized SAL type '{sal_type}'"),
        }
    }
}
