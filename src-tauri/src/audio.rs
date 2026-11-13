use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use hound::{WavSpec, WavWriter};
use crate::error::AppError;

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    stop_sender: Option<Sender<()>>,
    sample_rate: Arc<AtomicU32>,
    channels: Arc<AtomicU16>,
    start_time: Option<Instant>,
    last_voice_time: Arc<Mutex<Instant>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            stop_sender: None,
            sample_rate: Arc::new(AtomicU32::new(44100)),
            channels: Arc::new(AtomicU16::new(1)),
            start_time: None,
            last_voice_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn get_last_voice_time(&self) -> Instant {
        *self.last_voice_time.lock().unwrap()
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

        let sample_rate_val = config.sample_rate().0;
        let channels_val = config.channels();

        self.sample_rate.store(sample_rate_val, Ordering::SeqCst);
        self.channels.store(channels_val, Ordering::SeqCst);

        {
            let mut buf = self.audio_buffer.lock().unwrap();
            buf.clear();
        }

        let buffer_clone = Arc::clone(&self.audio_buffer);
        let is_recording_flag = Arc::clone(&self.is_recording);
        let last_voice_clone = Arc::clone(&self.last_voice_time);
        *self.last_voice_time.lock().unwrap() = Instant::now();
        is_recording_flag.store(true, Ordering::SeqCst);

        let (tx, rx) = channel::<()>();
        self.stop_sender = Some(tx);
        self.start_time = Some(Instant::now());

        // Spawn capture loop on a dedicated thread so Stream stays on that thread
        thread::spawn(move || {
            let err_fn = |err| {
                log::error!("Audio stream error: {}", err);
            };

            let stream_res = match config.sample_format() {
                SampleFormat::F32 => {
                    let buf = Arc::clone(&buffer_clone);
                    let flag = Arc::clone(&is_recording_flag);
                    let voice_time = Arc::clone(&last_voice_clone);
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &_| {
                            if flag.load(Ordering::SeqCst) {
                                let mut has_speech = false;
                                for &s in data {
                                    if s.abs() > 0.035 {
                                        has_speech = true;
                                        break;
                                    }
                                }
                                if has_speech {
                                    if let Ok(mut vt) = voice_time.lock() {
                                        *vt = Instant::now();
                                    }
                                }
                                if let Ok(mut b) = buf.lock() {
                                    b.extend_from_slice(data);
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                SampleFormat::I16 => {
                    let buf = Arc::clone(&buffer_clone);
                    let flag = Arc::clone(&is_recording_flag);
                    let voice_time = Arc::clone(&last_voice_clone);
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &_| {
                            if flag.load(Ordering::SeqCst) {
                                let mut has_speech = false;
                                for &s in data {
                                    let sample_f = s as f32 / i16::MAX as f32;
                                    if sample_f.abs() > 0.035 {
                                        has_speech = true;
                                        break;
                                    }
                                }
                                if has_speech {
                                    if let Ok(mut vt) = voice_time.lock() {
                                        *vt = Instant::now();
                                    }
                                }
                                if let Ok(mut b) = buf.lock() {
                                    for &s in data {
                                        b.push(s as f32 / i16::MAX as f32);
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                SampleFormat::U16 => {
                    let buf = Arc::clone(&buffer_clone);
                    let flag = Arc::clone(&is_recording_flag);
                    let voice_time = Arc::clone(&last_voice_clone);
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[u16], _: &_| {
                            if flag.load(Ordering::SeqCst) {
                                let mut has_speech = false;
                                for &s in data {
                                    let sample_f = (s as f32 - 32768.0) / 32768.0;
                                    if sample_f.abs() > 0.035 {
                                        has_speech = true;
                                        break;
                                    }
                                }
                                if has_speech {
                                    if let Ok(mut vt) = voice_time.lock() {
                                        *vt = Instant::now();
                                    }
                                }
                                if let Ok(mut b) = buf.lock() {
                                    for &s in data {
                                        b.push((s as f32 - 32768.0) / 32768.0);
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                _ => {
                    is_recording_flag.store(false, Ordering::SeqCst);
                    return;
                }
            };

            if let Ok(stream) = stream_res {
                if stream.play().is_ok() {
                    // Wait for stop signal
                    let _ = rx.recv();
                }
                // Dropping stream releases mic
            }
            is_recording_flag.store(false, Ordering::SeqCst);
        });

        Ok(())
    }

    pub fn extract_current_chunk(&mut self) -> Result<(Vec<u8>, f64), AppError> {
        let duration_secs = self
            .start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);

        let raw_samples = {
            let mut buf = self.audio_buffer.lock().unwrap();
            let samples = buf.clone();
            buf.clear();
            samples
        };

        if raw_samples.is_empty() || duration_secs < 0.4 {
            return Err(AppError::Audio(
                "Chunk is too short or empty".to_string(),
            ));
        }

        let wav_bytes = self.create_whisper_wav(&raw_samples)?;
        Ok((wav_bytes, duration_secs))
    }

    pub fn stop_recording(&mut self) -> Result<(Vec<u8>, f64), AppError> {
        self.is_recording.store(false, Ordering::SeqCst);

        if let Some(tx) = self.stop_sender.take() {
            let _ = tx.send(());
        }

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

        if raw_samples.is_empty() || duration_secs < 0.3 {
            return Err(AppError::Audio(
                "Recording was too short or empty".to_string(),
            ));
        }

        let wav_bytes = self.create_whisper_wav(&raw_samples)?;
        Ok((wav_bytes, duration_secs))
    }

    pub fn cancel_recording(&mut self) {
        self.is_recording.store(false, Ordering::SeqCst);
        if let Some(tx) = self.stop_sender.take() {
            let _ = tx.send(());
        }
        self.start_time = None;
        if let Ok(mut buf) = self.audio_buffer.lock() {
            buf.clear();
        }
    }

    fn create_whisper_wav(&self, samples: &[f32]) -> Result<Vec<u8>, AppError> {
        const TARGET_SAMPLE_RATE: u32 = 16000;
        let channels = self.channels.load(Ordering::SeqCst);
        let sample_rate = self.sample_rate.load(Ordering::SeqCst);

        let mono_samples: Vec<f32> = if channels > 1 {
            samples
                .chunks_exact(channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                .collect()
        } else {
            samples.to_vec()
        };

        let resampled: Vec<f32> = if sample_rate != TARGET_SAMPLE_RATE && sample_rate > 0 {
            let ratio = sample_rate as f64 / TARGET_SAMPLE_RATE as f64;
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
