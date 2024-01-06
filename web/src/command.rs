use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone, Eq, Hash)]
pub struct SubCommand {
    pub command: String,
    pub description: String,
    pub placeholders: Option<Vec<Placeholder>>,
}

impl ToString for SubCommand {
    fn to_string(&self) -> String {
        format!("{} : {}", self.command, self.description)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone, Eq, Hash)]
pub struct Placeholder {
    pub name: String,
    pub description: String,
}
