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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt: cli::Opt = cli::Opt::parse();
    let base_url = reqwest::Url::parse(&format!(
        "http://{}:{}",
        opt.common_opt.host_addr, opt.common_opt.port
    ))?;
    let todos: Vec<AideTodo> = if opt.todo_type.is_some() {
        match opt.todo_type.unwrap() {
            cli::TodoTypes::Task => {
                let url = base_url.join("v1/types/task/todos")?;
                let res = reqwest::get(url).await?;
                res.json().await?
            }
            cli::TodoTypes::Daily => {
                let url = base_url.join("v1/types/daily/todos")?;
                let res = reqwest::get(url).await?;
                res.json().await?
            }
            cli::TodoTypes::Weekly => {
                let url = base_url.join("v1/types/weekly/todos")?;
                let res = reqwest::get(url).await?;
                res.json().await?
            }
            cli::TodoTypes::Periodic => {
                let url = base_url.join("v1/types/daily/todos")?;
                let res = reqwest::get(url).await?;
                let mut todos: Vec<AideTodo> = res.json().await?;
                let url = base_url.join("v1/types/weekly/todos")?;
                let res = reqwest::get(url).await?;
                let mut temp_todos: Vec<AideTodo> = res.json().await?;
                todos.append(&mut temp_todos);
                todos
            }
        }
    } else {
        let url = base_url.join("v1/todos")?;
        let res = reqwest::get(url).await?;
        res.json().await?
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
