#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::sync::LazyLock;

use axum::body::Body;
use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use uuid::Uuid;

const HTML_CONTENT: &str = include_str!("../assets/index.html");

static YT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(https?://)?(www\.)?(youtube\.com/watch\?v=|youtu\.be/|youtube\.com/shorts/)[\w\-]{11}",
    )
    .unwrap()
});

#[derive(Deserialize)]
struct PreviewRequest {
    url: String,
}

#[derive(Serialize)]
struct PreviewResponse {
    title: String,
    thumbnail: String,
    channel: String,
    duration: String,
}

#[derive(Deserialize)]
struct DownloadRequest {
    url: String,
    format: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn main() {
    // Log errors to TEMP directory (always writable)
    let log_path = std::env::temp_dir().join("yt-downloader-crash.log");
    let log_path_clone = log_path.clone();
    std::panic::set_hook(Box::new(move |info| {
        let _ = std::fs::write(&log_path_clone, format!("{info}"));
    }));

    fn log_error(msg: &str) {
        let path = std::env::temp_dir().join("yt-downloader-crash.log");
        let _ = std::fs::write(&path, msg);
    }

    let (tx, rx) = std::sync::mpsc::channel();

    // Spawn HTTP server in a background thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async move {
            let app = Router::new()
                .route("/", get(serve_index))
                .route("/preview", post(handle_preview))
                .route("/download", post(handle_download));

            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("Failed to bind server");
            let port = listener.local_addr().unwrap().port();
            tx.send(port).unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    });

    let port = rx.recv().expect("Failed to get server port");

    // Create native window with webview
    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new()
        .with_title("YT Downloader")
        .with_inner_size(LogicalSize::new(560.0, 720.0))
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => {
            log_error(&format!("Failed to create window: {e}"));
            return;
        }
    };

    // WebView2 needs a writable data directory — use LOCALAPPDATA
    let data_dir = {
        let base = std::env::var("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir());
        base.join("YTDownloader").join("webview")
    };
    let mut web_context = wry::WebContext::new(Some(data_dir));

    let _webview = match wry::WebViewBuilder::new(&window)
        .with_url(&format!("http://127.0.0.1:{port}"))
        .with_web_context(&mut web_context)
        .build()
    {
        Ok(wv) => wv,
        Err(e) => {
            log_error(&format!("Failed to create webview: {e}"));
            return;
        }
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}

// --- Handlers ---

async fn serve_index() -> Html<&'static str> {
    Html(HTML_CONTENT)
}

async fn handle_preview(Json(body): Json<PreviewRequest>) -> Response {
    let url = body.url.trim().to_string();
    if !YT_REGEX.is_match(&url) {
        return error_response(StatusCode::BAD_REQUEST, "Please provide a valid YouTube URL");
    }

    match tokio::task::spawn_blocking(move || get_video_info(&url))
        .await
        .unwrap()
    {
        Ok(info) => Json(info).into_response(),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_download(Json(body): Json<DownloadRequest>) -> Response {
    let url = body.url.trim().to_string();
    let format = body.format.trim().to_lowercase();

    if !matches!(format.as_str(), "mp3" | "mp4") {
        return error_response(StatusCode::BAD_REQUEST, "Format must be mp3 or mp4");
    }
    if !YT_REGEX.is_match(&url) {
        return error_response(StatusCode::BAD_REQUEST, "Please provide a valid YouTube URL");
    }

    match tokio::task::spawn_blocking(move || download_video(&url, &format))
        .await
        .unwrap()
    {
        Ok((filename, bytes)) => Response::builder()
            .header("content-type", "application/octet-stream")
            .header(
                "content-disposition",
                format!("attachment; filename=\"{filename}\""),
            )
            .body(Body::from(bytes))
            .unwrap(),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
        .into_response()
}

// --- yt-dlp wrappers ---

fn app_bin_dir() -> std::path::PathBuf {
    // Look for bundled bin/ directory next to the exe
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    exe_dir.join("bin")
}

fn find_yt_dlp() -> String {
    let bundled = app_bin_dir().join("yt-dlp.exe");
    if bundled.exists() {
        bundled.to_string_lossy().to_string()
    } else {
        "yt-dlp".to_string() // fall back to PATH
    }
}

fn find_ffmpeg_dir() -> Option<String> {
    let bin_dir = app_bin_dir().join("ffmpeg");
    if bin_dir.exists() {
        // Find ffmpeg.exe recursively inside the extracted folder
        for entry in walkdir(&bin_dir) {
            if entry.file_name().map(|n| n == "ffmpeg.exe").unwrap_or(false) {
                if let Some(parent) = entry.parent() {
                    return Some(parent.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(walkdir(&path));
            } else {
                results.push(path);
            }
        }
    }
    results
}

fn yt_dlp_cmd() -> Command {
    let mut cmd = Command::new(find_yt_dlp());
    // If bundled ffmpeg exists, tell yt-dlp where to find it
    if let Some(ffmpeg_dir) = find_ffmpeg_dir() {
        cmd.args(["--ffmpeg-location", &ffmpeg_dir]);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

fn get_video_info(url: &str) -> Result<PreviewResponse, String> {
    let output = yt_dlp_cmd()
        .args(["--dump-json", "--no-warnings", "--color", "never", url])
        .output()
        .map_err(|e| format!("Failed to run yt-dlp: {e}. Is yt-dlp installed and on PATH?"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {stderr}"));
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("Failed to parse info: {e}"))?;

    let duration_s = json["duration"].as_f64().unwrap_or(0.0) as u64;
    let seconds = duration_s % 60;
    let total_minutes = duration_s / 60;
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;

    let duration = if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    };

    Ok(PreviewResponse {
        title: json["title"].as_str().unwrap_or("Unknown").to_string(),
        thumbnail: json["thumbnail"].as_str().unwrap_or("").to_string(),
        channel: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
        duration,
    })
}

fn download_video(url: &str, format: &str) -> Result<(String, Vec<u8>), String> {
    let job_dir = std::env::temp_dir().join(format!("yt-dl-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&job_dir).map_err(|e| format!("Failed to create temp dir: {e}"))?;

    let output_template = job_dir.join("%(title)s.%(ext)s");
    let output_str = output_template.to_string_lossy().to_string();

    let mut cmd = yt_dlp_cmd();
    cmd.args(["--no-warnings", "--color", "never"]);

    match format {
        "mp3" => {
            cmd.args([
                "-f",
                "bestaudio/best",
                "-x",
                "--audio-format",
                "mp3",
                "--audio-quality",
                "192K",
                "-o",
                &output_str,
                url,
            ]);
        }
        "mp4" => {
            cmd.args([
                "-f",
                "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
                "--merge-output-format",
                "mp4",
                "-o",
                &output_str,
                url,
            ]);
        }
        _ => return Err("Invalid format".to_string()),
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run yt-dlp: {e}. Is yt-dlp installed and on PATH?"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_dir_all(&job_dir);
        return Err(format!("Download error: {stderr}"));
    }

    let target_ext = format!(".{format}");
    let file = std::fs::read_dir(&job_dir)
        .map_err(|e| format!("Failed to read dir: {e}"))?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .to_lowercase()
                .ends_with(&target_ext)
        })
        .ok_or_else(|| "No output file found".to_string())?;

    let filename = file.file_name().to_string_lossy().to_string();
    let bytes = std::fs::read(file.path()).map_err(|e| format!("Failed to read file: {e}"))?;

    let _ = std::fs::remove_dir_all(&job_dir);

    Ok((filename, bytes))
}
