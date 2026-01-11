//! Client to interact with SAL components.
//!
//! A [Remote] is a client/user side tool to communicate with SAL components.
//! It allows users to send commands and receive events and telemetry to
//! components.

use crate::domain;
use crate::error::errors::{SalObjError, SalObjResult};
use crate::sal_info;

use crate::topics::remote_command;

use crate::topics::{
    base_sal_topic::BaseSALTopic, read_topic::ReadTopic, remote_command::RemoteCommand,
};
use crate::utils::command_ack::CommandAck;
use crate::utils::types::{ReadTopicSet, RemoteCommandSet};
use apache_avro::types::Record;
use apache_avro::types::Value;
use apache_avro::Schema;
use serde::Serialize;
use std::collections::HashMap;
use std::{fmt::Debug, time::Duration};

/// Handle operations on a remote SAL object.
/// This object can execute commands to and receive telemetry and events from
/// a SAL component.
///
/// If a SAL component listens to or commands other SAL components
/// then it will have one Remote for each such component.
pub struct Remote<'b> {
    sal_info: sal_info::SalInfo,
    commands: RemoteCommandSet<'b>,
    events: ReadTopicSet<'b>,
    telemetry: ReadTopicSet<'b>,
}

impl<'b> Remote<'b> {
    pub async fn new(
        domain: &mut domain::Domain,
        name: &str,
        index: isize,
        readonly: bool,
        include: Vec<String>,
        exclude: Vec<String>,
        evt_max_history: usize,
    ) -> SalObjResult<Remote<'b>> {
        if !include.is_empty() && !exclude.is_empty() {
            panic!("include_only and exclude can not both have elements.");
        } else if !include.is_empty() || !exclude.is_empty() {
            unimplemented!(
                "Including only a subset of topics or \
                excluding a subset of topics is not implemented yet."
            );
        }

        let sal_info = sal_info::SalInfo::new(name, index)?;

        if let Err(error) = domain.register_topics(&sal_info.get_topics_name()).await {
            log::warn!("Failed to register topics: {error:?}. Continuing...");
        }

        let commands: RemoteCommandSet = if readonly {
            HashMap::new()
        } else {
            sal_info
                .get_command_names()
                .into_iter()
                .map(|command_name| {
                    (
                        command_name.to_owned(),
                        RemoteCommand::new(&command_name, domain, &sal_info),
                    )
                })
                .collect()
        };

        let events: ReadTopicSet = sal_info
            .get_event_names()
            .into_iter()
            .map(|event_name| {
                (
                    event_name.to_owned(),
                    ReadTopic::new(&event_name, &sal_info, domain, evt_max_history),
                )
            })
            .collect();

        let telemetry: ReadTopicSet = sal_info
            .get_telemetry_names()
            .into_iter()
            .map(|telemetry_name| {
                (
                    telemetry_name.to_owned(),
                    ReadTopic::new(&telemetry_name, &sal_info, domain, 0),
                )
            })
            .collect();

        Ok(Remote {
            sal_info,
            commands,
            events,
            telemetry,
        })
    }

    pub async fn from_name_index(
        domain: &mut domain::Domain,
        name: &str,
        index: isize,
    ) -> SalObjResult<Remote<'b>> {
        Remote::new(domain, name, index, false, Vec::new(), Vec::new(), 1).await
    }

    /// Get component name.
    pub fn get_name(&self) -> String {
        self.sal_info.get_name()
    }

    pub fn get_command_schema(&self, command_name: &str) -> Option<Schema> {
        Some(self.commands.get(command_name)?.get_schema().clone())
    }

    /// Get component index.
    pub fn get_index(&self) -> isize {
        self.sal_info.get_index()
    }

    pub async fn run_command<'c>(
        &mut self,
        command_name: String,
        parameters: &mut Record<'c>,
        timeout: Duration,
        wait_done: bool,
    ) -> remote_command::AckCmdResult {
        if !self.sal_info.is_command(&command_name) {
            return Err(CommandAck::invalid_command(&format!(
                "Invalid command name {command_name}."
            )));
        }

        if let Some(command) = self.commands.get_mut(&command_name) {
            command.run(parameters, timeout, wait_done).await
        } else {
            Err(CommandAck::invalid_command(&format!(
                "Command {command_name} not in the list of commands."
            )))
        }
    }

    pub fn get_command_data<T>(&self, cmd_name: &str) -> SalObjResult<T>
    where
        T: BaseSALTopic + Default + Debug,
    {
        if let Some(command) = self.commands.get(cmd_name) {
            let seq_num = command.get_seq_num();
            let origin = command.get_origin();
            let identity = command.get_identity();
            let sal_index = command.get_index();
            let data = T::default()
                .with_timestamps()
                .with_private_seq_num(seq_num)
                .with_private_origin(origin)
                .with_private_identity(&identity)
                .with_sal_index(sal_index);
            Ok(data)
        } else {
            Err(SalObjError::new(&format!("No event topic {cmd_name}")))
        }
    }

    pub async fn run_command_typed<T>(
        &mut self,
        command_name: &str,
        data: &T,
        timeout: Duration,
        wait_done: bool,
    ) -> remote_command::AckCmdResult
    where
        T: BaseSALTopic + Serialize + Debug,
    {
        if let Some(command) = self.commands.get_mut(command_name) {
            command.run_typed(data, timeout, wait_done).await
        } else {
            Err(CommandAck::invalid_command(&format!(
                "Command {command_name} not in the list of commands."
            )))
        }
    }

    pub async fn pop_event_front(
        &mut self,
        event_name: &str,
        flush: bool,
        timeout: Duration,
    ) -> std::result::Result<Option<Value>, ()> {
        if let Some(event_reader) = self.events.get_mut(event_name) {
            Ok(event_reader.pop_front(flush, timeout).await)
        } else {
            Err(())
        }
    }

    pub async fn pop_event_back(
        &mut self,
        event_name: &str,
        flush: bool,
        timeout: Duration,
    ) -> std::result::Result<Option<Value>, ()> {
        if let Some(event_reader) = self.events.get_mut(event_name) {
            Ok(event_reader.pop_back(flush, timeout).await)
        } else {
            Err(())
        }
    }

    pub async fn pop_telemetry_front(
        &mut self,
        telemetry_name: &str,
        flush: bool,
        timeout: Duration,
    ) -> std::result::Result<Option<Value>, ()> {
        if let Some(telemetry_reader) = self.telemetry.get_mut(telemetry_name) {
            Ok(telemetry_reader.pop_front(flush, timeout).await)
        } else {
            Err(())
        }
    }

    pub async fn pop_telemetry_back(
        &mut self,
        telemetry_name: &str,
        flush: bool,
        timeout: Duration,
    ) -> std::result::Result<Option<Value>, ()> {
        if let Some(telemetry_reader) = self.telemetry.get_mut(telemetry_name) {
            Ok(telemetry_reader.pop_back(flush, timeout).await)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_name() {
        let mut domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let remote = Remote::from_name_index(&mut domain, name, index)
            .await
            .unwrap();

        assert_eq!("Test", remote.get_name())
    }

    #[tokio::test]
    async fn test_get_index() {
        let mut domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let remote = Remote::from_name_index(&mut domain, name, index)
            .await
            .unwrap();

        assert_eq!(index, remote.get_index());
    }
}
