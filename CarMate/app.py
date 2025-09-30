import speech_recognition as sr
import pyttsx3
import time

# Initialize Text-to-Speech engine
engine = pyttsx3.init()
engine.setProperty('rate', 150)  # speaking speed
engine.setProperty('volume', 1.0)  # volume

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

def main():
    print("Starting voice assistant …")
    while True:
        with sr.Microphone(device_index=mic_index) as source:
            print("… listening for hotword 'Hey vehicle' …")
            audio = r.listen(source)
            try:
                text = r.recognize_google(audio, language="en-US")
                print("Recognized:", text)
                if "hey vehicle" in text.lower():
                    engine.say("Yes, I am listening")
                    engine.runAndWait()
                    command = listen_for_command()
                    if command:
                        # Here you can add logic, e.g., vehicle functions
                        engine.say("Command detected")
                        engine.say(f"You said: {command}")
                        engine.runAndWait()
                else:
                    print("Hotword not detected …")
            except sr.UnknownValueError:
                pass  # continue listening
            except sr.RequestError as e:
                print(f"❌ API error: {e}")
                engine.say("There was an error processing your request.")
                engine.runAndWait()
        time.sleep(0.2)  # short pause to reduce CPU load

if __name__ == "__main__":
    main()
