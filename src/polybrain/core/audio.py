"""

Tools to speak and record audio. Includes LLM utils

"""

import os
import wave
import pyaudio
import playsound
import numpy as np
import Levenshtein
from loguru import logger
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from polybrain.core.client import Client


class Audio:
    # Parameters
    FORMAT = pyaudio.paInt16  # Audio format (16-bit)
    CHANNELS = 1  # Number of audio channels
    RATE = 44100  # Sample rate (samples per second)
    CHUNK = 1024  # Buffer size (number of samples per chunk)
    SILENCE_THRESHOLD = 1000  # Threshold for considering a chunk as silence
    SILENCE_DURATION = (
        2  # Duration in seconds to stop recording after silence is detected
    )
    INITIAL_GRACE_PERIOD = 5  # Initial grace period in seconds

    def __init__(self, client: "Client") -> None:
        self.client = client
        self.tts_log: list[str] = []

    @staticmethod
    def is_silent(data_chunk, threshold):
        # Convert the data chunk to numpy array
        data = np.frombuffer(data_chunk, dtype=np.int16)
        # Compute the average amplitude
        amplitude = np.abs(data).mean()
        # Return if the amplitude is below the threshold
        return amplitude < threshold

    @classmethod
    def record_audio(cls) -> bytes:
        """Records a sample of audio from the user's microphone

        Returns:
            The Bytes of the audio mp3
        """
        playsound.playsound("assets/start-record.mp3")

        p = pyaudio.PyAudio()
        stream = p.open(
            format=cls.FORMAT,
            channels=cls.CHANNELS,
            rate=cls.RATE,
            input=True,
            frames_per_buffer=cls.CHUNK,
        )

        logger.debug("Recording...")

        frames = []
        silence_count = 0
        grace_period_chunks = int(cls.INITIAL_GRACE_PERIOD * cls.RATE / cls.CHUNK)
        total_chunks = 0

        while True:
            data = stream.read(cls.CHUNK)
            frames.append(data)
            total_chunks += 1

            # Start silence detection after the initial grace period
            if total_chunks > grace_period_chunks:
                if cls.is_silent(data, cls.SILENCE_THRESHOLD):
                    silence_count += 1
                else:
                    silence_count = 0

                if silence_count > (cls.RATE / cls.CHUNK * cls.SILENCE_DURATION):
                    logger.debug("Silence detected, stopping recording.")
                    break

        logger.debug("Recording stopped.")
        playsound.playsound("assets/end-record.mp3")

        stream.stop_stream()
        stream.close()
        p.terminate()

        return b"".join(frames)

    @staticmethod
    def save_wave_file(
        filename: str, data: bytes, sample_width: int, channels: int, rate: int
    ) -> None:
        """
        Save audio data to a WAV file

        Args:
            filename (str): The name of the file where the audio data will be saved.
            data (bytes): The audio data to be written to the file.
            sample_width (int): The number of bytes per sample (e.g., 1 for 8-bit, 2 for 16-bit audio).
            channels (int): The number of audio channels (e.g., 1 for mono, 2 for stereo).
            rate (int): The sampling rate (number of samples per second) of the audio.

        Returns:
            None
        """

        wf = wave.open(filename, "wb")
        wf.setnchannels(channels)
        wf.setsampwidth(sample_width)
        wf.setframerate(rate)
        wf.writeframes(data)
        wf.close()

    def translate_audio_file(self, filename: str) -> str:
        """Converts an audio file into a transcript

        Args:
            filename: The audio file to parse

        Returns:
            The transcript of the audio file
        """

        audio_file = open(filename, "rb")
        translation = self.client.openai_client.audio.translations.create(
            model="whisper-1", file=audio_file
        )

        return translation.text

    def get_audio_input(self) -> str:
        """Gets input from a user's microphone

        Returns:
            A string of what the user said
        """

        audio_data = self.record_audio()
        self.save_wave_file(
            "output.wav",
            audio_data,
            pyaudio.PyAudio().get_sample_size(self.FORMAT),
            self.CHANNELS,
            self.RATE,
        )
        transcript = self.translate_audio_file("output.wav")
        os.remove("output.wav")
        return transcript

    def speak_text(self, text: str) -> None:
        """Runs TTS on a certain text

        Args:
            text: The text to speak
        """

        # prevent duplicate messages
        if self.tts_log:
            distance = Levenshtein.distance(self.tts_log[0], text)
            if distance < 10:
                return

        files = []

        for i, segment in enumerate(text.split("\n")):

            if segment.strip() == "":
                continue

            with self.client.openai_client.audio.speech.with_streaming_response.create(
                model="tts-1",
                voice=self.client.settings["tts_voice"],
                input=segment,
            ) as response:
                file = f"speech[{i}].mp3"
                print(f"Loading {file}")
                response.stream_to_file(file)
                files.append(file)

        for file in files:
            playsound.playsound(file)
            os.remove(file)
