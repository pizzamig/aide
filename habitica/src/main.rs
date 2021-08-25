mod cli;
mod habitica;
const HABITICA_PORT: u16 = 9099;
const HABITICA_KEY_ENV_VAR: &str = "HABITICA_API_KEY";
const HABITICA_USER_ENV_VAR: &str = "HABITICA_API_USER";
use async_std::sync::{Arc, RwLock};
use clap::Clap;
use std::collections::HashMap;
use std::str::FromStr;
use tide::log::*;

#[derive(Debug, Default, Clone)]
struct HabiticaState {
    key: String,
    user: String,
    tag_cache: Arc<RwLock<HashMap<String, String>>>,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let opt: cli::Opt = cli::Opt::parse();
    tide::log::with_level(tide::log::LevelFilter::Debug);
    let mut state = HabiticaState::default();
    if let Ok(key) = std::env::var(HABITICA_KEY_ENV_VAR) {
        state.key = key;
    } else {
        panic!("The env var {} is missing", HABITICA_KEY_ENV_VAR);
    }
    if let Ok(user) = std::env::var(HABITICA_USER_ENV_VAR) {
        state.user = user;
    } else {
        panic!("The env var {} is missing", HABITICA_USER_ENV_VAR);
    }

    let fill_cache_state = state.clone();
    let mut app = tide::with_state(state);
    if opt.registration {
        todo!("Registration not implemented yet!")
        //async_std::task::spawn(async { subscribe_module().await.unwrap_or(()) });
    }
    async_std::task::spawn(async move { fill_tag_cache(fill_cache_state).await.unwrap_or(()) });

    app.at("/v1/types").get(types);
    app.at("/v1/types/:type/todos").get(type_todos);
    app.at("/v1/labels").get(labels);
    app.at("/v1/labels/:label/todos").get(label_todos);
    app.at("/v1/todos").get(todos);
    let binded = format!("0.0.0.0:{}", HABITICA_PORT);
    app.listen(binded).await?;
    Ok(())
}

async fn types(_req: tide::Request<HabiticaState>) -> tide::Result<String> {
    use aide_proto::v1::{todo::TodoTypes, DataResponse};
    use strum::VariantNames;
    let data: Vec<String> = TodoTypes::VARIANTS.iter().map(|t| t.to_string()).collect();
    let response = DataResponse { data };
    Ok(serde_json::to_string(&response).unwrap())
}

async fn type_todos(req: tide::Request<HabiticaState>) -> tide::Result<tide::Response> {
    use aide_proto::v1::todo::TodoTypes;
    let type_str = req.param("type")?;
    if let Ok(todo_type) = TodoTypes::from_str(type_str) {
        if todo_type == TodoTypes::Task {
            let todos = get_tasks(req.state(), habitica::UsersTaskTypes::Todos).await?;
            Ok(tide::Response::builder(200)
                .body(serde_json::to_string(&todos).unwrap())
                .build())
        } else {
            let label = match todo_type {
                TodoTypes::Daily => "daily",
                TodoTypes::Weekly => "weekly",
                _ => "",
            };
            let unlocked_cache = req.state().tag_cache.read().await;
            let tag_id = unlocked_cache
                .iter()
                .find(|(_k, v)| v == &label)
                .map(|(k, _)| k.to_string())
                .unwrap();
            drop(unlocked_cache);
            let dailys = get_tasks(req.state(), habitica::UsersTaskTypes::Dailys).await?;
            let mut filtered_dailys: Vec<aide_proto::v1::todo::Todo> = dailys
                .iter()
                .filter(|d| d.tags.iter().any(|t| t == &tag_id))
                .map(|d| {
                    let mut owned_daily = d.to_owned();
                    owned_daily.todo_type = todo_type;
                    owned_daily
                })
                .collect();
            replace_tag_id(&mut filtered_dailys, req.state()).await;
            Ok(tide::Response::builder(200)
                .body(serde_json::to_string(&filtered_dailys).unwrap())
                .build())
        }
    } else {
        Ok(tide::Response::builder(404)
            .body("TodoTypeNotFound")
            .build())
    }
}

