//! Native inference helper invoked by the Python topic pipeline.
//! No database, HTTP API, or article generation lives here.
use any_tts::models::qwen3_tts::Qwen3TtsModel;
use any_tts::traits::TtsModel;
use any_tts::{AudioSamples, DType, DeviceSelection, ModelType, SynthesisRequest, TtsConfig};
use std::{fs, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: football-qwen-tts MODEL_PATH REQUEST_JSON OUTPUT_WAV".into());
    }
    let request: serde_json::Value = serde_json::from_slice(&fs::read(&args[2])?)?;
    if request["voice"] != "eric" || request["language"] != "sichuan_dialect" {
        return Err("only Eric Sichuan narration is supported".into());
    }
    let parts = request["parts"].as_array().ok_or("parts must be an array")?;
    if parts.is_empty() || parts.len() > 60 {
        return Err("expected 1..60 narration segments".into());
    }
    let device = match request["device"].as_str().unwrap_or("cpu") {
        "metal" => DeviceSelection::Metal(0),
        "cuda" => DeviceSelection::Cuda(0),
        "cpu" => DeviceSelection::Cpu,
        _ => return Err("unsupported device".into()),
    };
    let dtype = match request["dtype"].as_str().unwrap_or("bf16") {
        "bf16" => DType::BF16,
        "f16" => DType::F16,
        "f32" => DType::F32,
        _ => return Err("unsupported dtype".into()),
    };
    let start = Instant::now();
    let model = Qwen3TtsModel::load(TtsConfig::new(ModelType::Qwen3Tts)
        .with_model_path(&args[1]).with_device(device).with_dtype(dtype))?;
    if !model.supported_voices().iter().any(|voice| voice == "eric") {
        return Err("local model does not support Eric".into());
    }
    let mut samples = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let text = part.as_str().ok_or("narration segment must be text")?;
        if text.trim().is_empty() || text.chars().count() > 150 {
            return Err("narration segment must contain 1..150 characters".into());
        }
        let audio = model.synthesize(&SynthesisRequest::new(text)
            .with_language("sichuan_dialect").with_voice("eric").with_max_tokens(2048))?;
        if audio.duration_secs() >= 2047.0 / 12.0 {
            return Err("narration reached token limit; refusing truncated audio".into());
        }
        if audio.sample_rate != 24000 {
            return Err("unexpected Qwen sample rate".into());
        }
        samples.extend(audio.samples);
        if index + 1 < parts.len() {
            samples.extend(std::iter::repeat_n(0.0, 6000));
        }
    }
    let audio = AudioSamples::new(samples, 24000);
    audio.save_wav(&args[3])?;
    println!("{}", serde_json::json!({"duration_seconds": audio.duration_secs(),
        "elapsed_seconds": start.elapsed().as_secs_f64(), "voice": "eric", "dialect": "sichuan"}));
    Ok(())
}
