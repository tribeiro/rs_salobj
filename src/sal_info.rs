use crate::domain;

///Information for one SAL component and index.
pub struct SalInfo<'a> {
    domain: &'a mut domain::Domain,
    name: &'a str,
    index: &'a usize,
    command_names: Vec<String>,
    event_names: Vec<String>,
    telemetry_names: Vec<String>,
}

impl<'a> SalInfo<'a> {
    pub fn new(domain: &'a mut domain::Domain, name: &'a str, index: &'a usize) -> SalInfo<'a> {
        SalInfo {
            domain: domain,
            name: name,
            index: index,
            command_names: Vec::new(),
            event_names: Vec::new(),
            telemetry_names: Vec::new(),
        }
    }
    pub fn get_command_names(&self) -> &Vec<String> {
        &self.command_names
    }
    pub fn get_event_names(&self) -> &Vec<String> {
        &self.event_names
    }
    pub fn get_telemetry_names(&self) -> &Vec<String> {
        &self.telemetry_names
    }
}
