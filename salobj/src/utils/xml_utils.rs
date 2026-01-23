/// Returns default sal index.
pub fn get_default_sal_index() -> i32 {
    0
}

/// Convert a topic sal_name into its topic name.
pub fn convert_sal_name_to_topic_name(component_name: &str, topic_name: &str) -> String {
    topic_name
        .replace(&format!("{}_", component_name), "")
        .replace("SALGeneric_", "")
}
