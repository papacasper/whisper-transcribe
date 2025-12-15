#![windows_subsystem = "windows"]

use anyhow::{Context, Result, bail};
use eframe::egui;
use futures_util::StreamExt;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// Available Whisper models
const WHISPER_MODELS: &[(&str, &str)] = &[
    ("tiny", "ggml-tiny.bin"),
    ("base", "ggml-base.bin"),
    ("small", "ggml-small.bin"),
    ("medium", "ggml-medium.bin"),
    ("large", "ggml-large-v3-turbo.bin"),
];

fn get_model_url(filename: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        filename
    )
}

// Supported audio formats
const AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3", "flac", "ogg", "m4a", "aac", "wma", "opus", "webm"];

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Whisper Transcribe",
        options,
        Box::new(|_cc| Ok(Box::new(WhisperApp::default()))),
    )
}

#[derive(PartialEq)]
enum TranscribeStatus {
    Idle,
    Loading,
    Transcribing,
    Done,
    Error(String),
}

#[derive(PartialEq, Clone)]
enum DownloadStatus {
    Idle,
    Downloading { downloaded: u64, total: u64 },
    Done,
    Error(String),
}

enum DownloadMessage {
    Progress { downloaded: u64, total: u64 },
    Done(PathBuf),
    Error(String),
}

/// Check if CUDA is available by attempting to initialize whisper with GPU
fn check_cuda_available() -> bool {
    // We can't easily check without a model, so we'll detect during first transcription
    // For now, check if CUDA environment is set up
    std::env::var("CUDA_PATH").is_ok()
}

struct WhisperApp {
    model_path: Option<PathBuf>,
    audio_path: Option<PathBuf>,
    transcription: String,
    status: TranscribeStatus,
    receiver: Option<Receiver<TranscribeMessage>>,
    using_gpu: Option<bool>,
    // Download state
    selected_model_idx: usize,
    download_status: DownloadStatus,
    download_receiver: Option<Receiver<DownloadMessage>>,
    cuda_available: bool,
}

enum TranscribeMessage {
    Status(String),
    GpuStatus(bool),
    Done(String),
    Error(String),
}

impl Default for WhisperApp {
    fn default() -> Self {
        Self {
            model_path: None,
            audio_path: None,
            transcription: String::new(),
            status: TranscribeStatus::Idle,
            receiver: None,
            using_gpu: None,
            selected_model_idx: 0,
            download_status: DownloadStatus::Idle,
            download_receiver: None,
            cuda_available: check_cuda_available(),
        }
    }
}

impl WhisperApp {
    fn start_transcription(&mut self) {
        let model_path = self.model_path.clone().unwrap();
        let audio_path = self.audio_path.clone().unwrap();

        let (tx, rx) = channel();
        self.receiver = Some(rx);
        self.status = TranscribeStatus::Loading;
        self.transcription.clear();

        thread::spawn(move || {
            run_transcription(model_path, audio_path, tx);
        });
    }

    fn check_messages(&mut self) {
        let mut should_clear_receiver = false;

        if let Some(ref receiver) = self.receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    TranscribeMessage::Status(s) => {
                        if s.contains("Transcribing") {
                            self.status = TranscribeStatus::Transcribing;
                        }
                    }
                    TranscribeMessage::GpuStatus(gpu) => {
                        self.using_gpu = Some(gpu);
                    }
                    TranscribeMessage::Done(text) => {
                        self.transcription = text;
                        self.status = TranscribeStatus::Done;
                        should_clear_receiver = true;
                    }
                    TranscribeMessage::Error(e) => {
                        self.status = TranscribeStatus::Error(e);
                        should_clear_receiver = true;
                    }
                }
            }
        }

        if should_clear_receiver {
            self.receiver = None;
        }
    }

    fn start_download(&mut self) {
        let (_, filename) = WHISPER_MODELS[self.selected_model_idx];
        let url = get_model_url(filename);
        let dest_path = PathBuf::from("models").join(filename);

        let (tx, rx) = channel();
        self.download_receiver = Some(rx);
        self.download_status = DownloadStatus::Downloading { downloaded: 0, total: 0 };

        thread::spawn(move || {
            run_download(url, dest_path, tx);
        });
    }

    fn check_download_messages(&mut self) {
        let mut should_clear_receiver = false;
        let mut completed_path: Option<PathBuf> = None;

        if let Some(ref receiver) = self.download_receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    DownloadMessage::Progress { downloaded, total } => {
                        self.download_status = DownloadStatus::Downloading { downloaded, total };
                    }
                    DownloadMessage::Done(path) => {
                        self.download_status = DownloadStatus::Done;
                        completed_path = Some(path);
                        should_clear_receiver = true;
                    }
                    DownloadMessage::Error(e) => {
                        self.download_status = DownloadStatus::Error(e);
                        should_clear_receiver = true;
                    }
                }
            }
        }

        if should_clear_receiver {
            self.download_receiver = None;
        }

        // Auto-select downloaded model
        if let Some(path) = completed_path {
            self.model_path = Some(path);
        }
    }

    fn copy_to_clipboard(&self) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(&self.transcription);
        }
    }

    fn save_to_file(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .set_file_name("transcription.txt")
            .save_file()
        {
            let _ = std::fs::write(path, &self.transcription);
        }
    }
}