async fn labels(req: tide::Request<HabiticaState>) -> tide::Result<String> {
    use aide_proto::v1::{todo::TodoTypes, DataResponse};
    let unlocked_cache = req.state().tag_cache.read().await;
    let data: Vec<String> = unlocked_cache
        .values()
        .filter(|l| TodoTypes::from_str(l).is_err())
        .map(|l| l.to_string())
        .collect();
    drop(unlocked_cache);
    let response = DataResponse { data };
    Ok(serde_json::to_string(&response).unwrap())
}

async fn label_todos(req: tide::Request<HabiticaState>) -> tide::Result<String> {
    let label = req.param("label")?;
    let todos = get_all_tasks(req.state()).await?;
    let unlocked_cache = req.state().tag_cache.read().await;
    let tag_id = unlocked_cache
        .iter()
        .find(|(_k, v)| v == &label)
        .map(|(k, _)| k.to_string())
        .unwrap();
    drop(unlocked_cache);
    let mut filtered_todos: Vec<aide_proto::v1::todo::Todo> = todos
        .iter()
        .filter(|d| d.tags.iter().any(|t| t == &tag_id))
        .map(|d| d.to_owned())
        .collect();
    replace_tag_id(&mut filtered_todos, req.state()).await;
    Ok(serde_json::to_string(&filtered_todos).unwrap())
}

async fn todos(req: tide::Request<HabiticaState>) -> tide::Result<String> {
    let mut todos = get_all_tasks(req.state()).await?;
    replace_tag_id(&mut todos, req.state()).await;
    Ok(serde_json::to_string(&todos).unwrap())
}

const BASE_URL_V3: &str = "https://habitica.com/api/v3/";
const CLIENT_ID: &str = "3f56b8ab-940c-40d6-8365-1d85b0e3b43d-Testing";
//async fn get_todo(state: &HabiticaState) -> tide::Result {
//let mut todos = get_tasks_with_type(state, "todos").await?;
//let dailys = get_tasks_with_type(state, "dailys").await?;
//todos.extend(dailys);
//let get_todo_reply = aide_proto::v1::GetTodoReply { todo: todos };
//let todo_reply = aide_proto::v1::TodoReplyType::GetTodo(get_todo_reply);
//Ok(serde_json::to_string(&todo_reply).unwrap().into())
//}

async fn get_tasks(
    state: &HabiticaState,
    task_type: habitica::UsersTaskTypes,
) -> tide::Result<Vec<aide_proto::v1::todo::Todo>> {
    use habitica::UsersTaskTypes;
    let base_url = surf::Url::parse(BASE_URL_V3)?;
    let mut todo_url = base_url.join("tasks/user")?;
    todo_url.set_query(Some(&format!("type={}", task_type.to_string())));
    let client = surf::Client::new();
    let mut res = client
        .get(todo_url)
        .header("x-client", CLIENT_ID)
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    match task_type {
        UsersTaskTypes::Dailys => {
            let resp_daily: habitica::RespDaily = res.body_json().await?;
            let todos: Vec<aide_proto::v1::todo::Todo> = resp_daily
                .data
                .iter()
                .filter(|d| d.is_due())
                .map(|t| t.into())
                .collect();
            debug!("received from habitica {} todos", todos.len());
            Ok(todos)
        }
        UsersTaskTypes::Todos => {
            let resp_task: habitica::RespTask = res.body_json().await?;
            let todos: Vec<aide_proto::v1::todo::Todo> =
                resp_task.data.iter().map(|t| t.into()).collect();
            debug!("received from habitica {} todos", todos.len());
            Ok(todos)
        }
        _ => Ok(Vec::new()),
    }
}

