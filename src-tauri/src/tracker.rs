// src-tauri/src/tracker.rs

use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub struct AppState {
    pub membership_id: Mutex<String>,
    pub membership_type: Mutex<i32>,
    pub latest_snapshot_json: Mutex<Option<String>>,
    pub latest_raids_json: Mutex<Option<String>>,
    pub poller_handle: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    pub profile_version: Mutex<u64>,
    pub cached_activity_name_hash: Mutex<Option<i64>>,
    pub cached_activity_name: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            membership_id: Mutex::new(String::new()),
            membership_type: Mutex::new(0),
            latest_snapshot_json: Mutex::new(None),
            latest_raids_json: Mutex::new(None),
            poller_handle: Mutex::new(None),
            profile_version: Mutex::new(0),
            cached_activity_name_hash: Mutex::new(None),
            cached_activity_name: Mutex::new(None),
        }
    }
}

fn get_daily_reset_utc(now: DateTime<Utc>) -> DateTime<Utc> {
    let today_reset = now.date_naive().and_hms_opt(17, 0, 0).unwrap().and_utc();

    if now >= today_reset {
        today_reset
    } else {
        today_reset - Duration::days(1)
    }
}

fn parse_rfc3339_to_utc(value: &str) -> Option<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

async fn fetch_activity_name(
    client: &reqwest::Client,
    api_key: &str,
    activity_hash: i64,
) -> String {
    let url = format!(
        "https://www.bungie.net/Platform/Destiny2/Manifest/DestinyActivityDefinition/{}/",
        activity_hash
    );

    let response = client.get(url).header("X-API-Key", api_key).send().await;

    match response {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => json["Response"]["displayProperties"]["name"]
                    .as_str()
                    .unwrap_or("Unknown Activity")
                    .to_string(),
                Err(_) => "Unknown Activity".to_string(),
            },
            Err(_) => "Unknown Activity".to_string(),
        },
        Err(_) => "Unknown Activity".to_string(),
    }
}
async fn resolve_current_activity_name(
    state: &AppState,
    client: &reqwest::Client,
    api_key: &str,
    activity_hash: Option<i64>,
) -> Option<String> {
    let hash = activity_hash.unwrap_or(0);
    if hash <= 0 {
        return None;
    }

    {
        let cached_hash = *state.cached_activity_name_hash.lock().unwrap();
        let cached_name = state.cached_activity_name.lock().unwrap().clone();

        if cached_hash == Some(hash) {
            if let Some(name) = cached_name {
                return Some(name);
            }
        }
    }

    let fetched_name = fetch_activity_name(client, api_key, hash).await;

    {
        let mut cached_hash = state.cached_activity_name_hash.lock().unwrap();
        let mut cached_name = state.cached_activity_name.lock().unwrap();

        *cached_hash = Some(hash);
        *cached_name = Some(fetched_name.clone());
    }

    Some(fetched_name)
}

