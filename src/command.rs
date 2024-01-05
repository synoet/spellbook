use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub name: String,
    pub commands: Vec<SubCommand>,
}

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

pub trait CompareSubCommands {
    fn compare(&self, other: &Self) -> (Vec<SubCommand>, Vec<SubCommand>);
}

impl CompareSubCommands for Vec<SubCommand> {
    fn compare(&self, other: &Vec<SubCommand>) -> (Vec<SubCommand>, Vec<SubCommand>) {
        let set1: HashSet<_> = self.iter().cloned().collect();
        let set2: HashSet<_> = other.iter().cloned().collect();

        let added: HashSet<_> = set2.difference(&set1).collect();
        let removed: HashSet<_> = set1.difference(&set2).collect();
        (
            added.into_iter().cloned().collect::<Vec<_>>().clone(),
            removed.into_iter().cloned().collect::<Vec<_>>().clone(),
        )
    }
}
