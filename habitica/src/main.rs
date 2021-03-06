mod cli;
mod habitica;
mod habitica_aide;
const HABITICA_KEY_ENV_VAR: &str = "HABITICA_API_KEY";
const HABITICA_USER_ENV_VAR: &str = "HABITICA_API_USER";
const CLIENT_ID_ENV_VAR: &str = "HABITICA_CLIENT_ID";
use aide_common::{healthz, http_404};
use aide_proto::v1::ResultResponse;
use clap::Parser;
use habitica_aide::{get_all_tasks, get_tasks, HabiticaState};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, Uri};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

type TagCache = Arc<RwLock<HashMap<String, String>>>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::try_init()
        .unwrap_or_else(|e| eprintln!("Failed to initialize env_logger [{:?}]", e));
    let opt: cli::Opt = cli::Opt::parse();
    let key = std::env::var(HABITICA_KEY_ENV_VAR)
        .unwrap_or_else(|_| panic!("The env var {} is missing", HABITICA_KEY_ENV_VAR));
    let user = std::env::var(HABITICA_USER_ENV_VAR)
        .unwrap_or_else(|_| panic!("The env var {} is missing", HABITICA_USER_ENV_VAR));
    let client_id = std::env::var(CLIENT_ID_ENV_VAR)
        .unwrap_or_else(|_| panic!("the env var {} is missing", CLIENT_ID_ENV_VAR));
    let builder = reqwest_pool::ReqwestPoolBuilder::new(1).unwrap();
    let pool = builder.build().await;
    let state = HabiticaState {
        key,
        user,
        client_id,
        pool,
        tag_cache: TagCache::default(),
    };

    habitica_aide::fill_tag_cache(state.clone()).await?;
    let service = make_service_fn(|_| {
        let cloned_state = state.clone();
        async {
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
    if req.method() != Method::GET && req.method() != Method::DELETE && req.method() != Method::POST
    {
        return Ok(http_404(
            &"The only methods supported are GET, POST and DELETE",
        ));
    }
    if req.uri().path() == "/healthz" {
        return Ok(healthz());
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
    // /v1/types
    if req.uri().path() == "/v1/types" {
        let data = TodoTypes::VARIANTS.to_vec();
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
    // /v1/types/:type/todos
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
    use habitica_aide::{replace_tag_id, UsersTaskTypes};
    let type_str = path[0];

    if let Ok(todo_type) = TodoTypes::from_str(type_str) {
        use habitica_aide::get_tag_id;
        match todo_type {
            TodoTypes::Task => {
                let todos = get_tasks(&state, UsersTaskTypes::Todos).await?;
                Ok(Response::builder()
                    .body(Body::from(serde_json::to_string(&todos).unwrap()))
                    .unwrap())
            }
            TodoTypes::Daily => {
                if let Some(tag_id) = get_tag_id(&state, "daily").await {
                    let dailys = get_tasks(&state, UsersTaskTypes::Dailys).await?;
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
                } else {
                    Ok(http_404(&"daily label not found"))
                }
            }

            TodoTypes::Weekly => {
                if let Some(tag_id) = get_tag_id(&state, "weekly").await {
                    let dailys = get_tasks(&state, UsersTaskTypes::Dailys).await?;
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
                } else {
                    Ok(http_404(&"weekly label not found"))
                }
            }
        }
    } else {
        return Ok(http_404(&format!("Type not supported: {}", type_str)));
    }
}

async fn labels(req: Request<Body>, state: HabiticaState) -> Result<Response<Body>, anyhow::Error> {
    if req.method() == Method::DELETE {
        delete_label(req, state).await
    } else if req.method() == Method::POST {
        add_label(req, state).await
    } else if req.uri().path() == "/v1/labels" {
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

// POST /v1/labels
// { name: "label_name" }
async fn add_label(
    req: Request<Body>,
    state: HabiticaState,
) -> Result<Response<Body>, anyhow::Error> {
    let path = url_to_pathvec(req.uri());
    if path.len() < 2 {
        return Ok(http_404(&format!("Path has wrong length: {}", path.len())));
    }
    let body = hyper::body::to_bytes(req.into_body()).await?;
    let label: aide_proto::v1::todo::Label = serde_json::from_slice(&body)?;

    habitica_aide::create_label(&state, &label.name).await?;
    let response = ResultResponse { success: true };

    Ok(Response::builder()
        .body(Body::from(serde_json::to_string(&response).unwrap()))
        .unwrap())
}

// DELETE /v1/labels/:label
async fn delete_label(
    req: Request<Body>,
    state: HabiticaState,
) -> Result<Response<Body>, anyhow::Error> {
    let path = url_to_pathvec(req.uri());
    if path.len() < 3 {
        return Ok(http_404(&format!("Path has wrong length: {}", path.len())));
    }
    let label = path[2];
    habitica_aide::delete_label(&state, label).await?;
    let response = ResultResponse { success: true };

    Ok(Response::builder()
        .body(Body::from(serde_json::to_string(&response).unwrap()))
        .unwrap())
}

fn url_to_pathvec(uri: &Uri) -> Vec<&str> {
    uri.path().split('/').skip_while(|x| x.is_empty()).collect()
}
async fn label_todos(
    req: Request<Body>,
    state: HabiticaState,
) -> Result<Response<Body>, anyhow::Error> {
    use habitica_aide::replace_tag_id;
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
    use habitica_aide::get_tag_id;
    if let Some(tag_id) = get_tag_id(&state, path[0]).await {
        let todos = get_all_tasks(&state).await?;
        let mut filtered_todos: Vec<aide_proto::v1::todo::Todo> = todos
            .iter()
            .filter(|d| d.tags.iter().any(|t| t == &tag_id))
            .map(|d| d.to_owned())
            .collect();
        replace_tag_id(&mut filtered_todos, &state).await;
        Ok(Response::builder()
            .body(Body::from(serde_json::to_string(&filtered_todos).unwrap()))
            .unwrap())
    } else {
        Ok(http_404(&format!("label {} not found", path[0])))
    }
}

async fn todos(req: Request<Body>, state: HabiticaState) -> Result<Response<Body>, anyhow::Error> {
    use habitica_aide::replace_tag_id;
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
