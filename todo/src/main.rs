mod cli;

use aide_proto::v1::{ResultResponse, Todo as AideTodo};
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
                let client = reqwest::Client::new();
                let _res: ResultResponse = client
                    .post(url)
                    .body(serde_json::to_string(&label)?)
                    .send()
                    .await?
                    .json()
                    .await?;
                return Ok(());
            } else {
                assert!(delete);
                let url_path = format!("labels/{name}");
                let url = base_url.join(&url_path)?;
                let client = reqwest::Client::new();
                let _res: ResultResponse = client.delete(url).send().await?.json().await?;
                return Ok(());
            }
        }
    }
    let todos: Vec<AideTodo> = match opt.todo_type {
        Some(cli::TodoTypes::Task) => {
            let url = base_url.join("types/task/todos")?;
            let res = reqwest::get(url).await?;
            res.json().await?
        }
        Some(cli::TodoTypes::Daily) => {
            let url = base_url.join("types/daily/todos")?;
            let res = reqwest::get(url).await?;
            res.json().await?
        }
        Some(cli::TodoTypes::Weekly) => {
            let url = base_url.join("types/weekly/todos")?;
            let res = reqwest::get(url).await?;
            res.json().await?
        }
        Some(cli::TodoTypes::Periodic) => {
            let url = base_url.join("types/daily/todos")?;
            let res = reqwest::get(url).await?;
            let mut todos: Vec<AideTodo> = res.json().await?;
            let url = base_url.join("types/weekly/todos")?;
            let res = reqwest::get(url).await?;
            let mut temp_todos: Vec<AideTodo> = res.json().await?;
            todos.append(&mut temp_todos);
            todos
        }
        None => {
            let url = base_url.join("todos")?;
            let res = reqwest::get(url).await?;
            res.json().await?
        }
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
