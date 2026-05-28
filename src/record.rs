use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CommandRecord {
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub duration: Option<u64>,
    pub timestamp: i64,
    pub session_id: String,
    pub hostname: String,
}
