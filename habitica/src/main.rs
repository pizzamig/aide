mod cli;
mod habitica;
const HABITICA_KEY_ENV_VAR: &str = "HABITICA_API_KEY";
const HABITICA_USER_ENV_VAR: &str = "HABITICA_API_USER";
const BASE_URL_V3: &str = "https://habitica.com/api/v3/";
const CLIENT_ID: &str = "3f56b8ab-940c-40d6-8365-1d85b0e3b43d-Testing";
use aide_common::http_404;
use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

type TagCache = Arc<RwLock<HashMap<String, String>>>;

#[derive(Debug, Clone)]
struct HabiticaState {
    key: String,
    user: String,
    //pool: SurfPool,
    tag_cache: Arc<RwLock<HashMap<String, String>>>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt: cli::Opt = cli::Opt::parse();
    // TODO log
    //tide::log::with_level(tide::log::LevelFilter::Debug);
    let key = std::env::var(HABITICA_KEY_ENV_VAR)
        .unwrap_or_else(|_| panic!("The env var {} is missing", HABITICA_KEY_ENV_VAR));
    let user = std::env::var(HABITICA_USER_ENV_VAR)
        .unwrap_or_else(|_| panic!("The env var {} is missing", HABITICA_USER_ENV_VAR));
    // TODO connection pool
    //let pool = surf_pool::SurfPoolBuilder::new(1).unwrap().build().await;
    let state = HabiticaState {
        key,
        user,
        //pool,
        tag_cache: TagCache::default(),
    };

    fill_tag_cache(state.clone()).await?;
    let service = make_service_fn(|_| {
        let cloned_state = state.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                habitica_svc(req, cloned_state.clone())
            }))
        }
    });
    if opt.common_opt.registration {
        todo!("Registration not implemented yet!")
    }

    let socket_addr = std::net::SocketAddr::new(opt.common_opt.host_addr, opt.common_opt.port);
    let server = Server::bind(&socket_addr).serve(service);
    server.await?;

    Ok(())
}

async fn habitica_svc(
    req: Request<Body>,
    state: HabiticaState,
) -> Result<Response<Body>, anyhow::Error> {
    //app.at("/v1/types").get(types);
    //app.at("/v1/types/:type/todos").get(type_todos);
    //app.at("/v1/labels").get(labels);
    //app.at("/v1/labels/:label/todos").get(label_todos);
    //app.at("/v1/todos").get(todos);
    if req.method() != Method::GET {
        return Ok(http_404(&"The only method supported is GET"));
    }
    if !req.uri().path().starts_with("/v1") {
        return Ok(http_404(&"Invalid path"));
    }
    if req.uri().path().starts_with("/v1/types") {
        return types(req, state).await;
    } else if req.uri().path().starts_with("/v1/labels") {
        return labels(req, state).await;
    } else if req.uri().path().starts_with("/v1/todos") {
        return todos(req, state).await;
    }
    Ok(http_404(&""))
}
async fn types(req: Request<Body>, state: HabiticaState) -> Result<Response<Body>, anyhow::Error> {
    use aide_proto::v1::{todo::TodoTypes, DataResponseRef};
    use strum::VariantNames;
    if req.uri().path() == "/v1/types" {
        let data: Vec<_> = TodoTypes::VARIANTS.iter().copied().collect();
        let response = DataResponseRef { data };
        Ok(Response::new(Body::from(
            serde_json::to_string(&response).unwrap(),
        )))
    } else {
        type_todos(req, state).await
    }
}

