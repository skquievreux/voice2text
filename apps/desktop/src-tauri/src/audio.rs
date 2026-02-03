use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::io::Cursor;
use hound::{WavSpec, WavWriter};

pub struct SendStream(pub cpal::Stream);
unsafe impl Send for SendStream {}

pub struct AudioRecorder {
    pub buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<SendStream>,
    spec: WavSpec,
}

impl AudioRecorder {
    pub fn new() -> Self {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            spec,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device found")?;
        
        let config = device.default_input_config().map_err(|e| e.to_string())?;
        let buffer = self.buffer.clone();

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = buffer.lock().unwrap();
                buffer.extend_from_slice(data);
            },
            |err| eprintln!("Audio input stream error: {}", err),
            None
        ).map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;
        self.stream = Some(SendStream(stream));
        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<u8>, String> {
        let _ = self.stream.take(); // Drops the SendStream(cpal::Stream)
        let buffer = self.buffer.lock().unwrap();
        
        let mut wav_data = Vec::new();
        let mut cursor = Cursor::new(&mut wav_data);
        let mut writer = WavWriter::new(&mut cursor, self.spec).map_err(|e| e.to_string())?;
        
        for &sample in buffer.iter() {
            writer.write_sample(sample).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
        
        // Debug: Save file to disk
        use std::io::Write;
        if let Ok(mut file) = std::fs::File::create("debug_recording.wav") {
            let _ = file.write_all(&wav_data);
        }

        // Clear buffer for next time
        drop(buffer);
        self.buffer.lock().unwrap().clear();

        Ok(wav_data)
    }
}
