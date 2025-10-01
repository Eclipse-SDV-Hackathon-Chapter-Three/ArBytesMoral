# CarMate – Speech-to-Text (STT) & Text-to-Speech (TTS) Demo

A simple **Flask web application** that allows:

- 🎙️ **Speech-to-Text (STT)**  
  Record audio in the browser → upload to server → transcribed using [SpeechRecognition](https://pypi.org/project/SpeechRecognition/).

- 🔊 **Text-to-Speech (TTS)**  
  Enter text in the browser → sent to server → spoken by [pyttsx3](https://pypi.org/project/pyttsx3/) and returned as a WAV file.

The web frontend (`static/index.html`) uses the browser microphone to record audio and interact with the Flask API.

---

## ⚙️ Requirements

- Python 3.10+ (tested with 3.12)  
- System dependencies for TTS on Linux:
  ```bash
  sudo apt update
  sudo apt install espeak libespeak1 espeak-data
🚀 Setup
Clone repository & move into folder

Create virtual environment
bash

python3 -m venv .venv
source .venv/bin/activate   # Linux / macOS
# On Windows (PowerShell):
# .venv\Scripts\activate

Install Python dependencies
bash

pip install -r CarMate/requirements.txt
▶️ Running the App
Run with Python (default Flask dev server)
bash

python3 compute/car_mate_io/app.py
Opens at: http://localhost:5000