async fn type_todos(
    req: Request<Body>,
    state: HabiticaState,
) -> Result<Response<Body>, anyhow::Error> {
    let path = req
        .uri()
        .path()
        .split('/')
        .skip_while(|x| x.is_empty())
        .skip(2)
        .collect::<Vec<_>>();
    if path.len() < 2 {
        return Ok(http_404(&format!("Path has wrong length: {}", path.len())));
    }
    if path[1] != "todos" {
        return Ok(http_404(&format!("Unrecognized word {}", path[1])));
    }
    use aide_proto::v1::todo::TodoTypes;
    let type_str = path[0];
    if let Ok(todo_type) = TodoTypes::from_str(type_str) {
        match todo_type {
            TodoTypes::Task => {
                let todos = get_tasks(&state, habitica::UsersTaskTypes::Todos).await?;
                Ok(Response::builder()
                    .body(Body::from(serde_json::to_string(&todos).unwrap()))
                    .unwrap())
            }
            TodoTypes::Daily => {
                let label = "daily";
                let unlocked_cache = state.tag_cache.read().await;
                // TODO: this unwrap can fail!
                let tag_id = unlocked_cache
                    .iter()
                    .find(|(_k, v)| v == &label)
                    .map(|(k, _)| k.to_string())
                    .unwrap();
                drop(unlocked_cache);
                let dailys = get_tasks(&state, habitica::UsersTaskTypes::Dailys).await?;
                let mut filtered_dailys: Vec<aide_proto::v1::todo::Todo> = dailys
                    .iter()
                    .filter(|d| d.tags.iter().any(|t| t == &tag_id))
                    .map(|d| {
                        let mut owned_daily = d.to_owned();
                        owned_daily.todo_type = todo_type;
                        owned_daily
                    })
                    .collect();
                replace_tag_id(&mut filtered_dailys, &state).await;
                Ok(Response::builder()
                    .body(Body::from(serde_json::to_string(&filtered_dailys).unwrap()))
                    .unwrap())
            }

            TodoTypes::Weekly => {
                let label = "weekly";
                let unlocked_cache = state.tag_cache.read().await;
                let tag_id = unlocked_cache
                    .iter()
                    .find(|(_k, v)| v == &label)
                    .map(|(k, _)| k.to_string())
                    .unwrap();
                drop(unlocked_cache);
                let dailys = get_tasks(&state, habitica::UsersTaskTypes::Dailys).await?;
                let mut filtered_dailys: Vec<aide_proto::v1::todo::Todo> = dailys
                    .iter()
                    .filter(|d| d.tags.iter().any(|t| t == &tag_id))
                    .map(|d| {
                        let mut owned_daily = d.to_owned();
                        owned_daily.todo_type = todo_type;
                        owned_daily
                    })
                    .collect();
                replace_tag_id(&mut filtered_dailys, &state).await;
                Ok(Response::builder()
                    .body(Body::from(serde_json::to_string(&filtered_dailys).unwrap()))
                    .unwrap())
            }
        }
    } else {
        return Ok(http_404(&format!("Type not supported: {}", type_str)));
    }
}

async fn get_tasks(
    state: &HabiticaState,
    task_type: habitica::UsersTaskTypes,
) -> Result<Vec<aide_proto::v1::todo::Todo>, anyhow::Error> {
    use habitica::UsersTaskTypes;
    let base_url = reqwest::Url::parse(BASE_URL_V3).unwrap();
    let mut todo_url = base_url.join("tasks/user").unwrap();
    todo_url.set_query(Some(&format!("type={}", task_type.to_string())));
    let client = reqwest::Client::new();
    let response = client
        .get(todo_url)
        .header("x-client", CLIENT_ID)
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    match task_type {
        UsersTaskTypes::Dailys => {
            let resp_daily: habitica::RespDaily = response.json().await?;
            let todos: Vec<aide_proto::v1::todo::Todo> = resp_daily
                .data
                .iter()
                .filter(|d| d.is_due())
                .map(|t| t.into())
                .collect();
            //debug!("received from habitica {} todos", todos.len());
            Ok(todos)
        }
        UsersTaskTypes::Todos => {
            let resp_task: habitica::RespTask = response.json().await?;
            let todos: Vec<aide_proto::v1::todo::Todo> =
                resp_task.data.iter().map(|t| t.into()).collect();
            //debug!("received from habitica {} todos", todos.len());
            Ok(todos)
        }
        _ => Ok(vec![]),
    }
}

async fn labels(req: Request<Body>, state: HabiticaState) -> Result<Response<Body>, anyhow::Error> {
    if req.uri().path() == "/v1/labels" {
        use aide_proto::v1::{todo::TodoTypes, DataResponse};
        let unlocked_cache = state.tag_cache.read().await;
        let data: Vec<String> = unlocked_cache
            .values()
            .filter(|l| TodoTypes::from_str(l).is_err())
            .map(|l| l.to_string())
            .collect();
        drop(unlocked_cache);
        let response = DataResponse { data };
        Ok(Response::builder()
            .body(Body::from(serde_json::to_string(&response).unwrap()))
            .unwrap())
    } else {
        label_todos(req, state).await
    }
}

