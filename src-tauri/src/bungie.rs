// src-tauri/src/bungie.rs

use serde_json::Value;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Olá {}, o tracker está vivo!", name)
}

#[tauri::command]
pub async fn bungie_status() -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.bungie.net/Platform/Settings/")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = response.text().await.map_err(|e| e.to_string())?;
    Ok(text)
}

#[tauri::command]
pub async fn search_player(bungie_name: String) -> Result<String, String> {
    let api_key = std::env::var("BUNGIE_API_KEY")
        .map_err(|_| "API key não encontrada. Verifica a variável de ambiente.")?;

    let parts: Vec<&str> = bungie_name.split('#').collect();

    if parts.len() != 2 {
        return Err("Formato inválido. Usa Nome#1234".into());
    }

    let name = parts[0];
    let code = parts[1];

    let client = reqwest::Client::new();

    let res = client
        .post("https://www.bungie.net/Platform/Destiny2/SearchDestinyPlayerByBungieName/3/")
        .header("X-API-Key", api_key)
        .json(&serde_json::json!({
            "displayName": name,
            "displayNameCode": code
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = res.text().await.map_err(|e| e.to_string())?;
    Ok(text)
}

#[tauri::command]
pub async fn get_profile(membership_id: String, membership_type: i32) -> Result<String, String> {
    let api_key = std::env::var("BUNGIE_API_KEY")
        .map_err(|_| "API key não encontrada. Verifica a variável de ambiente.")?;

    let url = format!(
        "https://www.bungie.net/Platform/Destiny2/{}/Profile/{}/?components=100,200,202,204",
        membership_type, membership_id
    );

    let client = reqwest::Client::new();

    let res = client
        .get(url)
        .header("X-API-Key", api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = res.text().await.map_err(|e| e.to_string())?;
    Ok(text)
}

#[tauri::command]
pub async fn get_current_activity(
    membership_id: String,
    membership_type: i32,
) -> Result<String, String> {
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

    let profile_json: Value =
        serde_json::from_str(&profile_text).map_err(|e| format!("Erro a ler profile JSON: {}", e))?;

    let characters_data = profile_json["Response"]["characters"]["data"]
        .as_object()
        .ok_or("Não foi possível encontrar characters.data no profile.")?;

    let character_activities_data = profile_json["Response"]["characterActivities"]["data"]
        .as_object()
        .ok_or("Não foi possível encontrar characterActivities.data no profile.")?;

    let profile_data = &profile_json["Response"]["profile"]["data"];
    let date_last_played = profile_data["dateLastPlayed"].as_str().unwrap_or("");

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

    for (character_id, activity_data) in character_activities_data {
        let activity_hash = activity_data["currentActivityHash"].as_i64();

        if activity_hash.unwrap_or(0) > 0 {
            selected_character_id = character_id.to_string();
            selected_character_last_played = characters_data
                .get(character_id)
                .and_then(|c| c["dateLastPlayed"].as_str())
                .unwrap_or("")
                .to_string();

            current_activity_hash = activity_hash;
            current_activity_mode_hash = activity_data["currentActivityModeHash"].as_i64();
            current_activity_mode_type = activity_data["currentActivityModeType"].as_i64();
            break;
        }
    }

    if current_activity_hash.is_none() {
        if let Some(activity_data) = character_activities_data.get(&latest_character_id) {
            current_activity_hash = activity_data["currentActivityHash"].as_i64();
            current_activity_mode_hash = activity_data["currentActivityModeHash"].as_i64();
            current_activity_mode_type = activity_data["currentActivityModeType"].as_i64();
        }
    }

    let result = serde_json::json!({
        "characterId": selected_character_id,
        "profileLastPlayed": date_last_played,
        "characterLastPlayed": selected_character_last_played,
        "currentActivityHash": current_activity_hash,
        "currentActivityModeHash": current_activity_mode_hash,
        "currentActivityModeType": current_activity_mode_type
    });

    Ok(result.to_string())
}