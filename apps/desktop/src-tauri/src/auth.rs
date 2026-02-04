use machine_uid;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

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
    let client = reqwest::Client::new();
    
    let res = client.post("https://voice2text-web.vercel.app/api/client/status")
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
