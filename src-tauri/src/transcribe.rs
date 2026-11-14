use crate::settings::AppSettings;
use reqwest::multipart;
// use serde::Deserialize;
use std::path::Path;
#[cfg(not(target_os = "android"))]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub async fn transcribe(audio_path: &Path, settings: &AppSettings) -> Result<String, String> {
    match settings.transcribe_provider.as_str() {
        "openai" => transcribe_openai(audio_path, settings).await,
        _ => transcribe_local(audio_path, settings),
    }
}

pub async fn transcribe_openai(audio_path: &Path, settings: &AppSettings) -> Result<String, String> {
    if settings.openai_api_key.is_empty() {
        return Err("请先在设置中填写 OpenAI API Key".into());
    }

    let bytes = tokio::fs::read(audio_path)
        .await
        .map_err(|e| e.to_string())?;
    let filename = audio_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "audio.m4a".into());

    let part = multipart::Part::bytes(bytes)
        .file_name(filename)
        .mime_str("audio/mp4")
        .map_err(|e| e.to_string())?;

    let mut form = multipart::Form::new()
        .part("file", part)
        .text("model", settings.openai_transcribe_model.clone())
        .text("response_format", "text");

    if settings.whisper_language != "auto" {
        form = form.text("language", settings.whisper_language.clone());
    }

    let url = format!(
        "{}/audio/transcriptions",
        settings.openai_base_url.trim_end_matches('/')
    );

    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&settings.openai_api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("在线转录请求失败: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!("在线转录失败 HTTP {}: {}", status, body));
    }

    // response_format=text 时返回纯文本
    Ok(body.trim().to_string())
}

// 本地方案原名 transcribe_local，包装原 transcribe_audio
pub fn transcribe_local(audio_path: &Path, settings: &AppSettings) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        let _ = (audio_path, settings);
        return Err("Android 暂不支持本地 Whisper，请在设置中切换到在线转录".into());
    }
    #[cfg(not(target_os = "android"))]
    {
        if settings.whisper_model_path.is_empty() {
            return Err("请先在设置中指定 Whisper 模型路径".into());
        }
        transcribe_audio(
            audio_path,
            Path::new(&settings.whisper_model_path),
            &settings.whisper_language,
        )
    }
}

#[cfg(not(target_os = "android"))]
pub fn transcribe_audio(
    audio_path: &Path,
    model_path: &Path,
    language: &str,
) -> Result<String, String> {
    let samples = decode_to_16k_mono(audio_path)?;

    let ctx = WhisperContext::new_with_params(
        model_path.to_str().ok_or("模型路径无效")?,
        WhisperContextParameters::default(),
    )
    .map_err(|e| format!("加载 Whisper 模型失败: {e}"))?;

    let mut params = FullParams::new(SamplingStrategy::BeamSearch {
        beam_size: 5,
        patience: -1.0,
    });

    if language != "auto" {
        params.set_language(Some(language));
    }
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let mut state = ctx.create_state().map_err(|e| e.to_string())?;
    state
        .full(params, &samples)
        .map_err(|e| format!("转录失败: {e}"))?;

    let mut text = String::new();
    for segment in state.as_iter() {
        text.push_str(&segment.to_string());
        text.push('\n');
    }
    Ok(text)
}

/// 用 symphonia 解码任意音频，重采样到 16kHz mono f32
#[cfg(not(target_os = "android"))]
fn decode_to_16k_mono(path: &Path) -> Result<Vec<f32>, String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| e.to_string())?;
    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or_else(|| "没有默认音轨".to_string())?;
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;

    let mut all_samples: Vec<f32> = Vec::new();
    let mut sample_rate: u32 = 0;
    let mut channels: usize = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let spec = *decoded.spec();
        sample_rate = spec.rate;
        channels = spec.channels.count();

        let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        all_samples.extend_from_slice(sample_buf.samples());
    }

    if sample_rate == 0 || all_samples.is_empty() {
        return Err("音频解码结果为空".into());
    }

    // 多声道降为单声道
    let mono: Vec<f32> = if channels == 1 {
        all_samples
    } else {
        all_samples
            .chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    };

    // 线性插值重采样到 16000 Hz
    Ok(linear_resample(&mono, sample_rate, 16_000))
}

#[cfg(not(target_os = "android"))]
fn linear_resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to {
        return input.to_vec();
    }
    let ratio = to as f64 / from as f64;
    let out_len = (input.len() as f64 * ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos.floor() as usize;
        let frac = src_pos - idx as f64;
        let a = input.get(idx).copied().unwrap_or(0.0);
        let b = input.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac as f32);
    }
    out
}
