use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, EnumVariantNames};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, EnumString, EnumVariantNames, Deserialize, Serialize,
)]
#[strum(ascii_case_insensitive)]
pub enum TodoTypes {
    #[default]
    Task,
    Daily,
    Weekly,
}

#[derive(Debug, Default, PartialEq, EnumString, EnumVariantNames)]
#[strum(ascii_case_insensitive)]
pub enum Day {
    Monday,
    Tuesday,
    #[default]
    Sunday,
}

// TODO convert due_date in a proper type
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Todo {
    pub name: String,
    pub descr: Option<String>,
    pub tags: Vec<String>,
    pub todo_type: TodoTypes,
    pub checklist: Vec<CheckListItem>,
    pub due_date: Option<String>,
    pub done: bool,
}

impl std::fmt::Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.name)?;
        if self.tags.is_empty() {
            writeln!(f, "  labels: none")?;
        } else {
            write!(f, "  labels: ")?;
            self.tags.iter().for_each(|t| write!(f, "{} ", t).unwrap());
            writeln!(f)?;
        }
        if self.due_date.is_some() {
            writeln!(f, "  due date: {}", self.due_date.clone().unwrap())?;
        }
        if !self.checklist.is_empty() {
            writeln!(f, "  checklist:")?;
            self.checklist
                .iter()
                .for_each(|t| writeln!(f, "{}", t).unwrap());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckListItem {
    pub name: String,
    pub done: bool,
}

impl std::fmt::Display for CheckListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.done {
            write!(f, "  x {}", self.name)
        } else {
            write!(f, "  o {}", self.name)
        }
    }
}

impl From<String> for Todo {
    fn from(name: String) -> Self {
        Todo {
            name,
            ..Todo::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TodoTypes;
    use std::str::FromStr;
    #[test]
    fn todo_type_from_str() {
        let uut: TodoTypes = TodoTypes::from_str("Task").unwrap();
        assert_eq!(TodoTypes::Task, uut);
        let uut: TodoTypes = TodoTypes::from_str("task").unwrap();
        assert_eq!(TodoTypes::Task, uut);
    }
}
