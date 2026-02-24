use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum ActionPayload {
    #[serde(rename = "CREATE_TICKET")]
    CreateTicket {
        id: String,
        title: String,
        queue: String,
        #[serde(default)]
        points: u32,
    },
    #[serde(rename = "UPDATE_TICKET")]
    UpdateTicket {
        id: String,
        #[serde(default)]
        points: u32,
    },
    #[serde(rename = "CHANGE_STATUS")]
    ChangeStatus {
        id: String,
        from: String,
        to: String,
    },
    #[serde(rename = "ADD_COMMENT")]
    AddComment { id: String, comment_id: String },
    #[serde(rename = "ASSIGN_TICKET")]
    AssignTicket {
        id: String,
        assignee: Option<String>,
    },
    #[serde(rename = "ATTACH_FILE")]
    AttachFile { id: String, filename: String },
    #[serde(rename = "UPDATE_BOARD_INFO")]
    UpdateBoardInfo,
    #[serde(rename = "MANAGE_USERS")]
    ManageUsers { op: String, user: String },
    #[serde(rename = "MANAGE_QUEUES")]
    ManageQueues {
        op: String,
        queue: String,
        new_name: Option<String>,
    },
}

impl std::fmt::Display for ActionPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let action_name = match self {
            Self::CreateTicket { .. } => "CREATE_TICKET",
            Self::UpdateTicket { .. } => "UPDATE_TICKET",
            Self::ChangeStatus { .. } => "CHANGE_STATUS",
            Self::AddComment { .. } => "ADD_COMMENT",
            Self::AssignTicket { .. } => "ASSIGN_TICKET",
            Self::AttachFile { .. } => "ATTACH_FILE",
            Self::UpdateBoardInfo => "UPDATE_BOARD_INFO",
            Self::ManageUsers { .. } => "MANAGE_USERS",
            Self::ManageQueues { .. } => "MANAGE_QUEUES",
        };
        write!(f, "{}", action_name)
    }
}