impl eframe::App for WhisperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_messages();
        self.check_download_messages();

        // Handle dropped files
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "bin" {
                        self.model_path = Some(path.clone());
                    } else if AUDIO_EXTENSIONS.contains(&ext_lower.as_str()) {
                        self.audio_path = Some(path.clone());
                    }
                }
            }
        });

        // Request repaint while processing
        if self.receiver.is_some() || self.download_receiver.is_some() {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Whisper Transcribe");
            ui.add_space(10.0);

            // Model download section
            ui.horizontal(|ui| {
                ui.label("Download Model:");
                let selected_name = WHISPER_MODELS[self.selected_model_idx].0;
                egui::ComboBox::from_id_salt("model_select")
                    .selected_text(selected_name)
                    .show_ui(ui, |ui| {
                        for (idx, (name, _)) in WHISPER_MODELS.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_model_idx, idx, *name);
                        }
                    });

                let is_downloading = matches!(self.download_status, DownloadStatus::Downloading { .. });
                if ui.add_enabled(!is_downloading, egui::Button::new("Download")).clicked() {
                    self.start_download();
                }
            });

            // Download progress bar
            match &self.download_status {
                DownloadStatus::Downloading { downloaded, total } => {
                    let progress = if *total > 0 {
                        *downloaded as f32 / *total as f32
                    } else {
                        0.0
                    };
                    let downloaded_mb = *downloaded as f64 / 1_000_000.0;
                    let total_mb = *total as f64 / 1_000_000.0;
                    ui.add(egui::ProgressBar::new(progress).text(format!(
                        "{:.1} MB / {:.1} MB ({:.0}%)",
                        downloaded_mb,
                        total_mb,
                        progress * 100.0
                    )));
                }
                DownloadStatus::Done => {
                    ui.colored_label(egui::Color32::from_rgb(76, 175, 80), "Download complete!");
                }
                DownloadStatus::Error(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Download error: {}", e));
                }
                DownloadStatus::Idle => {}
            }

            ui.add_space(5.0);

            // Model selection (browse or use downloaded)
            ui.horizontal(|ui| {
                ui.label("Model:");
                if let Some(ref path) = self.model_path {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                } else {
                    ui.label("(none)");
                }
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Whisper Model", &["bin"])
                        .pick_file()
                    {
                        self.model_path = Some(path);
                    }
                }
            });

            ui.add_space(5.0);

            // Audio selection
            ui.horizontal(|ui| {
                ui.label("Audio:");
                if let Some(ref path) = self.audio_path {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                } else {
                    ui.label("(none)");
                }
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Audio Files", AUDIO_EXTENSIONS)
                        .pick_file()
                    {
                        self.audio_path = Some(path);
                    }
                }
            });

            ui.add_space(10.0);

            // Drag & drop hint with supported formats
            ui.label("Drag & drop audio files (MP3, WAV, FLAC, OGG, M4A, AAC, WMA, Opus)");

            ui.add_space(10.0);

            // Transcribe button
            let can_transcribe = self.model_path.is_some()
                && self.audio_path.is_some()
                && self.status != TranscribeStatus::Loading
                && self.status != TranscribeStatus::Transcribing;

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_transcribe, egui::Button::new("Transcribe"))
                    .clicked()
                {
                    self.start_transcription();
                }

                // Status indicator
                match &self.status {
                    TranscribeStatus::Idle => {}
                    TranscribeStatus::Loading => {
                        ui.spinner();
                        ui.label("Loading...");
                    }
                    TranscribeStatus::Transcribing => {
                        ui.spinner();
                        ui.label("Transcribing...");
                    }
                    TranscribeStatus::Done => {
                        ui.label("Done!");
                    }
                    TranscribeStatus::Error(e) => {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
                    }
                }
            });

            ui.add_space(10.0);

            // Output area
            ui.separator();
            ui.label("Transcription:");

            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.transcription.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(10),
                    );
                });

            ui.add_space(10.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.transcription.is_empty(), egui::Button::new("Copy"))
                    .clicked()
                {
                    self.copy_to_clipboard();
                }
                if ui
                    .add_enabled(!self.transcription.is_empty(), egui::Button::new("Save..."))
                    .clicked()
                {
                    self.save_to_file();
                }
                if ui.button("Clear").clicked() {
                    self.transcription.clear();
                    self.status = TranscribeStatus::Idle;
                }
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Show CUDA availability
                if self.cuda_available {
                    ui.colored_label(
                        egui::Color32::from_rgb(76, 175, 80),
                        "CUDA Available",
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 152, 0),
                        "CUDA Not Found (CPU mode)",
                    );
                }

                ui.separator();

                // Show runtime status
                let (icon, text, color) = match self.using_gpu {
                    Some(true) => ("⚡", "Running on GPU", egui::Color32::from_rgb(76, 175, 80)),
                    Some(false) => ("💻", "Running on CPU", egui::Color32::from_rgb(255, 152, 0)),
                    None => ("○", "Ready", egui::Color32::GRAY),
                };
                ui.colored_label(color, format!("{} {}", icon, text));
            });
        });
    }
}

