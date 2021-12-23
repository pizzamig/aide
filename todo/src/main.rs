mod cli;

use aide_proto::v1::Todo as AideTodo;
use clap::Parser;

fn print_todo(t: &&AideTodo) {
    let type_symbol = match t.todo_type {
        aide_proto::v1::todo::TodoTypes::Task => "[T]",
        aide_proto::v1::todo::TodoTypes::Daily => "[D]",
        aide_proto::v1::todo::TodoTypes::Weekly => "[W]",
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

#[async_std::main]
async fn main() -> surf::Result<()> {
    let opt: cli::Opt = cli::Opt::parse();
    let todos: Vec<AideTodo> = if opt.todo_type.is_some() {
        match opt.todo_type.unwrap() {
            cli::TodoTypes::Task => {
                let mut res = surf::get("http://localhost:9099/v1/types/task/todos").await?;
                res.body_json().await?
            }
            cli::TodoTypes::Daily => {
                let mut res = surf::get("http://localhost:9099/v1/types/daily/todos").await?;
                res.body_json().await?
            }
            cli::TodoTypes::Weekly => {
                let mut res = surf::get("http://localhost:9099/v1/types/weekly/todos").await?;
                res.body_json().await?
            }
            cli::TodoTypes::Periodic => {
                let mut res = surf::get("http://localhost:9099/v1/types/daily/todos").await?;
                let mut todos: Vec<AideTodo> = res.body_json().await?;
                let mut res = surf::get("http://localhost:9099/v1/types/weekly/todos").await?;
                let mut temp_todos: Vec<AideTodo> = res.body_json().await?;
                todos.append(&mut temp_todos);
                todos
            }
        }
    } else {
        let mut res = surf::get("http://localhost:9099/v1/todos").await?;
        res.body_json().await?
    };
    let temp_todos: Vec<&AideTodo> = if let Some(label) = opt.label {
        todos.iter().filter(|t| t.tags.contains(&label)).collect()
    } else {
        todos.iter().collect()
    };

    temp_todos.iter().for_each(print_todo);
    println!("total: {}", get_todos_count(&temp_todos));

    Ok(())
}
