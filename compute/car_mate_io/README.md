### CarMate – Speech-to-Text (STT) & Text-to-Speech (TTS) Demo

Voice Command Server — STT + ultra-fast TTS (Flask)

A tiny Flask app that serves:

  Speech-to-Text (STT): accepts browser-recorded WAV uploads and transcribes via SpeechRecognition (Google Web Speech API).

  Text-to-Speech (TTS): on Linux, streams WAV directly from espeak (super fast; output cached).

  RPC bridge: forwards text to a Zenoh/uProtocol RPC service and returns the response.

Features

  STT: expects PCM 16-bit mono 16 kHz WAV; uses Google Web Speech (internet required).

  TTS: calls espeak --stdout → streams WAV without touching disk. LRU-cached (64 entries).

  RPC: async call over uProtocol + Zenoh using InMemoryRpcClient.

  Static UI: serves static/index.html at /.

## ⚙️ Requirements

- Python 3.10+ (tested with 3.12)  
run the docker file after

# Start Container
docker build -t car_mate_io:latest .
docker run --rm -it --net=host car_mate_io

## 🎙️ Enable Microphone

Methods to enable mic use in a non-localhost development web app:

Use HTTPS for your dev server:

Set up your development server with HTTPS and a valid or self-signed certificate. This is the recommended and most secure approach.
Temporarily treat HTTP as secure in your browser:

For Chromium-based browsers (Chrome, Edge, Brave, etc.), you can enable a flag:
- Go to chrome://flags/#unsafely-treat-insecure-origin-as-secure in the browser.
- Add your HTTP development URL (e.g., http://192.168.x.x:port).
- Restart the browser and it will treat that origin as secure, allowing mic access.
  Note: This is only for development and should not be used in production as it bypasses security.