async fn label_todos(
    req: Request<Body>,
    state: HabiticaState,
) -> Result<Response<Body>, anyhow::Error> {
    let path = req
        .uri()
        .path()
        .split('/')
        .skip_while(|x| x.is_empty())
        .skip(2)
        .collect::<Vec<_>>();
    if path.len() < 2 {
        return Ok(http_404(&format!("Path has wrong length: {}", path.len())));
    }
    if path[1] != "todos" {
        return Ok(http_404(&format!("Unrecognized word {}", path[1])));
    }
    let label = path[0];
    let todos = get_all_tasks(&state).await?;
    let unlocked_cache = state.tag_cache.read().await;
    // the label can be missing
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
    replace_tag_id(&mut filtered_todos, &state).await;
    Ok(Response::builder()
        .body(Body::from(serde_json::to_string(&filtered_todos).unwrap()))
        .unwrap())
}

async fn todos(req: Request<Body>, state: HabiticaState) -> Result<Response<Body>, anyhow::Error> {
    if req.uri().path() == "/v1/todos" {
        let mut todos = get_all_tasks(&state).await?;
        replace_tag_id(&mut todos, &state).await;
        Ok(Response::builder()
            .body(Body::from(serde_json::to_string(&todos).unwrap()))
            .unwrap())
    } else {
        Ok(http_404(&format!(
            "path not recognized: {}",
            req.uri().path()
        )))
    }
}

//async fn get_todo(state: &HabiticaState) -> tide::Result {
//let mut todos = get_tasks_with_type(state, "todos").await?;
//let dailys = get_tasks_with_type(state, "dailys").await?;
//todos.extend(dailys);
//let get_todo_reply = aide_proto::v1::GetTodoReply { todo: todos };
//let todo_reply = aide_proto::v1::TodoReplyType::GetTodo(get_todo_reply);
//Ok(serde_json::to_string(&todo_reply).unwrap().into())
//}

async fn get_all_tasks(
    state: &HabiticaState,
) -> Result<Vec<aide_proto::v1::todo::Todo>, anyhow::Error> {
    let mut result = get_tasks(state, habitica::UsersTaskTypes::Todos).await?;
    let dailys = get_tasks(state, habitica::UsersTaskTypes::Dailys).await?;
    let unlocked_cache = state.tag_cache.read().await;
    // TODO: daily or weekly can be missed!
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

//async fn _get_tasks_with_type(
//state: &HabiticaState,
//ttype: &str,
//) -> tide::Result<Vec<aide_proto::v1::todo::Todo>> {
//let base_url = surf::Url::parse(BASE_URL_V3)?;
//let mut todo_url = base_url.join("tasks/user")?;
//todo_url.set_query(Some(&format!("type={}", ttype)));
//let client = surf::Client::new();
//let mut res = client
//.get(todo_url)
//.header("x-client", CLIENT_ID)
//.header("x-api-user", state.user.clone())
//.header("x-api-key", state.key.clone())
//.send()
//.await?;
//if ttype == "dailys" {
//let resp_daily: habitica::RespDaily = res.body_json().await?;
//let mut todos: Vec<aide_proto::v1::todo::Todo> = resp_daily
//.data
//.iter()
//.inspect(|d| {
//dbg!(d);
//})
//.filter(|d| d.is_due())
//.map(|t| t.into())
//.collect();
//debug!("received from habitica {} todos", todos.len());
//replace_tag_id(&mut todos, state).await;
//Ok(todos)
//} else {
//// todos
//let resp_task: habitica::RespTask = res.body_json().await?;
//let mut todos: Vec<aide_proto::v1::todo::Todo> =
//resp_task.data.iter().map(|t| t.into()).collect();
//debug!("received from habitica {} todos", todos.len());
//replace_tag_id(&mut todos, state).await;
//Ok(todos)
//}
//}

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
async fn fill_tag_cache(state: HabiticaState) -> Result<(), anyhow::Error> {
    let base_url = surf::Url::parse(BASE_URL_V3)?;
    let tags_url = base_url.join("tags")?;
    let client = reqwest::Client::new();
    let response = client
        .get(tags_url)
        .header("x-client", CLIENT_ID)
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    let resp_tags: habitica::RespTags = response.json().await?;
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
