use std::{
    collections::BTreeSet,
    fmt::Display,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    time::Duration,
};
use uuid::Uuid;

use crate::{
    config::{NodeId, OperatorId},
    descriptor::Descriptor,
};

pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub const DORA_COORDINATOR_PORT_DEFAULT: u16 = 0xD02A;
pub const DORA_DAEMON_LOCAL_LISTEN_PORT_DEFAULT: u16 = 0xD02B;
pub const DORA_COORDINATOR_PORT_CONTROL_DEFAULT: u16 = 0x177C;

pub const MANUAL_STOP: &str = "dora/stop";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum ControlRequest {
    Start {
        dataflow: Descriptor,
        name: Option<String>,
        // TODO: remove this once we figure out deploying of node/operator
        // binaries from CLI to coordinator/daemon
        local_working_dir: PathBuf,
    },
    Reload {
        dataflow_id: Uuid,
        node_id: NodeId,
        operator_id: Option<OperatorId>,
    },
    Check {
        dataflow_uuid: Uuid,
    },
    Stop {
        dataflow_uuid: Uuid,
        grace_duration: Option<Duration>,
    },
    StopByName {
        name: String,
        grace_duration: Option<Duration>,
    },
    Logs {
        uuid: Option<Uuid>,
        name: Option<String>,
        node: String,
    },
    Destroy,
    List,
    DaemonConnected,
    ConnectedMachines,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DataflowList(pub Vec<DataflowListEntry>);

impl DataflowList {
    pub fn get_active(&self) -> Vec<DataflowId> {
        self.0
            .iter()
            .filter(|d| d.status == DataflowStatus::Running)
            .map(|d| d.id.clone())
            .collect()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DataflowListEntry {
    pub id: DataflowId,
    pub status: DataflowStatus,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub enum DataflowStatus {
    Running,
    Finished,
    Failed,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ControlRequestReply {
    Error(String),
    CoordinatorStopped,
    DataflowStarted {
        uuid: Uuid,
    },
    DataflowReloaded {
        uuid: Uuid,
    },
    DataflowStopped {
        uuid: Uuid,
        result: Result<(), String>,
    },

    DataflowList(DataflowList),
    DestroyOk,
    DaemonConnected(bool),
    ConnectedMachines(BTreeSet<String>),
    Logs(Vec<u8>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataflowId {
    pub uuid: Uuid,
    pub name: Option<String>,
}

impl Display for DataflowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "[{name}] {}", self.uuid)
        } else {
            write!(f, "[<unnamed>] {}", self.uuid)
        }
    }
}
