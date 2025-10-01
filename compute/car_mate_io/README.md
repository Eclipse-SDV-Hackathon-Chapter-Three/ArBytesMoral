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
run the docker file after
docker build -t car_mate_io .



python3 compute/car_mate_io/app.py
Opens at: http://localhost:5000