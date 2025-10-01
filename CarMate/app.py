import speech_recognition as sr
import pyttsx3
import time

# --- Webserver: Flask-Endpunkte für STT/TTS ---
from flask import Flask, request, send_file, jsonify
from datetime import datetime
import io
import wave
import os
import tempfile

import platform

_engine = None

def get_tts_engine():
    global _engine
    if _engine is not None:
        return _engine
    try:
        # unter Linux explizit den espeak-Treiber versuchen
        if platform.system().lower() == "linux":
            _engine = pyttsx3.init(driverName="espeak")
        else:
            _engine = pyttsx3.init()  # macOS: nsss, Windows: sapi5
        _engine.setProperty('rate', 150)
        _engine.setProperty('volume', 1.0)
        return _engine
    except OSError as e:
        # typisch: libespeak.so.1 fehlt
        raise RuntimeError(
            "TTS-Engine konnte nicht initialisiert werden. "
            "Unter Linux bitte 'espeak' und 'libespeak1' installieren. "
            f"Systemfehler: {e}"
        )

# Initialize Speech-to-Text
r = sr.Recognizer()
mic_index = 2  # your headset microphone index

def listen_for_command():
    """Listen for a command after the hotword"""
    with sr.Microphone(device_index=mic_index) as source:
        print("Speak your command …")
        audio = r.listen(source)
        try:
            command = r.recognize_google(audio, language="en-US")
            print("Recognized:", command)
            return command
        except sr.UnknownValueError:
            print("❌ Could not understand audio.")
            engine.say("Sorry, I did not understand you.")
            engine.runAndWait()
        except sr.RequestError as e:
            print(f"❌ API error: {e}")
            engine.say("There was an error processing your request.")
            engine.runAndWait()
    return None

app = Flask(__name__, static_url_path="/static", static_folder="static")

@app.route("/")
def root():
    # Liefert deine statische HTML-Seite
    return app.send_static_file("index.html")

@app.route("/stt", methods=["POST"])
def stt():
    """
    Erwartet eine Audiodatei als multipart/form-data:
      field name: 'audio'
      Format: WAV (PCM 16-bit, mono, 16kHz) – genau so erzeugt die index.html unten die Datei.
    """
    if "audio" not in request.files:
        return jsonify({"error": "No 'audio' file part found"}), 400

    f = request.files["audio"]
    if f.filename == "":
        return jsonify({"error": "Empty filename"}), 400

    # Datei im Speicher -> SpeechRecognition einlesen
    audio_bytes = f.read()
    audio_file = io.BytesIO(audio_bytes)

    try:
        with sr.AudioFile(audio_file) as source:
            audio_data = r.record(source)
        text = r.recognize_google(audio_data, language="en-US")
        return jsonify({"text": text})
    except sr.UnknownValueError:
        return jsonify({"text": "", "message": "Could not understand audio"}), 200
    except sr.RequestError as e:
        return jsonify({"error": f"STT API error: {e}"}), 502
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route("/tts", methods=["POST"])
def tts():
    data = request.get_json(silent=True) or {}
    text = data.get("text", "").trim() if hasattr(str, "trim") else data.get("text","").strip()
    if not text:
        return jsonify({"error": "No text provided"}), 400

    try:
        engine = get_tts_engine()
    except RuntimeError as e:
        return jsonify({"error": str(e)}), 500

    with tempfile.NamedTemporaryFile(delete=False, suffix=".wav") as tmp:
        tmp_path = tmp.name
    try:
        engine.save_to_file(text, tmp_path)
        engine.runAndWait()
        return send_file(tmp_path, mimetype="audio/wav", as_attachment=False, download_name="tts.wav")
    finally:
        try: os.remove(tmp_path)
        except OSError: pass


if __name__ == "__main__":
    # Anstatt der Endlos-Mikrofon-Schleife jetzt Webserver starten
    app.run(host="0.0.0.0", port=5000, debug=True)
