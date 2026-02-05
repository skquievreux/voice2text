use machine_uid;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientStatus {
    pub status: String, // trial, active, banned
    pub valid_until: String,
    pub signature: String,
    pub token: String,
    pub now: String,
    pub messages: Vec<ServerMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMessage {
    pub id: String,
    pub type_str: Option<String>,
    pub title: String,
    pub body: String,
    pub action_url: Option<String>,
}

pub fn get_hw_id() -> String {
    let uid = machine_uid::get().unwrap_or_else(|_| "unknown_device".to_string());
    let mut hasher = Sha256::new();
    hasher.update(uid.as_bytes());
    hasher.update(b"v2t-salt-2026"); // Production salt
    format!("{:x}", hasher.finalize())
}

pub async fn check_status() -> Result<ClientStatus, String> {
    let hw_id = get_hw_id();
    use std::time::Duration;
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default();
    
    let api_base = if cfg!(debug_assertions) {
        "http://127.0.0.1:3000"
    } else {
        "https://voice2text.runitfast.xyz"
    };
    
    let api_url = format!("{}/api/client/status", api_base);
    println!("INFO: Connecting to Auth API: {}", api_url);

    let res = client.post(&api_url)
        .json(&serde_json::json!({ "hwId": hw_id }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status_code = res.status();
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Server error ({}): {}", status_code, err_text));
    }

    let status: ClientStatus = res.json().await.map_err(|e| e.to_string())?;
    Ok(status)

}

#[tauri::command]
pub async fn fetch_campaigns(_app: AppHandle, state: State<'_, crate::AppState>) -> Result<serde_json::Value, String> {
    let token = {
        let status_guard = state.client_status.lock().unwrap();
        if let Some(ref s) = *status_guard {
            s.token.clone()
        } else {
            return Err("Client not registered".to_string());
        }
    };

    let client = reqwest::Client::new();
    let api_base = if cfg!(debug_assertions) {
        "http://localhost:3000"
    } else {
        "https://voice2text.runitfast.xyz"
    };
    
    let api_url = format!("{}/api/campaigns/fetch", api_base);
    println!("INFO: Fetching Campaigns from: {}", api_url);

    let res = client.post(&api_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("API Error: {}", res.status()));
    }

    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(data)
}
