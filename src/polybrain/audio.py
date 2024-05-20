import pyaudio
import numpy as np
from openai import OpenAI
import playsound
import os

# TODO: Move this to __init__
import os 
os.chdir(os.path.dirname(__file__))

client = OpenAI()

# Parameters
FORMAT = pyaudio.paInt16  # Audio format (16-bit)
CHANNELS = 1              # Number of audio channels
RATE = 44100              # Sample rate (samples per second)
CHUNK = 1024              # Buffer size (number of samples per chunk)
SILENCE_THRESHOLD = 1000  # Threshold for considering a chunk as silence
SILENCE_DURATION = 2      # Duration in seconds to stop recording after silence is detected
INITIAL_GRACE_PERIOD = 5  # Initial grace period in seconds

def is_silent(data_chunk, threshold):
    # Convert the data chunk to numpy array
    data = np.frombuffer(data_chunk, dtype=np.int16)
    # Compute the average amplitude
    amplitude = np.abs(data).mean()
    # Return if the amplitude is below the threshold
    return amplitude < threshold

def record_audio():

    playsound.playsound("assets/start-record.mp3")


    p = pyaudio.PyAudio()
    stream = p.open(format=FORMAT,
                    channels=CHANNELS,
                    rate=RATE,
                    input=True,
                    frames_per_buffer=CHUNK)

    print("Recording...")

    frames = []
    silence_count = 0
    grace_period_chunks = int(INITIAL_GRACE_PERIOD * RATE / CHUNK)
    total_chunks = 0

    while True:
        data = stream.read(CHUNK)
        frames.append(data)
        total_chunks += 1

        # Start silence detection after the initial grace period
        if total_chunks > grace_period_chunks:
            if is_silent(data, SILENCE_THRESHOLD):
                silence_count += 1
            else:
                silence_count = 0

            if silence_count > (RATE / CHUNK * SILENCE_DURATION):
                print("Silence detected, stopping recording.")
                break

    print("Recording stopped.")
    playsound.playsound("assets/end-record.mp3")

    stream.stop_stream()
    stream.close()
    p.terminate()

    return b''.join(frames)

def save_wave_file(filename, data, sample_width, channels, rate):
    import wave

    wf = wave.open(filename, 'wb')
    wf.setnchannels(channels)
    wf.setsampwidth(sample_width)
    wf.setframerate(rate)
    wf.writeframes(data)
    wf.close()

def translate_audio_file(filename: str) -> str:
    """Converts an audio file into a transcript"""

    audio_file= open(filename, "rb")
    translation = client.audio.translations.create(
    model="whisper-1", 
    file=audio_file
    )

    return translation.text

def get_audio_input() -> str:
    """Gets input from a user's microphone
    
    Returns:
        A string of what the user said
    """

    audio_data = record_audio()
    save_wave_file("output.wav", audio_data, pyaudio.PyAudio().get_sample_size(FORMAT), CHANNELS, RATE)
    transcript = translate_audio_file("output.wav")
    os.remove("output.wav")
    return transcript

def speak_text(text: str):
    """Runs TTS on a certain text"""

    files = [""]

    for i, segment in enumerate(text.split("\n")):

        if segment.strip() == "":
            continue

        with client.audio.speech.with_streaming_response.create(
        model="tts-1",
        voice="alloy",
        input=segment,
        ) as response:
            file = f"speech[{i}].mp3"
            print(f"Loading {file}")
            response.stream_to_file(file)
            files.append(file)

    for file in files:
            playsound.playsound(file)
            os.remove(file)
