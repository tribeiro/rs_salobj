use std::fmt;

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

/// Avro Schema field.
#[derive(Serialize, Deserialize, Debug)]
pub struct AvroSchemaField {
    name: String,
    #[serde(rename = "type")]
    avro_type: String,
    default: String,
    description: String,
    units: String,
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

impl fmt::Display for SalType {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SalType::SalBool(val) => write!(f, "{}", val),
            SalType::SalByte(val) => write!(f, "{}", val),
            SalType::SalShort(val) => write!(f, "{}", val),
            SalType::SalInt(val) => write!(f, "{}", val),
            SalType::SalLong(val) => write!(f, "{}", val),
            SalType::SalLongLong(val) => write!(f, "{}", val),
            SalType::SalUnsignedShort(val) => write!(f, "{}", val),
            SalType::SalUnsignedInt(val) => write!(f, "{}", val),
            SalType::SalUnsignedLong(val) => write!(f, "{}", val),
            SalType::SalFloat(val) => write!(f, "{}", val),
            SalType::SalDouble(val) => write!(f, "{}", val),
            SalType::SalString(val) => write!(f, "{}", val),
        }
    }
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

    pub fn make_avro_schema(&self) -> AvroSchemaField {
        AvroSchemaField {
            name: self.name.to_owned(),
            avro_type: FieldInfo::get_avro_type_from_sal_type(&self.sal_type).to_owned(),
            default: self.default_scalar_value.to_string(),
            description: self.description.to_owned(),
            units: self.units.to_owned(),
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

    /// Return the avro type name from the sal type name.
    fn get_avro_type_from_sal_type(sal_type: &str) -> &str {
        match sal_type {
            "boolean" => "boolean",
            "byte" => "bytes",
            "short" => "int",
            "int" => "int",
            "long" => "long",
            "long long" => "long",
            "unsigned short" => "int",
            "unsigned int" => "int",
            "unsigned long" => "long",
            "float" => "float",
            "double" => "double",
            "string" => "string",
            _ => panic!("Unrecognized SAL type '{sal_type}'"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn make_avro_schema() {
        let field_info = FieldInfo::new("value", "string", 1, "unitless", "Test value.");

        let avro_schema = field_info.make_avro_schema();
        let avro_schema_str = serde_json::to_string(&avro_schema).unwrap();

        assert_eq!(
            avro_schema_str,
            r#"{"name":"value","type":"string","default":"","description":"Test value.","units":"unitless"}"#
        )
    }
}
