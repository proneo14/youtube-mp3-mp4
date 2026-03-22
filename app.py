import os
import re
import uuid
import shutil
from flask import Flask, request, jsonify, send_file, render_template
import yt_dlp
import imageio_ffmpeg

FFMPEG_PATH = imageio_ffmpeg.get_ffmpeg_exe()

app = Flask(__name__)

DOWNLOAD_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "downloads")
os.makedirs(DOWNLOAD_DIR, exist_ok=True)

YOUTUBE_URL_RE = re.compile(
    r"^(https?://)?(www\.)?(youtube\.com/watch\?v=|youtu\.be/|youtube\.com/shorts/)[\w\-]{11}"
)


@app.route("/")
def index():
    return render_template("index.html")


@app.route("/preview", methods=["POST"])
def preview():
    data = request.get_json()
    url = data.get("url", "").strip()

    if not url or not YOUTUBE_URL_RE.match(url):
        return jsonify({"error": "Please provide a valid YouTube URL"}), 400

    try:
        ydl_opts = {"quiet": True, "no_warnings": True, "skip_download": True}
        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            info = ydl.extract_info(url, download=False)

        duration_s = info.get("duration", 0)
        minutes, seconds = divmod(int(duration_s), 60)
        hours, minutes = divmod(minutes, 60)
        if hours:
            duration_str = f"{hours}:{minutes:02d}:{seconds:02d}"
        else:
            duration_str = f"{minutes}:{seconds:02d}"

        return jsonify({
            "title": info.get("title", "Unknown"),
            "thumbnail": info.get("thumbnail", ""),
            "channel": info.get("uploader", "Unknown"),
            "duration": duration_str,
        })
    except yt_dlp.utils.DownloadError as e:
        return jsonify({"error": f"Could not fetch video info: {str(e)}"}), 400
    except Exception as e:
        return jsonify({"error": f"Something went wrong: {str(e)}"}), 500


@app.route("/download", methods=["POST"])
def download():
    data = request.get_json()
    url = data.get("url", "").strip()
    fmt = data.get("format", "mp3").strip().lower()

    if fmt not in ("mp3", "mp4"):
        return jsonify({"error": "Format must be mp3 or mp4"}), 400

    if not url or not YOUTUBE_URL_RE.match(url):
        return jsonify({"error": "Please provide a valid YouTube URL"}), 400

    job_id = uuid.uuid4().hex
    job_dir = os.path.join(DOWNLOAD_DIR, job_id)
    os.makedirs(job_dir, exist_ok=True)

    try:
        if fmt == "mp3":
            ydl_opts = {
                "format": "bestaudio/best",
                "outtmpl": os.path.join(job_dir, "%(title)s.%(ext)s"),
                "ffmpeg_location": FFMPEG_PATH,
                "postprocessors": [
                    {
                        "key": "FFmpegExtractAudio",
                        "preferredcodec": "mp3",
                        "preferredquality": "192",
                    }
                ],
                "quiet": True,
                "no_warnings": True,
            }
        else:
            ydl_opts = {
                "format": "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
                "outtmpl": os.path.join(job_dir, "%(title)s.%(ext)s"),
                "ffmpeg_location": FFMPEG_PATH,
                "merge_output_format": "mp4",
                "quiet": True,
                "no_warnings": True,
            }

        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            ydl.download([url])

        # Find the resulting file
        files = os.listdir(job_dir)
        if not files:
            return jsonify({"error": "Download failed — no file produced"}), 500

        target_ext = f".{fmt}"
        result_file = next(
            (f for f in files if f.lower().endswith(target_ext)), files[0]
        )
        file_path = os.path.join(job_dir, result_file)

        return send_file(file_path, as_attachment=True, download_name=result_file)

    except yt_dlp.utils.DownloadError as e:
        return jsonify({"error": f"Download error: {str(e)}"}), 400
    except Exception as e:
        return jsonify({"error": f"Something went wrong: {str(e)}"}), 500
    finally:
        # Clean up after sending (best-effort)
        try:
            shutil.rmtree(job_dir, ignore_errors=True)
        except Exception:
            pass


if __name__ == "__main__":
    app.run(debug=True, port=5000)
