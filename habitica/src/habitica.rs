use aide_proto::v1::todo::{CheckListItem, Todo};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RespTask {
    pub success: bool,
    pub data: Vec<Task>,
}

#[derive(Deserialize, Debug)]
pub struct Task {
    #[serde(rename(deserialize = "id"))]
    pub id: String,
    #[serde(rename(deserialize = "text"))]
    pub description: String,
    #[serde(rename(deserialize = "type"))]
    pub task_type: TaskTypes,
    #[serde(rename(deserialize = "notes"))]
    pub notes: String,
    #[serde(rename(deserialize = "tags"))]
    pub tags: Vec<String>,
    #[serde(rename(deserialize = "checklist"))]
    pub checklist: Option<Vec<Checklist>>,
    #[serde(rename(deserialize = "completed"))]
    pub completed: Option<bool>,
}

impl From<&Task> for Todo {
    fn from(t: &Task) -> Self {
        let name = t.description.clone();
        let descr = Some(t.notes.clone());
        let tags = t.tags.clone();
        let checklist = match t.checklist.clone() {
            None => Vec::new(),
            Some(v) => v.iter().map(|cl| cl.into()).collect(),
        };
        let due_date = None;
        let done = t.completed.unwrap_or(false);
        Todo {
            name,
            descr,
            tags,
            todo_type: aide_proto::v1::todo::TodoTypes::Task,
            checklist,
            due_date,
            done,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum TaskTypes {
    #[serde(rename(deserialize = "habit"))]
    Habit,
    #[serde(rename(deserialize = "todo"))]
    Todo,
    #[serde(rename(deserialize = "reward"))]
    Reward,
    #[serde(rename(deserialize = "daily"))]
    Daily,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Checklist {
    completed: bool,
    text: String,
    #[allow(dead_code)]
    id: String,
}

impl From<&Checklist> for CheckListItem {
    fn from(cl: &Checklist) -> Self {
        CheckListItem {
            name: cl.text.clone(),
            done: cl.completed,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RespDaily {
    pub success: bool,
    pub data: Vec<Daily>,
}

#[derive(Deserialize, Debug)]
pub struct Daily {
    #[serde(rename(deserialize = "id"))]
    pub id: String,
    #[serde(rename(deserialize = "text"))]
    pub description: String,
    #[serde(rename(deserialize = "type"))]
    pub task_type: TaskTypes,
    #[serde(rename(deserialize = "notes"))]
    pub notes: String,
    #[serde(rename(deserialize = "tags"))]
    pub tags: Vec<String>,
    #[serde(rename(deserialize = "checklist"))]
    pub checklist: Option<Vec<Checklist>>,
    #[serde(rename(deserialize = "completed"))]
    pub completed: Option<bool>,
    #[serde(rename(deserialize = "repeat"))]
    pub repeat: Repeat,
    #[serde(rename(deserialize = "frequency"))]
    pub frequency: String,
    #[serde(rename(deserialize = "nextDue"))]
    pub next_due: Vec<String>,
    #[serde(rename(deserialize = "everyX"))]
    pub every: u32,
}

impl Daily {
    pub fn is_due(&self) -> bool {
        use chrono::prelude::*;
        let date = Local::now().naive_local() - chrono::Duration::hours(4);
        self.is_due_today(&date.date())
    }
    fn is_due_today(&self, today: &chrono::NaiveDate) -> bool {
        if let Some(done) = self.completed {
            if done {
                return false;
            }
        }
        match self.frequency.as_str() {
            "daily" => true,
            "weekly" => {
                if self.repeat.is_today_on(today) {
                    true
                } else if self.repeat.is_only_sunday() {
                    !self.is_checklist_done()
                } else {
                    false
                }
            }
            _ => true,
        }
    }
    fn is_checklist_done(&self) -> bool {
        if let Some(cl) = self.checklist.clone() {
            for item in cl.iter() {
                if !item.completed {
                    return false;
                }
            }
        }
        true
    }
}

impl From<&Daily> for Todo {
    fn from(t: &Daily) -> Self {
        let name = t.description.clone();
        let descr = Some(t.notes.clone());
        let tags = t.tags.clone();
        let checklist = match t.checklist.clone() {
            None => Vec::new(),
            Some(v) => v.iter().map(|cl| cl.into()).collect(),
        };
        use chrono::prelude::*;
        let date = Local::now().naive_local() - chrono::Duration::hours(4);
        let due_date = if t.is_due() && t.repeat.is_today_on(&date.date()) {
            Some(format!("{}", date.date().format("%Y-%m-%d")))
        } else if t.is_due() && t.frequency.as_str() == "weekly" {
            let days_to_sunday = 7i64 - date.weekday().num_days_from_sunday() as i64;
            let next_sunday = date + chrono::Duration::days(days_to_sunday);
            Some(format!("{}", next_sunday.date().format("%Y-%m-%d")))
        } else {
            None
        };
        let done = t.completed.unwrap_or(false);
        // Assigning TodoTypes Daily, becuase at this time it would be too complicated to know
        // if it's Daily or Weekly
        Todo {
            name,
            descr,
            tags,
            todo_type: aide_proto::v1::todo::TodoTypes::Daily,
            checklist,
            due_date,
            done,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Repeat {
    pub su: bool,
    pub s: bool,
    pub f: bool,
    pub th: bool,
    pub w: bool,
    pub t: bool,
    pub m: bool,
}

impl Repeat {
    fn _is_all_week(&self) -> bool {
        self.su && self.s && self.f && self.th && self.w && self.t && self.m
    }
    fn is_only_sunday(&self) -> bool {
        self.su && !self.s && !self.f && !self.th && !self.w && !self.t && !self.m
    }
    fn is_today_on(&self, today: &chrono::NaiveDate) -> bool {
        use chrono::prelude::*;
        match today.weekday() {
            Weekday::Mon => self.m,
            Weekday::Tue => self.t,
            Weekday::Wed => self.w,
            Weekday::Thu => self.th,
            Weekday::Fri => self.f,
            Weekday::Sat => self.s,
            Weekday::Sun => self.su,
        }
    }
}
#[derive(Deserialize, Debug)]
pub struct RespTags {
    pub success: bool,
    pub data: Vec<Tag>,
}

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub name: String,
    pub id: String,
}

#[derive(strum::Display, Debug, Clone, PartialEq)]
pub enum UsersTaskTypes {
    #[allow(dead_code)]
    #[strum(serialize = "habit")]
    Habits,
    #[strum(serialize = "todos")]
    Todos,
    #[allow(dead_code)]
    #[strum(serialize = "reward")]
    Rewards,
    #[strum(serialize = "dailys")]
    Dailys,
    #[allow(dead_code)]
    #[strum(serialize = "completedTodos")]
    CompletedTodos,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checklist_conversion() {
        let input = Checklist {
            completed: true,
            text: "Checklist item 1".to_string(),
            id: "ignored".to_string(),
        };
        let uut = aide_proto::v1::todo::CheckListItem::from(&input);
        assert_eq!(uut.done, input.completed);
        assert_eq!(uut.name, input.text);
    }
    #[test]
    fn test_checklist_vector_conversion() {
        let cl1 = Checklist {
            completed: true,
            text: "Checklist item 1".to_string(),
            id: "ignored".to_string(),
        };
        let cl2 = Checklist {
            completed: false,
            text: "Checklist item 2".to_string(),
            id: "ignored".to_string(),
        };
        let input = vec![cl1, cl2];
        let uut: Vec<CheckListItem> = input.iter().map(|cl| cl.into()).collect();
        assert_eq!(uut.len(), 2);
        assert_eq!(uut[0].done, input[0].completed);
        assert_eq!(uut[0].name, input[0].text);
        assert_eq!(uut[1].done, input[1].completed);
        assert_eq!(uut[1].name, input[1].text);
    }
    #[test]
    fn test_due_daily_with_dates() {
        let repeat = Repeat {
            su: true,
            s: false,
            f: false,
            th: false,
            w: false,
            t: false,
            m: false,
        };
        let mut uut = Daily {
            id: "ID".to_string(),
            description: "Short Description".to_string(),
            task_type: TaskTypes::Daily,
            notes: "Long Description".to_string(),
            tags: Vec::new(),
            checklist: None,
            completed: None,
            repeat,
            frequency: "weekly".to_string(),
            next_due: Vec::new(),
            every: 1,
        };
        // without a checklist, it's a sunday only thing
        // 2021-02-27: saturday
        let saturday = chrono::NaiveDate::from_ymd(2021, 2, 27);
        assert!(!uut.is_due_today(&saturday));
        // 2021-02-28: sunday
        let sunday = chrono::NaiveDate::from_ymd(2021, 2, 28);
        assert!(uut.is_due_today(&sunday));
        // with a checklist, it's a weekly thing
        let cl1 = Checklist {
            completed: false,
            text: "Checklist item 1".to_string(),
            id: "ignored".to_string(),
        };
        let cl = vec![cl1];
        uut.checklist = Some(cl);
        assert!(!uut.is_checklist_done());
        assert!(uut.repeat.is_only_sunday());
        assert!(!uut.repeat.is_today_on(&saturday));
        assert!(uut.repeat.is_today_on(&sunday));
        assert!(uut.is_due_today(&saturday));
        assert!(uut.is_due_today(&sunday));
        // with a completed checklist, it's done, but not the task
        let cl1 = Checklist {
            completed: true,
            text: "Checklist item 1".to_string(),
            id: "ignored".to_string(),
        };
        let cl = vec![cl1];
        uut.checklist = Some(cl);
        assert!(uut.is_checklist_done());
        assert!(uut.repeat.is_only_sunday());
        assert!(!uut.repeat.is_today_on(&saturday));
        assert!(uut.repeat.is_today_on(&sunday));
        assert!(!uut.is_due_today(&saturday));
        assert!(uut.is_due_today(&sunday));
    }
    #[test]
    fn test_due_daily() {
        let repeat = Repeat {
            su: true,
            s: true,
            f: true,
            th: true,
            w: true,
            t: true,
            m: true,
        };
        let mut uut = Daily {
            id: "ID".to_string(),
            description: "Short Description".to_string(),
            task_type: TaskTypes::Daily,
            notes: "Long Description".to_string(),
            tags: Vec::new(),
            checklist: None,
            completed: None,
            repeat,
            frequency: "weekly".to_string(),
            next_due: Vec::new(),
            every: 1,
        };
        assert!(uut.is_due());
        uut.completed = Some(false);
        assert!(uut.is_due());
        uut.completed = Some(true);
        assert!(!uut.is_due());
    }

    #[test]
    fn test_next_due_conversion() {
        let next_due = "Wed Dec 09 2020 00:00:00 GMT+0100";
        assert!(
            chrono::NaiveDateTime::parse_from_str(next_due, "%a %b %d %Y %H:%M:%S GMT%z").is_ok()
        )
    }
}