async fn build_tracker_snapshot_json(
    state: &AppState,
    membership_id: &str,
    membership_type: i32,
) -> Result<String, String> {
    if membership_id.is_empty() {
        return Err("Nenhum perfil ativo definido.".into());
    }

    let api_key = std::env::var("BUNGIE_API_KEY")
        .map_err(|_| "API key não encontrada. Verifica a variável de ambiente.")?;

    let client = reqwest::Client::new();

    let profile_url = format!(
        "https://www.bungie.net/Platform/Destiny2/{}/Profile/{}/?components=100,200,204",
        membership_type, membership_id
    );

    let profile_text = client
        .get(profile_url)
        .header("X-API-Key", &api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let profile_json: Value = serde_json::from_str(&profile_text)
        .map_err(|e| format!("Erro a ler profile JSON: {}", e))?;

    let profile_data = &profile_json["Response"]["profile"]["data"];
    let characters_data = profile_json["Response"]["characters"]["data"]
        .as_object()
        .ok_or("Não foi possível encontrar characters.data no profile.")?;

    let character_activities_data = profile_json["Response"]["characterActivities"]["data"]
        .as_object()
        .ok_or("Não foi possível encontrar characterActivities.data no profile.")?;

    let profile_last_played = profile_data["dateLastPlayed"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let mut latest_character_id = String::new();
    let mut latest_character_last_played = String::new();

    for (character_id, character_data) in characters_data {
        let character_last_played = character_data["dateLastPlayed"].as_str().unwrap_or("");

        if character_last_played > latest_character_last_played.as_str() {
            latest_character_last_played = character_last_played.to_string();
            latest_character_id = character_id.to_string();
        }
    }

    if latest_character_id.is_empty() {
        return Err("Não foi possível determinar a personagem mais recente.".into());
    }

    let mut selected_character_id = latest_character_id.clone();
    let mut selected_character_last_played = latest_character_last_played.clone();

    let mut current_activity_hash: Option<i64> = None;
    let mut current_activity_mode_hash: Option<i64> = None;
    let mut current_activity_mode_type: Option<i64> = None;
    let mut activity_started_at: Option<String> = None;

    let mut newest_started_at: Option<DateTime<Utc>> = None;

    for (character_id, activity_data) in character_activities_data {
        let activity_hash = activity_data["currentActivityHash"].as_i64().unwrap_or(0);
        let activity_mode_hash = activity_data["currentActivityModeHash"].as_i64();
        let activity_mode_type = activity_data["currentActivityModeType"].as_i64();
        let started_at_str = activity_data["dateActivityStarted"].as_str().unwrap_or("");
        let started_at = parse_rfc3339_to_utc(started_at_str);

        if activity_hash <= 0 {
            continue;
        }

        let should_replace = match (newest_started_at, started_at) {
            (None, Some(_)) => true,
            (Some(_), None) => false,
            (Some(current_best), Some(candidate)) => candidate > current_best,
            (None, None) => false,
        };

        if should_replace {
            newest_started_at = started_at;
            selected_character_id = character_id.to_string();
            selected_character_last_played = characters_data
                .get(character_id)
                .and_then(|c| c["dateLastPlayed"].as_str())
                .unwrap_or("")
                .to_string();

            current_activity_hash = Some(activity_hash);
            current_activity_mode_hash = activity_mode_hash;
            current_activity_mode_type = activity_mode_type;
            activity_started_at = if started_at_str.is_empty() {
                None
            } else {
                Some(started_at_str.to_string())
            };
        }
    }

    if current_activity_hash.is_none() {
        if let Some(activity_data) = character_activities_data.get(&latest_character_id) {
            let fallback_hash = activity_data["currentActivityHash"].as_i64().unwrap_or(0);
            let fallback_mode_hash = activity_data["currentActivityModeHash"].as_i64();
            let fallback_mode_type = activity_data["currentActivityModeType"].as_i64();

            if fallback_hash > 0 {
                selected_character_id = latest_character_id.clone();
                selected_character_last_played = latest_character_last_played.clone();
                current_activity_hash = Some(fallback_hash);
                current_activity_mode_hash = fallback_mode_hash;
                current_activity_mode_type = fallback_mode_type;

                let started_at_str = activity_data["dateActivityStarted"].as_str().unwrap_or("");
                if !started_at_str.is_empty() {
                    activity_started_at = Some(started_at_str.to_string());
                }
            }
        }
    }

    let now = Utc::now();

    let elapsed_seconds = activity_started_at
        .as_deref()
        .and_then(parse_rfc3339_to_utc)
        .map(|started| (now - started).num_seconds().max(0))
        .unwrap_or(0);

    let timer_source = if activity_started_at.is_some() && current_activity_hash.unwrap_or(0) > 0 {
        Some("dateActivityStarted".to_string())
    } else {
        None
    };

    let activity_instance_key = if current_activity_hash.unwrap_or(0) > 0 {
        format!(
            "{}:{}:{}:{}",
            selected_character_id,
            current_activity_hash.unwrap_or(0),
            current_activity_mode_type.unwrap_or(0),
            activity_started_at.clone().unwrap_or_default()
        )
    } else {
        String::new()
    };

    let activity_name =
        resolve_current_activity_name(state, &client, &api_key, current_activity_hash).await;
    let result = serde_json::json!({
        "membershipId": membership_id,
        "membershipType": membership_type,
        "characterId": selected_character_id,
        "profileLastPlayed": profile_last_played,
        "characterLastPlayed": selected_character_last_played,
        "currentActivityHash": current_activity_hash,
        "currentActivityModeHash": current_activity_mode_hash,
        "currentActivityModeType": current_activity_mode_type,
        "isInActivity": current_activity_hash.unwrap_or(0) > 0,
        "activityStartedAt": activity_started_at,
        "activityInstanceKey": activity_instance_key,
        "activityName": activity_name,
        "elapsedSeconds": elapsed_seconds,
        "timerSource": timer_source,
        "refreshedAt": now.to_rfc3339()
    });

    Ok(result.to_string())
}

async fn build_todays_raid_history_json(
    membership_id: &str,
    membership_type: i32,
) -> Result<String, String> {
    if membership_id.is_empty() {
        return Err("Nenhum perfil ativo definido.".into());
    }

    let api_key = std::env::var("BUNGIE_API_KEY")
        .map_err(|_| "API key não encontrada. Verifica a variável de ambiente.")?;

    let client = reqwest::Client::new();
    let now = Utc::now();
    let reset_at = get_daily_reset_utc(now);

    let profile_url = format!(
        "https://www.bungie.net/Platform/Destiny2/{}/Profile/{}/?components=200",
        membership_type, membership_id
    );

    let profile_text = client
        .get(profile_url)
        .header("X-API-Key", &api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let profile_json: Value = serde_json::from_str(&profile_text)
        .map_err(|e| format!("Erro a ler profile JSON: {}", e))?;

    let characters_data = profile_json["Response"]["characters"]["data"]
        .as_object()
        .ok_or("Não foi possível encontrar characters.data no profile.")?;

    let mut activities: Vec<Value> = Vec::new();

    for (character_id, _) in characters_data {
        let history_url = format!(
            "https://www.bungie.net/Platform/Destiny2/{}/Account/{}/Character/{}/Stats/Activities/?count=100&mode=4",
            membership_type, membership_id, character_id
        );

        let history_text = client
            .get(history_url)
            .header("X-API-Key", &api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let history_json: Value = serde_json::from_str(&history_text)
            .map_err(|e| format!("Erro a ler activity history JSON: {}", e))?;

        if let Some(items) = history_json["Response"]["activities"].as_array() {
            for item in items {
                let period_str = item["period"].as_str().unwrap_or("");
                if period_str.is_empty() {
                    continue;
                }

                let parsed_period = chrono::DateTime::parse_from_rfc3339(period_str)
                    .map(|dt| dt.with_timezone(&Utc));

                if let Ok(period) = parsed_period {
                    if period >= reset_at {
                        let activity_hash =
                            item["activityDetails"]["referenceId"].as_i64().unwrap_or(0);
                        let activity_name = if activity_hash > 0 {
                            fetch_activity_name(&client, &api_key, activity_hash).await
                        } else {
                            "Unknown Raid".to_string()
                        };

                        let duration_seconds = item["values"]["activityDurationSeconds"]["basic"]
                            ["value"]
                            .as_f64()
                            .unwrap_or(0.0) as i64;

                        let completed_value = item["values"]["completed"]["basic"]["value"]
                            .as_f64()
                            .unwrap_or(0.0);

                        activities.push(serde_json::json!({
                            "instanceId": item["activityDetails"]["instanceId"].as_str().unwrap_or("").to_string(),
                            "characterId": character_id,
                            "period": period_str,
                            "activityHash": activity_hash,
                            "activityName": activity_name,
                            "durationSeconds": duration_seconds,
                            "completed": completed_value > 0.0
                        }));
                    }
                }
            }
        }
    }

    activities.sort_by(|a, b| {
        let a_period = a["period"].as_str().unwrap_or("");
        let b_period = b["period"].as_str().unwrap_or("");
        b_period.cmp(a_period)
    });

    let result = serde_json::json!({
        "resetAt": reset_at.to_rfc3339(),
        "now": now.to_rfc3339(),
        "totalRaidsToday": activities.len(),
        "activities": activities
    });

    Ok(result.to_string())
}

fn emit_snapshot(app: &AppHandle, snapshot_json: &str) {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.emit("tracker_snapshot_update", snapshot_json.to_string());
    }

    if let Some(w) = app.get_webview_window("main") {
        let _ = w.emit("tracker_snapshot_update", snapshot_json.to_string());
    }
}

fn emit_raids(app: &AppHandle, raids_json: &str) {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.emit("tracker_raids_update", raids_json.to_string());
    }

    if let Some(w) = app.get_webview_window("main") {
        let _ = w.emit("tracker_raids_update", raids_json.to_string());
    }
}

async fn refresh_snapshot_once(app: &AppHandle, expected_profile_version: u64) {
    let state = app.state::<AppState>();

    let current_profile_version = *state.profile_version.lock().unwrap();
    if current_profile_version != expected_profile_version {
        return;
    }

    let membership_id = state.membership_id.lock().unwrap().clone();
    let membership_type = *state.membership_type.lock().unwrap();

    if membership_id.is_empty() {
        return;
    }

    if let Ok(snapshot_json) =
        build_tracker_snapshot_json(&state, &membership_id, membership_type).await
    {
        let current_profile_version = *state.profile_version.lock().unwrap();
        if current_profile_version != expected_profile_version {
            return;
        }

        let should_emit = {
            let mut lock = state.latest_snapshot_json.lock().unwrap();
            let changed = lock.as_ref() != Some(&snapshot_json);
            *lock = Some(snapshot_json.clone());
            changed
        };

        if should_emit {
            emit_snapshot(app, &snapshot_json);
        }
    }
}

async fn refresh_raids_once(app: &AppHandle, expected_profile_version: u64) {
    let state = app.state::<AppState>();

    let current_profile_version = *state.profile_version.lock().unwrap();
    if current_profile_version != expected_profile_version {
        return;
    }

    let membership_id = state.membership_id.lock().unwrap().clone();
    let membership_type = *state.membership_type.lock().unwrap();

    if membership_id.is_empty() {
        return;
    }

    if let Ok(raids_json) = build_todays_raid_history_json(&membership_id, membership_type).await {
        let current_profile_version = *state.profile_version.lock().unwrap();
        if current_profile_version != expected_profile_version {
            return;
        }

        let should_emit = {
            let mut lock = state.latest_raids_json.lock().unwrap();
            let changed = lock.as_ref() != Some(&raids_json);
            *lock = Some(raids_json.clone());
            changed
        };

        if should_emit {
            emit_raids(app, &raids_json);
        }
    }
}

pub fn start_tracker_poller(app: AppHandle) {
    let state = app.state::<AppState>();

    let mut handle_lock = state.poller_handle.lock().unwrap();
    if handle_lock.is_some() {
        return;
    }

    let app_handle = app.clone();

    let handle: tauri::async_runtime::JoinHandle<()> = tauri::async_runtime::spawn(async move {
        let mut tick: u64 = 0;

        loop {
            let profile_version = *app_handle
                .state::<AppState>()
                .profile_version
                .lock()
                .unwrap();

            refresh_snapshot_once(&app_handle, profile_version).await;

            if tick % 20 == 0 {
                refresh_raids_once(&app_handle, profile_version).await;
            }

            tick = tick.wrapping_add(1);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    *handle_lock = Some(handle);
}

#[tauri::command]
pub fn set_active_profile(
    app: AppHandle,
    state: tauri::State<AppState>,
    membership_id: String,
    membership_type: i32,
) {
    {
        let mut id = state.membership_id.lock().unwrap();
        let mut ty = state.membership_type.lock().unwrap();

        *id = membership_id;
        *ty = membership_type;
    }

    {
        let mut snapshot_lock = state.latest_snapshot_json.lock().unwrap();
        let mut raids_lock = state.latest_raids_json.lock().unwrap();

        *snapshot_lock = None;
        *raids_lock = None;
    }

    {
        let mut cached_hash = state.cached_activity_name_hash.lock().unwrap();
        let mut cached_name = state.cached_activity_name.lock().unwrap();

        *cached_hash = None;
        *cached_name = None;
    }

    let new_profile_version = {
        let mut v = state.profile_version.lock().unwrap();
        *v += 1;
        *v
    };

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        refresh_snapshot_once(&app_clone, new_profile_version).await;
        refresh_raids_once(&app_clone, new_profile_version).await;
    });
}

#[tauri::command]
pub async fn get_tracker_snapshot(state: tauri::State<'_, AppState>) -> Result<String, String> {
    if let Some(cached) = state.latest_snapshot_json.lock().unwrap().clone() {
        return Ok(cached);
    }

    let membership_id = state.membership_id.lock().unwrap().clone();
    let membership_type = *state.membership_type.lock().unwrap();

    let json = build_tracker_snapshot_json(&state, &membership_id, membership_type).await?;
    *state.latest_snapshot_json.lock().unwrap() = Some(json.clone());
    Ok(json)
}

#[tauri::command]
pub async fn get_todays_raid_history(state: tauri::State<'_, AppState>) -> Result<String, String> {
    if let Some(cached) = state.latest_raids_json.lock().unwrap().clone() {
        return Ok(cached);
    }

    let membership_id = state.membership_id.lock().unwrap().clone();
    let membership_type = *state.membership_type.lock().unwrap();

    let json = build_todays_raid_history_json(&membership_id, membership_type).await?;
    *state.latest_raids_json.lock().unwrap() = Some(json.clone());
    Ok(json)
}
