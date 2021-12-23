const BASE_URL_V3: &str = "https://habitica.com/api/v3/";
const CLIENT_ID: &str = "3f56b8ab-940c-40d6-8365-1d85b0e3b43d-Testing";
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// hashmap with K = tag_id, V = tag_name
type TagCache = Arc<RwLock<HashMap<String, String>>>;

#[derive(Debug, Clone)]
pub struct HabiticaState {
    pub key: String,
    pub user: String,
    //pub pool: SurfPool,
    pub tag_cache: TagCache,
}

pub async fn fill_tag_cache(state: HabiticaState) -> Result<(), anyhow::Error> {
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
    let resp_tags: super::habitica::RespTags = response.json().await?;
    let mut unlocked_cache = state.tag_cache.write().await;
    for tag in resp_tags.data.iter() {
        unlocked_cache.insert(tag.id.clone(), tag.name.clone());
    }
    Ok(())
}

pub async fn replace_tag_id(todos: &mut [aide_proto::v1::todo::Todo], state: &HabiticaState) {
    for t in todos {
        let tags_unlocked = state.tag_cache.read().await;
        for tag in t.tags.iter_mut() {
            if tags_unlocked.contains_key(tag) {
                *tag = tags_unlocked.get(tag).cloned().unwrap_or_default();
            }
        }
    }
}

pub async fn get_tag_id<'a, 'b>(state: &HabiticaState, label: &str) -> Option<String> {
    let cache = state.tag_cache.read().await;
    cache
        .iter()
        .find(|(_k, v)| *v == label)
        .map(|(k, _)| k.clone())
}

use super::habitica::{RespDaily, RespTask, UsersTaskTypes};

pub async fn get_tasks(
    state: &HabiticaState,
    task_type: UsersTaskTypes,
) -> Result<Vec<aide_proto::v1::todo::Todo>, anyhow::Error> {
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
            let resp_daily: RespDaily = response.json().await?;
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
            let resp_task: RespTask = response.json().await?;
            let todos: Vec<aide_proto::v1::todo::Todo> =
                resp_task.data.iter().map(|t| t.into()).collect();
            //debug!("received from habitica {} todos", todos.len());
            Ok(todos)
        }
        _ => Ok(vec![]),
    }
}
