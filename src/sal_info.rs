use crate::component_info::ComponentInfo;
use crate::domain;
use avro_rs::{
    Schema,
};
use std::collections::HashMap;
use std::env;

///Information for one SAL component and index.
pub struct SalInfo<'a> {
    domain: &'a mut domain::Domain,
    name: String,
    index: usize,
    component_info: ComponentInfo,
    topic_schema: HashMap<String, Schema>,
}

impl<'a> SalInfo<'a> {
    pub fn new(domain: &'a mut domain::Domain, name: &'a str, index: &'a usize) -> SalInfo<'a> {
        SalInfo {
        let topic_subname = match env::var("LSST_TOPIC_SUBNAME") {
            Ok(val) => val,
            Err(_) => panic!("You must define environment variable LSST_TOPIC_SUBNAME"),
        };
        let component_info = ComponentInfo::new(name, &topic_subname);

        if index != 0 && !component_info.is_indexed() {
            panic!("Invalid index={index}. Component {name} is not indexed. Index must be 0.")
        }

        let topic_schema = component_info
            .make_avro_schema()
            .into_iter()
            .map(|(topic, avro_schema)| {
                (
                    topic.to_owned(),
                    Schema::parse_str(&serde_json::to_string(&avro_schema).unwrap()).unwrap(),
                )
            })
            .collect();

            domain: domain,
            name: name,
            name: name.to_owned(),
            index: index,
            command_names: Vec::new(),
            event_names: Vec::new(),
            telemetry_names: Vec::new(),
            component_info: component_info,
            topic_schema: topic_schema,
    /// Is the component indexed?
    pub fn is_indexed(&self) -> bool {
        self.component_info.is_indexed()
    }

    /// Get name[:index]
    ///
    /// The suffix is only passed if the component is index.
    pub fn get_name_index(&self) -> String {
        if self.is_indexed() {
            format!("{}:{}", self.name, self.index)
        } else {
            format!("{}", self.name)
        }
    }

    /// Get name of all commands topics.
    pub fn get_command_names(&self) -> Vec<String> {
        self.component_info.get_command_names()
    }

    /// Get names of all events topics.
    pub fn get_event_names(&self) -> Vec<String> {
        self.component_info.get_event_names()
    }

    /// Get names of all telemetry topics.
    pub fn get_telemetry_names(&self) -> Vec<String> {
        self.component_info.get_telemetry_names()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sal_info_get_command_names() {
        let domain = Rc::new(RefCell::new(domain::Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        let command_names = sal_info.get_command_names();

        assert!(command_names.contains(&"Test_command_setScalars".to_owned()))
    }

    #[test]
    fn sal_info_get_event_names() {
        let domain = Rc::new(RefCell::new(domain::Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        let event_names = sal_info.get_event_names();

        assert!(event_names.contains(&"Test_logevent_scalars".to_owned()))
    }

    #[test]
    fn sal_info_get_telemetry_names() {
        let domain = Rc::new(RefCell::new(domain::Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        let telemetry_names = sal_info.get_telemetry_names();

        assert!(telemetry_names.contains(&"Test_scalars".to_owned()))
    }

    #[test]
    #[should_panic(expected = "Invalid index=1. Component ATMCS is not indexed. Index must be 0.")]
    fn panic_if_index_for_non_indexed() {
        let domain = Rc::new(RefCell::new(domain::Domain::new()));
        let _ = SalInfo::new(domain, "ATMCS", 1);
    }

    #[test]
    fn get_name_index_indexed() {
        let domain = Rc::new(RefCell::new(domain::Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        assert_eq!(sal_info.get_name_index(), "Test:1")
    }

    #[test]
    fn get_name_index_non_indexed() {
        let domain = Rc::new(RefCell::new(domain::Domain::new()));
        let sal_info = SalInfo::new(domain, "ATMCS", 0);

        assert_eq!(sal_info.get_name_index(), "ATMCS")
    }
}
