use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use hound::{WavSpec, WavWriter};
use crate::error::AppError;

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    active_stream: Option<Stream>,
    sample_rate: u32,
    channels: u16,
    start_time: Option<Instant>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            active_stream: None,
            sample_rate: 44100,
            channels: 1,
            start_time: None,
        }
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn check_device_ready() -> Result<bool, AppError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("No audio input device found".to_string()))?;

        let _ = device
            .default_input_config()
            .map_err(|e| AppError::Audio(format!("Failed to get input device config: {}", e)))?;

        Ok(true)
    }

    pub fn start_recording(&mut self) -> Result<(), AppError> {
        if self.is_recording() {
            return Ok(());
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("No audio input device found".to_string()))?;

        let config = device
            .default_input_config()
            .map_err(|e| AppError::Audio(format!("Failed to get input config: {}", e)))?;

        self.sample_rate = config.sample_rate().0;
        self.channels = config.channels();

        {
            let mut buf = self.audio_buffer.lock().unwrap();
            buf.clear();
        }

        let buffer_clone = Arc::clone(&self.audio_buffer);
        let is_recording_flag = Arc::clone(&self.is_recording);
        is_recording_flag.store(true, Ordering::SeqCst);

        let err_fn = |err| {
            log::error!("Audio recording stream error: {}", err);
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    if is_recording_flag.load(Ordering::SeqCst) {
                        let mut buf = buffer_clone.lock().unwrap();
                        buf.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    if is_recording_flag.load(Ordering::SeqCst) {
                        let mut buf = buffer_clone.lock().unwrap();
                        for &sample in data {
                            buf.push(sample as f32 / i16::MAX as f32);
                        }
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _: &_| {
                    if is_recording_flag.load(Ordering::SeqCst) {
                        let mut buf = buffer_clone.lock().unwrap();
                        for &sample in data {
                            buf.push((sample as f32 - 32768.0) / 32768.0);
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => {
                self.is_recording.store(false, Ordering::SeqCst);
                return Err(AppError::Audio("Unsupported sample format".to_string()));
            }
        }
        .map_err(|e| AppError::Audio(format!("Failed to build audio stream: {}", e)))?;

        stream
            .play()
            .map_err(|e| AppError::Audio(format!("Failed to start audio playback/capture: {}", e)))?;

        self.active_stream = Some(stream);
        self.start_time = Some(Instant::now());

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<(Vec<u8>, f64), AppError> {
        self.is_recording.store(false, Ordering::SeqCst);

        // Drop the stream to release microphone device immediately
        self.active_stream.take();

        let duration_secs = self
            .start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        self.start_time = None;

        let raw_samples = {
            let mut buf = self.audio_buffer.lock().unwrap();
            let samples = buf.clone();
            buf.clear();
            samples
        };

        // Minimum recording threshold check (300ms)
        if raw_samples.is_empty() || duration_secs < 0.3 {
            return Err(AppError::Audio(
                "Recording was too short or empty".to_string(),
            ));
        }

        // Convert audio to 16kHz mono WAV bytes (Whisper standard)
        let wav_bytes = self.create_whisper_wav(&raw_samples)?;

        Ok((wav_bytes, duration_secs))
    }

    pub fn cancel_recording(&mut self) {
        self.is_recording.store(false, Ordering::SeqCst);
        self.active_stream.take();
        self.start_time = None;
        if let Ok(mut buf) = self.audio_buffer.lock() {
            buf.clear();
        }
    }

    /// Resamples and encodes to 16kHz 16-bit mono WAV in memory
    fn create_whisper_wav(&self, samples: &[f32]) -> Result<Vec<u8>, AppError> {
        const TARGET_SAMPLE_RATE: u32 = 16000;

        // Downmix multi-channel to mono if needed
        let mono_samples: Vec<f32> = if self.channels > 1 {
            samples
                .chunks_exact(self.channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / self.channels as f32)
                .collect()
        } else {
            samples.to_vec()
        };

        // Resample to 16000 Hz using linear interpolation
        let resampled: Vec<f32> = if self.sample_rate != TARGET_SAMPLE_RATE && self.sample_rate > 0 {
            let ratio = self.sample_rate as f64 / TARGET_SAMPLE_RATE as f64;
            let target_len = (mono_samples.len() as f64 / ratio) as usize;
            let mut output = Vec::with_capacity(target_len);

            for i in 0..target_len {
                let src_idx = i as f64 * ratio;
                let idx_floor = src_idx.floor() as usize;
                let idx_ceil = (idx_floor + 1).min(mono_samples.len() - 1);
                let frac = (src_idx - idx_floor as f64) as f32;

                if idx_floor < mono_samples.len() {
                    let val = mono_samples[idx_floor] * (1.0 - frac) + mono_samples[idx_ceil] * frac;
                    output.push(val);
                }
            }
            output
        } else {
            mono_samples
        };

        // Encode to WAV using hound
        let spec = WavSpec {
            channels: 1,
            sample_rate: TARGET_SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut cursor, spec)
                .map_err(|e| AppError::Audio(format!("Failed to create WAV writer: {}", e)))?;

            for &s in &resampled {
                // Clamp sample to [-1.0, 1.0] and scale to i16
                let clamped = s.clamp(-1.0, 1.0);
                let sample_i16 = (clamped * 32767.0) as i16;
                writer
                    .write_sample(sample_i16)
                    .map_err(|e| AppError::Audio(format!("WAV write sample error: {}", e)))?;
            }

            writer
                .finalize()
                .map_err(|e| AppError::Audio(format!("Failed to finalize WAV: {}", e)))?;
        }

        Ok(cursor.into_inner())
    }
}
