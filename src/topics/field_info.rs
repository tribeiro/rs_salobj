extern crate serde;

#[derive(Deserialize, Debug)]
enum SalType {
    SalBool(bool),
    SalFloat(f32),
    SalInt(u32),
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
