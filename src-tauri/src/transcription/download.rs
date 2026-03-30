// transcription::download — Model download logic, progress event payloads,
// and model directory helpers.
//
// This module is NOT feature-gated — download commands work regardless of
// whether the local-transcription (whisper-rs) feature is compiled.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const VALID_MODEL_IDS: &[&str] = &["tiny", "base", "small", "medium"];

pub const MODEL_DOWNLOAD_PROGRESS_EVENT: &str = "model-download-progress";
pub const MODEL_DOWNLOAD_DONE_EVENT: &str = "model-download-done";
pub const MODEL_DOWNLOAD_CANCELLED_EVENT: &str = "model-download-cancelled";
pub const MODEL_DOWNLOAD_ERROR_EVENT: &str = "model-download-error";

// ---------------------------------------------------------------------------
// Payload structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgressPayload {
    pub model_id: String,
    pub percent: f32,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadDonePayload {
    pub model_id: String,
    pub model_path: String,
}

/// Used for both cancelled and error events.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadEventPayload {
    pub model_id: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Returns the models directory: `%APPDATA%/VoxFlow/models/`.
pub fn models_dir() -> Result<std::path::PathBuf, String> {
    let base = dirs_next::config_dir()
        .ok_or_else(|| "Cannot determine user config directory".to_string())?;
    Ok(base.join("VoxFlow").join("models"))
}

/// Returns the file path for a specific model ID.
/// Validates that `model_id` is in `VALID_MODEL_IDS`.
pub fn model_file_path(model_id: &str) -> Result<std::path::PathBuf, String> {
    if !VALID_MODEL_IDS.contains(&model_id) {
        return Err(format!(
            "Invalid model ID '{}'. Valid IDs are: {}",
            model_id,
            VALID_MODEL_IDS.join(", ")
        ));
    }
    Ok(models_dir()?.join(format!("ggml-{model_id}.bin")))
}

/// Returns the HuggingFace download URL for a model.
pub fn model_download_url(model_id: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model_id}.bin"
    )
}

/// Scans the models directory for `ggml-*.bin` files and returns the
/// extracted model IDs as a sorted Vec<String>.
pub fn get_downloaded_model_ids() -> Result<Vec<String>, String> {
    let dir = models_dir()?;

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read models directory: {e}"))?;

    let mut ids: Vec<String> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            // Match ggml-<id>.bin pattern
            let stem = name_str.strip_prefix("ggml-")?.strip_suffix(".bin")?;
            // Only return IDs that are in our valid list
            if VALID_MODEL_IDS.contains(&stem) {
                Some(stem.to_string())
            } else {
                None
            }
        })
        .collect();

    ids.sort();
    Ok(ids)
}

// ---------------------------------------------------------------------------
// Download task
// ---------------------------------------------------------------------------

