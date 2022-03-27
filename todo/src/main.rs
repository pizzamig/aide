mod cli;
use aide_proto::v1::{todo::TodoTypes, ResultResponse, Todo as AideTodo};
use clap::Parser;
use crossterm::event::{Event, KeyCode};

fn main() -> Result<(), anyhow::Error> {
    let opt: cli::Opt = cli::Opt::parse();
    let base_url = reqwest::Url::parse(&format!(
        "{}://{}:{}/v1/",
        opt.common_opt.get_proto_str(),
        opt.common_opt.host_addr,
        opt.common_opt.port
    ))?;
    if opt.command.is_some() {
        if let Some(cli::Subcommands::Label {
            name,
            create,
            delete,
        }) = &opt.command
        {
            if *create {
                use aide_proto::v1::todo::Label;
                let label = Label { name: name.clone() };
                let url = base_url.join("labels")?;
                let client = reqwest::blocking::Client::new();
                let _res: ResultResponse = client
                    .post(url)
                    .body(serde_json::to_string(&label)?)
                    .send()?
                    .json()?;
                return Ok(());
            } else {
                assert!(delete);
                let url_path = format!("labels/{name}");
                let url = base_url.join(&url_path)?;
                let client = reqwest::blocking::Client::new();
                let _res: ResultResponse = client.delete(url).send()?.json()?;
                return Ok(());
            }
        }
    }
    let todos: Vec<AideTodo> = match opt.todo_type {
        Some(cli::TodoTypes::Task) => {
            let url = base_url.join("types/task/todos")?;
            let res = reqwest::blocking::get(url)?;
            res.json()?
        }
        Some(cli::TodoTypes::Daily) => {
            let url = base_url.join("types/daily/todos")?;
            let res = reqwest::blocking::get(url)?;
            res.json()?
        }
        Some(cli::TodoTypes::Weekly) => {
            let url = base_url.join("types/weekly/todos")?;
            let res = reqwest::blocking::get(url)?;
            res.json()?
        }
        Some(cli::TodoTypes::Periodic) => {
            let url = base_url.join("types/daily/todos")?;
            let res = reqwest::blocking::get(url)?;
            let mut todos: Vec<AideTodo> = res.json()?;
            let url = base_url.join("types/weekly/todos")?;
            let res = reqwest::blocking::get(url)?;
            let mut temp_todos: Vec<AideTodo> = res.json()?;
            todos.append(&mut temp_todos);
            todos
        }
        None => {
            let url = base_url.join("todos")?;
            let res = reqwest::blocking::get(url)?;
            res.json()?
        }
    };
    let temp_todos: Vec<&AideTodo> = if let Some(label) = opt.label {
        todos.iter().filter(|t| t.tags.contains(&label)).collect()
    } else {
        todos.iter().collect()
    };

    if opt.tui {
        tui_todo(&temp_todos)?;
    } else {
        temp_todos.iter().for_each(print_todo);
        println!("total: {}", get_todos_count(&temp_todos));
    }

    Ok(())
}

fn print_todo(t: &&AideTodo) {
    let type_symbol = match t.todo_type {
        TodoTypes::Task => "[T]",
        TodoTypes::Daily => "[D]",
        TodoTypes::Weekly => "[W]",
    };
    println!("{} {}", type_symbol, t.name);
    if !t.checklist.is_empty() {
        t.checklist
            .iter()
            .filter(|c| !c.done)
            .for_each(|c| println!("\t{}", c.name));
    }
    if t.due_date.is_some() {
        println!("\tdue date: {}", t.due_date.to_owned().unwrap())
    }
}

fn get_todos_count(v: &[&AideTodo]) -> i32 {
    let mut result = 0;
    for t in v {
        if t.checklist.is_empty() {
            result += 1;
        } else {
            for c in t.checklist.iter() {
                if !c.done {
                    result += 1;
                }
            }
        }
    }
    result
}

struct TodoStatefulList<'a> {
    todo_list: &'a [&'a AideTodo],
    state: usize,
}

impl<'a> TodoStatefulList<'a> {
    fn new(todo_list: &'a [&'a AideTodo], state: usize) -> Self {
        Self { todo_list, state }
    }
}

impl<'a> aide_common::tui::ToStringVec for TodoStatefulList<'a> {
    fn to_string_vec(&self) -> Vec<String> {
        self.todo_list
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i != self.state {
                    todo_to_one_line(t)
                } else {
                    todo_to_multi_line(t)
                }
            })
            .collect()
    }
}
impl<'a> aide_common::tui::ToListState for TodoStatefulList<'a> {
    fn to_state(&self) -> tui::widgets::ListState {
        let mut result = tui::widgets::ListState::default();
        result.select(Some(self.state));
        result
    }
}

impl<'a> aide_common::tui::GetTitle for TodoStatefulList<'a> {
    fn get_title(&self) -> &str {
        "Todo"
    }
}
fn tui_todo(v: &[&AideTodo]) -> Result<(), anyhow::Error> {
    if v.is_empty() {
        println!("There are no todos!");
        return Ok(());
    }
    let mut terminal = aide_common::tui::tui_setup()?;
    if let Err(e) = tui_todo_internal(v, &mut terminal) {
        aide_common::tui::tui_teardown(&mut terminal).unwrap_or(());
        return Err(e);
    }
    aide_common::tui::tui_teardown(&mut terminal)?;
    Ok(())
}

use std::io::Write;
use tui::{backend::Backend, Terminal};

fn tui_todo_internal(
    v: &[&AideTodo],
    terminal: &mut Terminal<impl Backend + Write>,
) -> Result<(), anyhow::Error> {
    let mut widget = TodoStatefulList::new(v, 0);
    loop {
        // draw list
        terminal.draw(|f| aide_common::tui::draw_list(f, &mut widget))?;
        if let Event::Key(key) = crossterm::event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => widget.state = (widget.state + 1) % v.len(),
                KeyCode::Up => {
                    if widget.state == 0 {
                        widget.state = widget.todo_list.len() - 1;
                    } else {
                        widget.state -= 1;
                    }
                }
                _ => (),
            }
        }
    }
    Ok(())
}

fn todo_to_one_line(t: &&AideTodo) -> String {
    let type_symbol = match t.todo_type {
        TodoTypes::Task => "[T]",
        TodoTypes::Daily => "[D]",
        TodoTypes::Weekly => "[W]",
    };
    format!("{} {}", type_symbol, t.name)
}

const INDENTATION: &str = "  ";
fn todo_to_multi_line(t: &&AideTodo) -> String {
    let mut result = todo_to_one_line(t);
    result.push('\n');
    if !t.checklist.is_empty() {
        t.checklist
            .iter()
            .filter(|c| !c.done)
            .for_each(|c| result.push_str(&format!("{INDENTATION}[] {}\n", c.name)));
    }
    if t.due_date.is_some() {
        result.push_str(&format!(
            "{INDENTATION}due date: {}\n",
            t.due_date.to_owned().unwrap()
        ));
    }
    if !t.tags.is_empty() {
        result.push_str(&format!("{INDENTATION}labels: "));
        t.tags
            .iter()
            .for_each(|l| result.push_str(&format!("{l}, ")));
        result.pop().unwrap();
        result.pop().unwrap();
        result.push('\n');
    }
    result
}
