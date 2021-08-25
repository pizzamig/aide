use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, EnumVariantNames};

#[derive(Debug, PartialEq, EnumString, EnumVariantNames)]
#[strum(ascii_case_insensitive)]
pub enum ModuleKind {
    Todo,
    Reminder,
    Notification,
    Event,
    Weather,
    Generic,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetModuleKindResponse {
    pub data: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::ModuleKind;
    use std::str::FromStr;
    #[test]
    fn module_kind_from_str() {
        assert_eq!(ModuleKind::Todo, ModuleKind::from_str("Todo").unwrap());
        assert_eq!(ModuleKind::Todo, ModuleKind::from_str("todo").unwrap());
        assert_eq!(
            ModuleKind::Reminder,
            ModuleKind::from_str("Reminder").unwrap()
        );
        assert_eq!(
            ModuleKind::Reminder,
            ModuleKind::from_str("reminder").unwrap()
        );
        assert_eq!(
            ModuleKind::Weather,
            ModuleKind::from_str("Weather").unwrap()
        );
        assert_eq!(
            ModuleKind::Weather,
            ModuleKind::from_str("weather").unwrap()
        );
    }
}
