use std::{
    collections::BTreeMap,
    fmt::{self, Debug},
};

use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

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
#[derive(Deserialize, Debug)]
pub struct AvroSchemaField {
    name: String,
    // #[serde(rename = "type")]
    avro_type: BTreeMap<String, String>,
    default: Vec<String>,
    description: String,
    units: String,
    // #[serde(skip)]
    scalar: bool,
}

impl Serialize for AvroSchemaField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut avro_schema_field = serializer.serialize_struct("AvroSchemaField", 5)?;
        avro_schema_field.serialize_field("name", &self.name)?;
        if self.scalar {
            avro_schema_field.serialize_field("type", &self.avro_type.get("items"))?;
            avro_schema_field.serialize_field("default", &self.default[0])?;
        } else {
            avro_schema_field.serialize_field("type", &self.avro_type)?;
            avro_schema_field.serialize_field("default", &self.default)?;
        }
        avro_schema_field.serialize_field("description", &self.description)?;
        avro_schema_field.serialize_field("units", &self.units)?;
        avro_schema_field.end()
    }
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

    /// Get field name.
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    /// Make avro schema for the field.
    pub fn make_avro_schema(&self) -> AvroSchemaField {
        AvroSchemaField {
            name: self.name.to_owned(),
            avro_type: FieldInfo::get_avro_array_type(&self.sal_type),
            default: self.get_default_value(),
            description: self.description.to_owned(),
            units: self.units.to_owned(),
            scalar: self.is_scalar(),
        }
    }

    /// Get default value.
    fn get_default_value(&self) -> Vec<String> {
        vec![self.default_scalar_value.to_string(); self.count]
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
    fn get_avro_scalar_type(sal_type: &str) -> String {
        let scalar_type = match sal_type {
            "boolean" => "boolean",
            "byte" => "int",
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
        };

        scalar_type.to_owned()
    }

    /// Return the avro type name from the sal type name.
    fn get_avro_array_type(sal_type: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("type".to_owned(), "array".to_owned()),
            (
                "items".to_owned(),
                FieldInfo::get_avro_scalar_type(sal_type),
            ),
        ])
    }

    /// Is this field value scalar?
    pub fn is_scalar(&self) -> bool {
        self.count == 1 || self.sal_type == "string"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn make_avro_schema_scalar() {
        let field_info = FieldInfo::new("value", "string", 1, "unitless", "Test value.");

        let avro_schema: AvroSchemaField = field_info.make_avro_schema();
        let avro_schema_str = serde_json::to_string(&avro_schema).unwrap();

        assert_eq!(
            avro_schema_str,
            r#"{"name":"value","type":"string","default":"","description":"Test value.","units":"unitless"}"#
        )
    }
    #[test]
    fn make_avro_schema_array() {
        let field_info = FieldInfo::new("value", "float", 5, "unitless", "Test value.");

        let avro_schema: AvroSchemaField = field_info.make_avro_schema();
        let avro_schema_str = serde_json::to_string(&avro_schema).unwrap();
        assert_eq!(
            avro_schema_str,
            r#"{"name":"value","type":{"items":"float","type":"array"},"default":["0","0","0","0","0"],"description":"Test value.","units":"unitless"}"#
        )
    }
}
