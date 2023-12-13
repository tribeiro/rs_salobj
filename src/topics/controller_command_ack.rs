//! Handles reading command topic and writing acknowledgements.

use tokio::{sync::mpsc, task};

use crate::{
    domain::Domain,
    sal_info::SalInfo,
    topics::{base_sal_topic::BaseSALTopic, write_topic::WriteTopic},
    utils::command_ack::CommandAck,
};

pub struct ControllerCommandAck {
    pub ack_sender: mpsc::Sender<CommandAck>,
    ack_task: task::JoinHandle<()>,
}

impl ControllerCommandAck {
    pub async fn start<'si>(domain: &Domain, sal_info: &SalInfo<'si>) -> ControllerCommandAck {
        let (ack_sender, mut ack_receiver): (mpsc::Sender<CommandAck>, mpsc::Receiver<CommandAck>) =
            mpsc::channel(100);
        let mut ack_writer = WriteTopic::new("ackcmd", sal_info, domain);
        let ack_task = task::spawn(async move {
            while let Some(command_ack) = ack_receiver.recv().await {
                let mut ackcmd = command_ack.to_ackcmd();
                ackcmd.set_private_seq_num(command_ack.get_seq_num());
                let _ = ack_writer.write_typed(&mut ackcmd).await;
            }
        });
        ControllerCommandAck {
            ack_task: ack_task,
            ack_sender: ack_sender,
        }
    }

    pub async fn is_finished(&self) -> bool {
        self.ack_task.is_finished()
    }
}
