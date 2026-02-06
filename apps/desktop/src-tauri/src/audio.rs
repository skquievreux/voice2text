#![allow(deprecated)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::io::{Cursor, Write};
use hound::{WavSpec, WavWriter};

#[allow(deprecated)]
pub struct SendStream(#[allow(dead_code)] pub cpal::Stream);
unsafe impl Send for SendStream {}

pub struct AudioRecorder {
    pub buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<SendStream>,
    spec: WavSpec,
}

impl AudioRecorder {
    pub fn list_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut device_names = Vec::new();
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    device_names.push(name);
                }
            }
        }
        device_names
    }

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

    pub fn start(&mut self, device_name: Option<String>) -> Result<(), String> {
        let host = cpal::default_host();
        
        let device = if let Some(target_name) = device_name {
            let mut found = None;
            if let Ok(devices) = host.input_devices() {
                for d in devices {
                    if let Ok(name) = d.name() {
                        if name == target_name {
                            found = Some(d);
                            break;
                        }
                    }
                }
            }
            found.ok_or(format!("Device '{}' not found", target_name))?
        } else {
             host.default_input_device()
                .ok_or("No input device found")?
        };
        
        let config = device.default_input_config().map_err(|e| e.to_string())?;
        let sample_format = config.sample_format();
        
        self.spec.sample_rate = config.sample_rate();
        self.spec.channels = config.channels();
        
        println!("INFO: Using Audio Device: {:?}", device.name().unwrap_or_default());
        println!("INFO: Hardware Sample Format: {:?}", sample_format);
        println!("INFO: Config: {}Hz, {} channels", self.spec.sample_rate, self.spec.channels);

        let buffer = self.buffer.clone();
        
        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    if let Ok(mut b) = buffer.lock() { b.extend_from_slice(data); }
                },
                |err| eprintln!("Audio error: {}", err),
                None
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    if let Ok(mut b) = buffer.lock() {
                        // Convert i16 to f32 [-1.0, 1.0]
                        b.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                    }
                },
                |err| eprintln!("Audio error: {}", err),
                None
            ),
            _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
        }.map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;
        self.stream = Some(SendStream(stream));
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(Vec<u8>, f32), String> {
        let _ = self.stream.take();
        let buffer_guard = self.buffer.lock().unwrap();
        
        // Check for signal
        let mut max_amp = 0.0f32;
        for &s in buffer_guard.iter() {
            if s.abs() > max_amp { max_amp = s.abs(); }
        }

        // Force Output Spec: 16kHz Mono 16-bit Int (Standard for ASR)
        let out_spec = WavSpec {
            channels: 1,
            sample_rate: self.spec.sample_rate, // Keep native rate to avoid aliasing issues for now
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut wav_data = Vec::new();
        let mut cursor = Cursor::new(&mut wav_data);
        let mut writer = WavWriter::new(&mut cursor, out_spec).map_err(|e| e.to_string())?;
        
        // Downmix to Mono & Convert to i16
        // If we have 2 channels, we average them. If 1, we just take it.
        let channels = self.spec.channels as usize;
        
        for chunk in buffer_guard.chunks(channels) {
            let mut sum = 0.0;
            for &sample in chunk {
                sum += sample;
            }
            let avg = sum / channels as f32;
            
            // Hard clip and scale to i16 range
            let sample_i16 = (avg.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(sample_i16).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
        
        if let Ok(appdata) = std::env::var("APPDATA") {
            let log_dir = std::path::Path::new(&appdata).join("Voice2Text").join("logs");
            let _ = std::fs::create_dir_all(&log_dir);
            let debug_file = log_dir.join("debug_recording.wav");
            if let Ok(mut file) = std::fs::File::create(&debug_file) {
                let _ = file.write_all(&wav_data);
            }
        }

        // Clear buffer
        drop(buffer_guard);
        self.buffer.lock().unwrap().clear();

        Ok((wav_data, max_amp))
    }
}
