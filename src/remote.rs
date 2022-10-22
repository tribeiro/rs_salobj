use crate::domain;
use crate::sal_info;
use crate::topics::{remote_command, remote_event, remote_telemetry};
use std::collections::HashMap;

/// Handle operations on a remote SAL object.
/// This object can execute commands to and receive telemetry and events from
/// a SAL component.
///
/// If a SAL component listens to or commands other SAL components
/// then it will have one Remote for each such component.
pub struct Remote {
    name: String,
    index: isize,
    domain: domain::Domain,
    sal_info: sal_info::SalInfo,
    commands: HashMap<String, remote_command::RemoteCommand>,
    events: HashMap<String, remote_event::RemoteEvent>,
    telemetry: HashMap<String, remote_telemetry::RemoteTelemetry>,
    started: bool,
}

impl Remote {
    pub fn new(
        domain: domain::Domain,
        name: &str,
        index: &isize,
        readonly: bool,
        include: Vec<String>,
        exclude: Vec<String>,
        evt_max_history: usize,
    ) -> Remote {
        let mut remote = Remote {
            name: String::from(name),
            domain: domain,
            index: index.clone(),
            sal_info: sal_info::SalInfo::new(name, *index),
            commands: HashMap::new(),
            events: HashMap::new(),
            telemetry: HashMap::new(),
            started: false,
        };

        remote.start(readonly, include, exclude, evt_max_history);
        remote
    }

    pub fn from_name_index(domain: domain::Domain, name: &str, index: &isize) -> Remote {
        Remote::new(domain, name, index, true, Vec::new(), Vec::new(), 1)
    }

    /// Start remote.
    ///
    /// This method setup the remote command, event and telemetry streams.
    ///
    /// # Panic
    ///
    /// If remote was already started.
    fn start(
        &mut self,
        readonly: bool,
        include: Vec<String>,
        exclude: Vec<String>,
        evt_max_history: usize,
    ) {
        assert!(!self.started);

        if !readonly {
            self.create_commmands();
        }

        let events_name = self.get_events_name(&include, &exclude);

        self.create_events(&events_name);

        let telemetry_names = self.get_telemetry_names(&include, &exclude);

        self.create_telemetry(&telemetry_names);

        self.started = true;
    }

    /// Create commands for remote.
    ///
    /// # Panic
    ///
    /// If remote was already started.
    fn create_commmands(&mut self) {
        assert!(!self.started);

        for cmd_name in self.sal_info.get_command_names() {
            let command = remote_command::RemoteCommand::new(&self.sal_info, &cmd_name);
            self.commands.entry(cmd_name.to_string()).or_insert(command);
        }
    }

    /// Create events for remote.
    ///
    /// # Panic
    ///
    /// If remote was already started.
    fn create_events(&mut self, events_name: &Vec<String>) {
        assert!(!self.started);
    }

    /// Create telemetry for remote.
    ///
    /// # Panic
    ///
    /// If remote was already started.
    fn create_telemetry(&mut self, telemetry_names: &Vec<String>) {
        assert!(!self.started);
    }

    /// Construct a list of valid event names.
    ///
    /// If both `include_only` and `exclude` are empty, return vector with all
    /// event names.
    ///
    /// If `include_only` is not empty and `exclude` is empty output will
    /// contain only the names specified in that.
    ///
    /// If `include_only` is empty and `exclude` is not empty, output will
    /// exclude names in the later.
    ///
    /// If neither input is empty the function will panic.
    ///
    /// # Panic
    ///
    /// If both `include_only` and `exclude` are not empty.
    fn get_events_name(&self, include_only: &Vec<String>, exclude: &Vec<String>) -> Vec<String> {
        if include_only.len() > 0 && exclude.len() > 0 {
            panic!("include_only and exclude can not both have elements.");
        }

        Vec::new()
    }

    /// Construct a list of valid telemetry names.
    ///
    /// This method is similar to `get_events_names` but for telemetry.
    fn get_telemetry_names(
        &self,
        include_only: &Vec<String>,
        exclude: &Vec<String>,
    ) -> Vec<String> {
        if include_only.len() > 0 && exclude.len() > 0 {
            panic!("include_only and exclude can not both have elements.");
        }
        Vec::new()
    }

    /// Get component name.
    pub fn get_name(&self) -> String {
        String::from(&self.name)
    }

    /// Get component index.
    pub fn get_index(&self) -> isize {
        self.index.clone()
    }

    pub fn get_salindex(&self) -> &sal_info::SalInfo {
        &self.sal_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_name() {
        let domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let remote = Remote::from_name_index(domain, name, &index);

        assert_eq!("Test", remote.get_name())
    }

    #[test]
    fn test_get_index() {
        let domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let remote = Remote::from_name_index(domain, name, &index);

        assert_eq!(index, remote.get_index());
    }

    #[test]
    #[should_panic(expected = "include_only and exclude can not both have elements.")]
    fn test_get_events_name_panic_if_include_and_exclude() {
        let domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let remote = Remote::from_name_index(domain, name, &index);

        let include_only = vec![String::from("event1")];
        let exclude = vec![String::from("event2")];

        remote.get_events_name(&include_only, &exclude);
    }

    #[test]
    #[should_panic(expected = "include_only and exclude can not both have elements.")]
    fn test_get_telemetry_names_panic_if_include_and_exclude() {
        let domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let remote = Remote::from_name_index(domain, name, &index);

        let include_only = vec![String::from("telemtry1")];
        let exclude = vec![String::from("telemetry2")];

        remote.get_events_name(&include_only, &exclude);
    }

    #[test]
    #[should_panic]
    fn test_start_when_started() {
        let domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let mut remote = Remote::from_name_index(domain, name, &index);

        remote.start(false, Vec::new(), Vec::new(), 1);
    }

    #[test]
    #[should_panic]
    fn test_create_commands_when_started() {
        let domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let mut remote = Remote::from_name_index(domain, name, &index);

        remote.create_commmands();
    }

    #[test]
    #[should_panic]
    fn test_create_events_when_started() {
        let domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let mut remote = Remote::from_name_index(domain, name, &index);

        let events_name = vec![];

        remote.create_events(&events_name);
    }

    #[test]
    #[should_panic]
    fn test_create_telemetry_when_started() {
        let domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let mut remote = Remote::from_name_index(domain, name, &index);

        let telemetry_names = vec![];
        remote.create_telemetry(&telemetry_names);
    }
}