fn run_transcription(model_path: PathBuf, audio_path: PathBuf, tx: Sender<TranscribeMessage>) {
    let result = (|| -> Result<String> {
        tx.send(TranscribeMessage::Status("Loading model...".to_string()))
            .ok();

        // Try GPU first, fallback to CPU if it fails
        let (ctx, using_gpu) = {
            let mut ctx_params = WhisperContextParameters::default();
            ctx_params.use_gpu(true);

            match WhisperContext::new_with_params(
                model_path.to_str().context("Invalid model path")?,
                ctx_params,
            ) {
                Ok(c) => (c, true),
                Err(_) => {
                    // Fallback to CPU
                    let ctx_params = WhisperContextParameters::default();
                    let c = WhisperContext::new_with_params(
                        model_path.to_str().context("Invalid model path")?,
                        ctx_params,
                    )
                    .context("Failed to load Whisper model")?;
                    (c, false)
                }
            }
        };

        tx.send(TranscribeMessage::GpuStatus(using_gpu)).ok();

        tx.send(TranscribeMessage::Status("Loading audio...".to_string()))
            .ok();

        let audio_data = load_audio_to_mono_16khz(&audio_path)?;

        tx.send(TranscribeMessage::Status("Transcribing...".to_string()))
            .ok();

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let mut state = ctx.create_state().context("Failed to create state")?;
        state
            .full(params, &audio_data)
            .context("Failed to transcribe audio")?;

        let num_segments = state.full_n_segments();
        let mut result = String::new();

        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(text) = segment.to_str_lossy() {
                    result.push_str(&text);
                }
            }
        }

        Ok(result.trim().to_string())
    })();

    match result {
        Ok(text) => {
            tx.send(TranscribeMessage::Done(text)).ok();
        }
        Err(e) => {
            tx.send(TranscribeMessage::Error(e.to_string())).ok();
        }
    }
}

fn run_download(url: String, dest_path: PathBuf, tx: Sender<DownloadMessage>) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = download_model(&url, &dest_path, &tx).await;
        if let Err(e) = result {
            tx.send(DownloadMessage::Error(e.to_string())).ok();
        }
    });
}

async fn download_model(
    url: &str,
    dest_path: &PathBuf,
    tx: &Sender<DownloadMessage>,
) -> Result<()> {
    // Ensure models directory exists
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create models directory")?;
    }

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to start download")?;

    if !response.status().is_success() {
        bail!("Download failed: HTTP {}", response.status());
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = File::create(dest_path).context("Failed to create model file")?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error downloading chunk")?;
        file.write_all(&chunk).context("Failed to write to file")?;
        downloaded += chunk.len() as u64;
        tx.send(DownloadMessage::Progress {
            downloaded,
            total: total_size,
        })
        .ok();
    }

    // Get absolute path for the model
    let abs_path = dest_path
        .canonicalize()
        .unwrap_or_else(|_| dest_path.clone());
    tx.send(DownloadMessage::Done(abs_path)).ok();
    Ok(())
}

/// Load any supported audio file and convert to mono 16kHz f32 samples
fn load_audio_to_mono_16khz(path: &PathBuf) -> Result<Vec<f32>> {
    let file = File::open(path).context("Failed to open audio file")?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Create a hint based on file extension
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    // Probe the media source
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .context("Unsupported audio format")?;

    let mut format = probed.format;

    // Find the first audio track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .context("No audio track found")?;

    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .context("Unknown sample rate")?;
    let channels = track
        .codec_params
        .channels
        .context("Unknown channel count")?
        .count();

    // Create decoder
    let decoder_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .context("Failed to create decoder")?;

    // Decode all packets
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => bail!("Error reading packet: {}", e),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => bail!("Decode error: {}", e),
        };

        let spec = *decoded.spec();
        let duration = decoded.capacity() as usize;

        let mut sample_buf = SampleBuffer::<f32>::new(duration as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);

        all_samples.extend_from_slice(sample_buf.samples());
    }

    if all_samples.is_empty() {
        bail!("No audio samples decoded");
    }

    // Convert to mono if stereo/multi-channel
    let mono_samples = if channels > 1 {
        all_samples
            .chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        all_samples
    };

    // Resample to 16kHz if needed
    let target_rate = 16000;
    if sample_rate == target_rate {
        Ok(mono_samples)
    } else {
        resample_audio(&mono_samples, sample_rate, target_rate)
    }
}

/// High-quality resampling using rubato
fn resample_audio(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>> {
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = SincFixedIn::<f32>::new(
        to_rate as f64 / from_rate as f64,
        2.0,
        params,
        samples.len(),
        1,
    )
    .context("Failed to create resampler")?;

    let waves_in = vec![samples.to_vec()];
    let waves_out = resampler
        .process(&waves_in, None)
        .context("Failed to resample audio")?;

    Ok(waves_out.into_iter().next().unwrap_or_default())
}
