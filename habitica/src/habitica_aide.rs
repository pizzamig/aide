const BASE_URL_V3: &str = "https://habitica.com/api/v3/";
use anyhow::anyhow;
use reqwest_pool::ReqwestPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// hashmap with K = tag_id, V = tag_name
type TagCache = Arc<RwLock<HashMap<String, String>>>;

#[derive(Debug, Clone)]
pub struct HabiticaState {
    pub key: String,
    pub user: String,
    pub client_id: String,
    pub pool: ReqwestPool,
    pub tag_cache: TagCache,
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

pub async fn fill_tag_cache(state: HabiticaState) -> Result<(), anyhow::Error> {
    let base_url = reqwest::Url::parse(BASE_URL_V3)?;
    let tags_url = base_url.join("tags")?;
    let handler = state.pool.get_handler().await?;
    let client = handler.get_client();
    let response = client
        .get(tags_url)
        .header("x-client", state.client_id.clone())
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    let resp_tags: super::habitica::RespTags = response.json().await?;
    drop(handler);
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

use super::habitica::{RespDaily, RespTask};

pub async fn get_tasks(
    state: &HabiticaState,
    task_type: UsersTaskTypes,
) -> Result<Vec<aide_proto::v1::todo::Todo>, anyhow::Error> {
    let base_url = reqwest::Url::parse(BASE_URL_V3).unwrap();
    let mut todo_url = base_url.join("tasks/user").unwrap();
    todo_url.set_query(Some(&format!("type={}", task_type)));
    let handler = state.pool.get_handler().await?;
    let client = handler.get_client();
    let response = client
        .get(todo_url)
        .header("x-client", state.client_id.clone())
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    match task_type {
        UsersTaskTypes::Dailys => {
            let resp_daily: RespDaily = response.json().await?;
            drop(handler);
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
            drop(handler);
            let todos: Vec<aide_proto::v1::todo::Todo> =
                resp_task.data.iter().map(|t| t.into()).collect();
            //debug!("received from habitica {} todos", todos.len());
            Ok(todos)
        }
        _ => Ok(vec![]),
    }
}

pub async fn get_all_tasks(
    state: &HabiticaState,
) -> Result<Vec<aide_proto::v1::todo::Todo>, anyhow::Error> {
    let mut result = get_tasks(state, UsersTaskTypes::Todos).await?;
    let dailys = get_tasks(state, UsersTaskTypes::Dailys).await?;
    let daily_tag_id = get_tag_id(state, "daily").await.unwrap_or_default();
    let weekly_tag_id = get_tag_id(state, "weekly").await.unwrap_or_default();
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

pub async fn _create_label(state: &HabiticaState, label: &str) -> Result<(), anyhow::Error> {
    let base_url = reqwest::Url::parse(BASE_URL_V3)?;
    let tags_url = base_url.join("tags")?;
    let handler = state.pool.get_handler().await?;
    let client = handler.get_client();
    let body = super::habitica::CreateTagBody {
        name: label.to_string(),
    };
    let response = client
        .post(tags_url)
        .body(serde_json::to_string(&body).unwrap())
        .header("x-client", state.client_id.clone())
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    let resp: super::habitica::RespCreateTag = response.json().await?;
    drop(handler);
    let mut unlocked_cache = state.tag_cache.write().await;
    unlocked_cache.insert(resp.data.id, resp.data.name);

    Ok(())
}

pub async fn _delete_label(state: &HabiticaState, label: &str) -> Result<(), anyhow::Error> {
    let tag_id = get_tag_id(state, label).await.ok_or_else(|| anyhow!("Unkown label: {}", label))?;
    let base_url = reqwest::Url::parse(BASE_URL_V3)?;
    let tags_url = base_url.join("tags")?;
    let delete_url = tags_url.join(&tag_id)?;
    let handler = state.pool.get_handler().await?;
    let client = handler.get_client();
    let response = client
        .delete(delete_url)
        .header("x-client", state.client_id.clone())
        .header("x-api-user", state.user.clone())
        .header("x-api-key", state.key.clone())
        .send()
        .await?;
    if response.status() == hyper::StatusCode::OK {
        let resp: super::habitica::RespGeneric = response.json().await?;
        if resp.success {
            return Ok(())
        } else {
            return Err(anyhow!("Delete of label {} not successful", label));
        }
    }
    Err(anyhow!("{}: {}", response.status(), response.text().await?))
}