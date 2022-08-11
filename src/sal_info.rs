use crate::component_info::ComponentInfo;
use crate::domain;
use crate::sal_enums;
use avro_rs::{
    types::{Record, Value},
    Schema,
};
use std::collections::HashMap;
use std::env;
use std::{cell::RefCell, rc::Rc};

///Information for one SAL component and index.
pub struct SalInfo {
    domain: Rc<RefCell<domain::Domain>>,
    name: String,
    index: usize,
    component_info: ComponentInfo,
    topic_schema: HashMap<String, Schema>,
}

impl SalInfo {
    /// Create a Reference Counted (Rc) instance of `SalInfo`.
    ///
    /// When creating an instance of `SalInfo` we require a Reference Counted
    /// Mutable Memory location instance of a `Domain` object (e.g.
    /// Rc<RefCell<Domain>>). The `Domain` contains information that is shared
    /// between a series of `SalInfo` and, as such it keeps a week reference
    /// of all `SalInfo` instances attached to it. Therefore, the only one
    /// to create an instance of `SalInfo` is to wrap it with an `Rc`.
    pub fn new(domain: Rc<RefCell<domain::Domain>>, name: &str, index: usize) -> Rc<SalInfo> {
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

        let sal_info = Rc::new(SalInfo {
            domain: domain,
            name: name.to_owned(),
            index: index,
            component_info: component_info,
            topic_schema: topic_schema,
        });

        sal_info.domain.borrow().add_salinfo(&sal_info);

        sal_info
    }

    /// Make an AckCmd `Record` from keyword arguments.
    ///
    /// A `Record` is an object that is built from the avro schema and,
    /// therefore, can be published directly afterwards.
    pub fn make_ackcmd(
        &self,
        private_seqnum: i32,
        ack: sal_enums::SalRetCode,
        error: i32,
        result: &str,
        timeout: f32,
    ) -> Record {
        let mut record = Record::new(&self.topic_schema.get("ackcmd").unwrap()).unwrap();
        record.put("private_seqNum", Value::Int(private_seqnum));
        record.put("ack", Value::Int(ack as i32));
        record.put("error", Value::Int(error));
        record.put("result", Value::String(result.to_owned()));
        record.put("timeout", Value::Float(timeout));

        record
    }

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

    #[test]
    fn make_ackcmd() {
        let domain = Rc::new(RefCell::new(domain::Domain::new()));
        let sal_info = SalInfo::new(domain, "Test", 1);

        let ackcmd = sal_info.make_ackcmd(
            12345,
            sal_enums::SalRetCode::CmdComplete,
            0,
            "Command completed successfully.",
            60.0,
        );

        let fields: HashMap<String, Value> = ackcmd.fields.into_iter().collect();

        let private_seqnum = match fields.get("private_seqNum").unwrap() {
            Value::Int(value) => value.to_owned(),
            _ => panic!("wrong type for private_seqNum."),
        };
        let ack = match fields.get("ack").unwrap() {
            Value::Int(value) => value.to_owned(),
            _ => panic!("wrong type for ack."),
        };
        let result = match fields.get("result").unwrap() {
            Value::String(value) => value.to_owned(),
            _ => panic!("wrong type for result."),
        };
        let timeout = match fields.get("timeout").unwrap() {
            Value::Float(value) => value.to_owned(),
            _ => panic!("wrong type for timeout."),
        };

        assert_eq!(private_seqnum, 12345 as i32);
        assert_eq!(ack, sal_enums::SalRetCode::CmdComplete as i32);
        assert_eq!(result, "Command completed successfully.");
        assert_eq!(timeout, 60.0);
    }
}
