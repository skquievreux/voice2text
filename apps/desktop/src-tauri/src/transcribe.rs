use reqwest::multipart;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TranscribeResponse {
    pub text: String,
}

pub async fn send_to_api(wav_data: Vec<u8>) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let part = multipart::Part::bytes(wav_data)
        .file_name("audio.wav")
        .mime_str("audio/wav").unwrap();

    let form = multipart::Form::new()
        .part("audio", part);

    let response = client.post("https://voice2-text-web.vercel.app/api/transcribe")
        .header("Authorization", "Bearer eyJhbGciOiJIUzI1NiJ9.eyJ0aWVyIjoicHJvIiwic3ViIjoiZGV2LXVzZXItaWQiLCJpYXQiOjE3NzAxMzM3OTcsImV4cCI6MTgwMTY2OTc5N30.MPRagqw7MoWAEmQ58yldqT6WxjYsAJw0ZnBb979vx_A")
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let res: TranscribeResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(res.text)
}
