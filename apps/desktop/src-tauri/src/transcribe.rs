use tauri::AppHandle;
use reqwest::multipart;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TranscribeResponse {
    pub text: String,
}

pub async fn send_to_api(app: &AppHandle, wav_data: Vec<u8>, jwt_token: &str) -> Result<String, String> {
    const MAX_CHUNK_SIZE: usize = 3 * 1024 * 1024; // 3 MB safety limit
    
    if wav_data.len() <= MAX_CHUNK_SIZE {
        return send_chunk(app, wav_data, 0, 1, jwt_token).await;
    }

    crate::write_to_log(app, &format!("API: Splitting large file ({} bytes)", wav_data.len()));
    
    let mut reader = hound::WavReader::new(std::io::Cursor::new(wav_data)).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    
    let bytes_per_sample = (spec.bits_per_sample / 8) as usize;
    let max_samples = MAX_CHUNK_SIZE / bytes_per_sample;
    
    let chunks: Vec<&[i16]> = samples.chunks(max_samples).collect();
    let total_chunks = chunks.len();
    
    let mut full_transcript = String::new();

    for (i, chunk) in chunks.iter().enumerate() {
        let mut chunk_wav = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut chunk_wav);
        let mut writer = hound::WavWriter::new(&mut cursor, spec).map_err(|e| e.to_string())?;
        
        for &s in *chunk {
            writer.write_sample(s).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
        
        let text = send_chunk(app, chunk_wav, i + 1, total_chunks, jwt_token).await?;
        if !full_transcript.is_empty() {
            full_transcript.push(' ');
        }
        full_transcript.push_str(&text);
    }

    Ok(full_transcript)
}

async fn send_chunk(app: &AppHandle, wav_data: Vec<u8>, chunk_idx: usize, total: usize, jwt_token: &str) -> Result<String, String> {
    crate::write_to_log(app, &format!("API: Sending chunk {}/{} ({} bytes)", chunk_idx, total, wav_data.len()));
    
    let client = reqwest::Client::new();
    let part = multipart::Part::bytes(wav_data)
        .file_name("audio.wav")
        .mime_str("audio/wav").unwrap();

    let form = multipart::Form::new().part("audio", part);

    let response = client.post("https://voice2text.runitfast.xyz/api/transcribe")
        .header("Authorization", format!("Bearer {}", jwt_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let err_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error ({}): {}", response.status(), err_body));
    }

    let res: TranscribeResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(res.text)
}
