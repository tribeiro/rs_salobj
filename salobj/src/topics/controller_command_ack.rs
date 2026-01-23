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
    pub async fn start(domain: &Domain, sal_info: &SalInfo) -> ControllerCommandAck {
        let (ack_sender, mut ack_receiver): (mpsc::Sender<CommandAck>, mpsc::Receiver<CommandAck>) =
            mpsc::channel(100);
        let mut ack_writer = WriteTopic::new("ackcmd", sal_info, domain);
        let sal_index: i32 = sal_info.get_index() as i32;
        let identity = domain.get_identity();
        let origin = domain.get_origin() as i32;

        let ack_task = task::spawn(async move {
            while let Some(command_ack) = ack_receiver.recv().await {
                let ackcmd = command_ack
                    .to_ackcmd()
                    .with_sal_index(sal_index)
                    .with_timestamps()
                    .with_private_identity(&identity)
                    .with_private_origin(origin)
                    .with_private_seq_num(command_ack.get_seq_num());
                log::debug!("{ackcmd:?}");
                ack_writer.set_seq_num(ackcmd.get_private_seq_num());
                if let Err(error) = ack_writer.write_typed(&ackcmd).await {
                    log::error!("Failed to write ackcmd: {error}");
                }
            }
        });
        ControllerCommandAck {
            ack_task,
            ack_sender,
        }
    }

    pub async fn is_finished(&self) -> bool {
        self.ack_task.is_finished()
    }
}