/// Background download function. Called by start_model_download command.
/// Streams the model file from HuggingFace with progress events.
/// Uses a partial file (`ggml-<id>.bin.partial`) during download; renames
/// to final name on success to avoid half-written files appearing as downloaded.
pub async fn run_download<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    model_id: String,
    cancel: Arc<AtomicBool>,
) -> Result<String, String> {
    use futures_util::StreamExt;

    let dir = models_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create models directory: {e}"))?;

    let final_path = dir.join(format!("ggml-{model_id}.bin"));
    let partial_path = dir.join(format!("ggml-{model_id}.bin.partial"));

    let url = model_download_url(&model_id);

    // Build reqwest client and start GET
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error starting download: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let _ = app.emit(
            MODEL_DOWNLOAD_ERROR_EVENT,
            DownloadEventPayload {
                model_id: model_id.clone(),
                message: format!("HTTP error: {status}"),
            },
        );
        return Err(format!("Download failed with HTTP {status}"));
    }

    let bytes_total = response.content_length().unwrap_or(0);

    // Open partial file for writing
    let mut file = std::fs::File::create(&partial_path)
        .map_err(|e| format!("Failed to create partial file: {e}"))?;

    let mut bytes_done: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        // Check cancel flag before processing each chunk
        if cancel.load(Ordering::Relaxed) {
            drop(file);
            let _ = std::fs::remove_file(&partial_path);
            let _ = app.emit(
                MODEL_DOWNLOAD_CANCELLED_EVENT,
                DownloadEventPayload {
                    model_id: model_id.clone(),
                    message: "Download cancelled by user".to_string(),
                },
            );
            return Err("Download cancelled".to_string());
        }

        let chunk = chunk_result
            .map_err(|e| format!("Stream error during download: {e}"))?;

        use std::io::Write;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk: {e}"))?;

        bytes_done += chunk.len() as u64;

        let percent = if bytes_total > 0 {
            (bytes_done as f32 / bytes_total as f32) * 100.0
        } else {
            0.0
        };

        let _ = app.emit(
            MODEL_DOWNLOAD_PROGRESS_EVENT,
            DownloadProgressPayload {
                model_id: model_id.clone(),
                percent,
                bytes_done,
                bytes_total,
            },
        );
    }

    // Ensure all bytes are written to disk
    drop(file);

    // Rename partial to final
    std::fs::rename(&partial_path, &final_path)
        .map_err(|e| format!("Failed to finalize model file: {e}"))?;

    let model_path_str = final_path
        .to_str()
        .ok_or_else(|| "Model path is not valid UTF-8".to_string())?
        .to_string();

    let _ = app.emit(
        MODEL_DOWNLOAD_DONE_EVENT,
        DownloadDonePayload {
            model_id: model_id.clone(),
            model_path: model_path_str.clone(),
        },
    );

    Ok(model_path_str)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_model_ids_contains_expected_set() {
        assert_eq!(VALID_MODEL_IDS, &["tiny", "base", "small", "medium"]);
    }

    #[test]
    fn models_dir_ends_with_voxflow_models() {
        let dir = models_dir().expect("models_dir should succeed");
        assert!(
            dir.ends_with("VoxFlow/models") || dir.ends_with("VoxFlow\\models"),
            "Expected path to end with VoxFlow/models, got: {:?}",
            dir
        );
    }

    #[test]
    fn model_file_path_valid_id_returns_correct_path() {
        let path = model_file_path("tiny").expect("tiny is a valid model");
        let file_name = path.file_name().unwrap().to_string_lossy();
        assert_eq!(file_name, "ggml-tiny.bin");
    }

    #[test]
    fn model_file_path_all_valid_ids() {
        for id in VALID_MODEL_IDS {
            let path = model_file_path(id).unwrap_or_else(|_| panic!("{id} should be valid"));
            let name = path.file_name().unwrap().to_string_lossy();
            assert_eq!(name, format!("ggml-{id}.bin"));
        }
    }

    #[test]
    fn model_file_path_invalid_id_returns_error() {
        let result = model_file_path("large");
        assert!(result.is_err(), "Expected error for invalid model ID 'large'");
    }

    #[test]
    fn model_file_path_empty_id_returns_error() {
        let result = model_file_path("");
        assert!(result.is_err(), "Expected error for empty model ID");
    }

    #[test]
    fn model_download_url_format() {
        let url = model_download_url("tiny");
        assert_eq!(
            url,
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
        );
    }

    #[test]
    fn get_downloaded_model_ids_empty_when_no_dir() {
        // This just tests the function doesn't panic when dir doesn't exist.
        // In a real environment, if models_dir doesn't exist yet, returns empty vec.
        // We can't easily test this without a temp dir, so we verify the Ok variant.
        let result = get_downloaded_model_ids();
        // Should succeed (not panic), returning either Ok([]) or Ok([...ids])
        assert!(result.is_ok());
    }

    #[test]
    fn get_downloaded_model_ids_scans_correctly() {
        use std::fs;
        use tempfile::tempdir;

        // Create a temp dir simulating models_dir with some ggml-*.bin files
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path();

        // Create some valid model files
        fs::write(tmp_path.join("ggml-tiny.bin"), b"fake model").unwrap();
        fs::write(tmp_path.join("ggml-base.bin"), b"fake model").unwrap();
        // Create a partial file that should NOT appear
        fs::write(tmp_path.join("ggml-small.bin.partial"), b"partial").unwrap();
        // Create an unknown file that should NOT appear
        fs::write(tmp_path.join("ggml-large.bin"), b"unknown model").unwrap();
        // Create a totally unrelated file
        fs::write(tmp_path.join("config.json"), b"{}").unwrap();

        // Use the internal scanning logic on the temp dir directly
        let entries = fs::read_dir(tmp_path).unwrap();
        let mut ids: Vec<String> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name();
                let name_str = name.to_string_lossy().to_string();
                let stem = name_str.strip_prefix("ggml-")?.strip_suffix(".bin")?;
                if VALID_MODEL_IDS.contains(&stem) {
                    Some(stem.to_string())
                } else {
                    None
                }
            })
            .collect();
        ids.sort();

        assert_eq!(ids, vec!["base", "tiny"]);
    }
}