async fn get_all_tasks(state: &HabiticaState) -> tide::Result<Vec<aide_proto::v1::todo::Todo>> {
    let mut result = get_tasks(state, habitica::UsersTaskTypes::Todos).await?;
    let dailys = get_tasks(state, habitica::UsersTaskTypes::Dailys).await?;
    let unlocked_cache = state.tag_cache.read().await;
    let daily_tag_id = unlocked_cache
        .iter()
        .find(|(_k, v)| v.as_str() == "daily")
        .map(|(k, _)| k.to_string())
        .unwrap();
    let weekly_tag_id = unlocked_cache
        .iter()
        .find(|(_k, v)| v.as_str() == "weekly")
        .map(|(k, _)| k.to_string())
        .unwrap();
    drop(unlocked_cache);
    let mut filtered_dailys: Vec<aide_proto::v1::todo::Todo> = dailys
        .iter()
        .filter(|d| {
            d.tags
                .iter()
                .any(|t| t == &daily_tag_id || t == &weekly_tag_id)
        })
        .map(|d| {
            let mut new_daily = d.to_owned();
            if new_daily.tags.contains(&weekly_tag_id) {
                new_daily.todo_type = aide_proto::v1::todo::TodoTypes::Weekly;
            }
            new_daily
        })
        .collect();
    result.append(&mut filtered_dailys);
    Ok(result)
}

async fn _get_tasks_with_type(
    state: &HabiticaState,
    ttype: &str,
) -> tide::Result<Vec<aide_proto::v1::todo::Todo>> {
    let base_url = surf::Url::parse(BASE_URL_V3)?;
    let mut todo_url = base_url.join("tasks/user")?;
    todo_url.set_query(Some(&format!("type={}", ttype)));
    let client = surf::Client::new();
    let mut res = client
        .get(todo_url)
        .header("x-client", CLIENT_ID)
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    if ttype == "dailys" {
        let resp_daily: habitica::RespDaily = res.body_json().await?;
        let mut todos: Vec<aide_proto::v1::todo::Todo> = resp_daily
            .data
            .iter()
            .inspect(|d| {
                dbg!(d);
            })
            .filter(|d| d.is_due())
            .map(|t| t.into())
            .collect();
        debug!("received from habitica {} todos", todos.len());
        replace_tag_id(&mut todos, state).await;
        Ok(todos)
    } else {
        // todos
        let resp_task: habitica::RespTask = res.body_json().await?;
        let mut todos: Vec<aide_proto::v1::todo::Todo> =
            resp_task.data.iter().map(|t| t.into()).collect();
        debug!("received from habitica {} todos", todos.len());
        replace_tag_id(&mut todos, state).await;
        Ok(todos)
    }
}

async fn replace_tag_id(todos: &mut [aide_proto::v1::todo::Todo], state: &HabiticaState) {
    for t in todos {
        let tags_unlocked = state.tag_cache.read().await;
        for tag in t.tags.iter_mut() {
            if tags_unlocked.contains_key(tag) {
                *tag = tags_unlocked.get(tag).unwrap().clone();
            }
        }
    }
}
async fn fill_tag_cache(state: HabiticaState) -> Result<(), Box<dyn std::error::Error>> {
    let base_url = surf::Url::parse(BASE_URL_V3)?;
    let tags_url = base_url.join("tags")?;
    let client = surf::Client::new();
    let mut res = client
        .get(tags_url)
        .header("x-client", CLIENT_ID)
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    let resp_tags: habitica::RespTags = res.body_json().await?;
    let mut unlocked_cache = state.tag_cache.write().await;
    for tag in resp_tags.data.iter() {
        unlocked_cache.insert(tag.id.clone(), tag.name.clone());
    }
    Ok(())
}

//async fn subscribe_module() -> Result<(), Box<dyn std::error::Error>> {
//let web_hook = format!("http://0.0.0.0:{}", HABITICA_PORT);
//let payload = aide_proto::v1::module::Module {
//name: "habitica-plugin".to_string(),
//kind: aide_proto::v1::module::ModuleType::Todo,
//web_hook,
//};
//let req = surf::post("http://localhost:9090/v1/modules")
//.body(surf::Body::from_json(
//&aide_proto::v1::ModuleRequestType::Subscribe(payload),
//)?)
//.await?;
//if req.status() == surf::StatusCode::Ok {
//return Ok(());
//}
//panic!("Connection failed");
//}
