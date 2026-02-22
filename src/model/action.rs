use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum ActionPayload {
    #[serde(rename = "CREATE_TICKET")]
    CreateTicket { id: String, title: String },
    #[serde(rename = "UPDATE_TICKET")]
    UpdateTicket { id: String },
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
        };
        write!(f, "{}", action_name)
    }
}
