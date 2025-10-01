# Full Flask app providing Speech-to-Text (STT) and fast Text-to-Speech (TTS).
# - STT: accepts WAV uploads from the browser and transcribes with SpeechRecognition (Google Web Speech API).
# - TTS: super fast on Linux using the native `espeak` CLI (WAV on stdout). Falls back to pyttsx3 on non-Linux or if espeak is unavailable.

import io
import os
import tempfile
import platform
import subprocess
from functools import lru_cache

from flask import Flask, request, send_file, jsonify
import speech_recognition as sr

import asyncio

from uprotocol.communication.inmemoryrpcclient import InMemoryRpcClient
from uprotocol.communication.upayload import UPayload
from uprotocol.v1.uattributes_pb2 import (
    UPayloadFormat,
)
from uprotocol.v1.uri_pb2 import UUri

from up_transport_zenoh.examples import common_uuri
from up_transport_zenoh.examples.common_uuri import create_method_uri, get_zenoh_default_config
from up_transport_zenoh.uptransportzenoh import UPTransportZenoh

source = UUri(authority_name="voice-command", ue_id=18)
transport = UPTransportZenoh.new(get_zenoh_default_config(), source)


async def send_rpc_request_to_zenoh(data):
    # create uuri
    uuri = create_method_uri()
    # create UPayload
    payload = UPayload(format=UPayloadFormat.UPAYLOAD_FORMAT_TEXT, data=bytes([ord(c) for c in data]))
    # invoke RPC method
    common_uuri.logging.debug(f"Send request to {uuri}")
    rpc_client = InMemoryRpcClient(transport)
    response_payload = await rpc_client.invoke_method(uuri, payload)
    common_uuri.logging.debug(f"Response payload {response_payload}")

# -----------------------------
# Global recognizer (STT)
# -----------------------------
r = sr.Recognizer()

# -----------------------------
# App (serves static/index.html)
# -----------------------------
app = Flask(__name__, static_url_path="/static", static_folder="static")


# -----------------------------
# Helper: platform checks
# -----------------------------
def _is_linux() -> bool:
    return platform.system().lower() == "linux"


# -----------------------------
# TTS via espeak (Linux) – FAST
# Returns WAV bytes directly from espeak --stdout
# -----------------------------
@lru_cache(maxsize=64)
def _espeak_tts_bytes_cached(text: str, voice: str = "en-us", rate: int = 150) -> bytes:
    """
    Render TTS using espeak CLI and return WAV bytes.
    This is very fast and avoids filesystem I/O.
    """
    cmd = ["espeak", "-v", voice, "-s", str(rate), "--stdout", text]
    try:
        res = subprocess.run(
            cmd,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=10,  # keep short to avoid long hangs
        )
        return res.stdout
    except FileNotFoundError:
        # espeak not installed / not in PATH
        raise RuntimeError("espeak is not installed or not found in PATH.")
    except subprocess.TimeoutExpired:
        raise RuntimeError("TTS timeout (espeak) – text too long or system busy?")
    except subprocess.CalledProcessError as e:
        msg = e.stderr.decode(errors="ignore")[:200]
        raise RuntimeError(f"TTS failed (espeak): {msg}")
    
# -----------------------------
# Routes
# -----------------------------
@app.route("/")
def root():
    # Serves your static HTML page
    return app.send_static_file("index.html")


@app.route("/stt", methods=["POST"])
def stt():
    """
    Expects an audio file as multipart/form-data:
      field name: 'audio'
      Format: WAV (PCM 16-bit, mono, 16 kHz) – that's what the provided index.html generates.
    """
    if "audio" not in request.files:
        return jsonify({"error": "No 'audio' file part found"}), 400

    f = request.files["audio"]
    if f.filename == "":
        return jsonify({"error": "Empty filename"}), 400

    # Load in-memory file into SpeechRecognition
    audio_bytes = f.read()
    audio_file = io.BytesIO(audio_bytes)

    try:
        with sr.AudioFile(audio_file) as source:
            audio_data = r.record(source)
        text = r.recognize_google(audio_data, language="en-US")

        print("text: " + text)
        asyncio.run(send_rpc_request_to_zenoh(text))

        return jsonify({"text": text})
    except sr.UnknownValueError:
        return jsonify({"text": "", "message": "Could not understand audio"}), 200
    except sr.RequestError as e:
        return jsonify({"error": f"STT API error: {e}"}), 502
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@app.route("/tts", methods=["POST"])
def tts():
    """
    Expects JSON: {"text": "...", "voice": "en-us", "rate": 150}
    Returns a WAV stream.
    On Linux: uses espeak (very fast).
    Otherwise or if espeak is not available: falls back to pyttsx3.
    """
    data = request.get_json(silent=True) or {}
    text = (data.get("text") or "").strip()
    if not text:
        return jsonify({"error": "No text provided"}), 400

    # Input sanity limits for web usage
    if len(text) > 2000:
        return jsonify({"error": "Text too long (max 2000 chars)."}), 413

    voice = data.get("voice", "en-us")
    try:
        rate = int(data.get("rate", 150))
    except Exception:
        rate = 150

    try:
        if _is_linux():
            # Primary fast path: espeak -> WAV bytes
            wav_bytes = _espeak_tts_bytes_cached(text, voice=voice, rate=rate)
            return send_file(
                io.BytesIO(wav_bytes),
                mimetype="audio/wav",
                as_attachment=False,
                download_name="tts.wav",
            )
        else:
            # Fallback: pyttsx3 (file-based)
            engine = _get_pyttsx3()
            with tempfile.NamedTemporaryFile(delete=False, suffix=".wav") as tmp:
                tmp_path = tmp.name
            try:
                engine.setProperty("rate", rate)
                engine.save_to_file(text, tmp_path)
                engine.runAndWait()
                return send_file(
                    tmp_path,
                    mimetype="audio/wav",
                    as_attachment=False,
                    download_name="tts.wav",
                )
            finally:
                try:
                    os.remove(tmp_path)
                except OSError:
                    pass

    except RuntimeError as e:
        return jsonify({"error": str(e)}), 500
    except Exception as e:
        return jsonify({"error": f"Unexpected TTS error: {e}"}), 500


# -----------------------------
# Main
# -----------------------------
if __name__ == "__main__":
    # Start the Flask development server
    app.run(host="0.0.0.0", port=5000, debug=True)    
