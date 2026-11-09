"""Edge TTS zh-CN-YunyangNeural synthesis and verified AAC cache."""
from __future__ import annotations

import asyncio
import hashlib
import json
import math
import re
import shutil
import struct
import subprocess
import tempfile
import time
import wave
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

EDGE_TTS_VERSION = 'edge-tts-yunyang-v1'
DEFAULT_EDGE_VOICE = 'zh-CN-YunyangNeural'


def split_narration(text: str, max_chars: int = 150) -> list[str]:
    """Bound narration segments without deleting sentence punctuation."""
    if max_chars < 1:
        raise ValueError('max_chars must be positive')
    result: list[str] = []
    current = ''
    for paragraph in text.splitlines():
        for sentence in re.findall(r'[^。！？!?]+[。！？!?]?|[。！？!?]', paragraph):
            while sentence:
                room = max_chars - len(current)
                current += sentence[:room]
                sentence = sentence[room:]
                if len(current) == max_chars:
                    result.append(current)
                    current = ''
    if current:
        result.append(current)
    return result


def validate_wav(path: Path) -> float:
    with wave.open(str(path), 'rb') as audio:
        if audio.getnchannels() != 1 or audio.getsampwidth() != 2 or audio.getframerate() != 24000:
            raise ValueError('expected 24kHz mono PCM16 audio')
        frames = audio.getnframes()
        if not frames:
            raise ValueError('empty audio')
        pcm = audio.readframes(frames)
    samples = [sample[0] for sample in struct.iter_unpack('<h', pcm)]
    rms = math.sqrt(sum(value * value for value in samples) / len(samples))
    if rms < 10:
        raise ValueError('silent audio')
    if sum(abs(value) >= 32760 for value in samples) / len(samples) > .02:
        raise ValueError('clipped audio')
    return frames / 24000


@dataclass(frozen=True)
class GeneratedAudio:
    path: Path
    duration_seconds: float
    sha256: str


def _edge_synthesize_mp3(text: str, voice: str, rate: str, destination: Path, timeout: int) -> None:
    import edge_tts

    async def _run() -> None:
        communicate = edge_tts.Communicate(text=text, voice=voice, rate=rate)
        await asyncio.wait_for(communicate.save(str(destination)), timeout=timeout)

    asyncio.run(_run())


class EdgeYunyangTts:
    def __init__(self, *, cache_dir: Path, voice: str = DEFAULT_EDGE_VOICE, rate: str = '+0%',
                 converter: str | None = None, run: Callable = subprocess.run,
                 synthesize_mp3: Callable[[str, str, str, Path, int], None] = _edge_synthesize_mp3,
                 timeout: int = 180, max_retries: int = 3, retry_delay: float = 1.0):
        self.cache_dir = cache_dir
        self.voice = voice.strip() or DEFAULT_EDGE_VOICE
        self.rate = rate.strip() or '+0%'
        self.converter = converter or shutil.which('ffmpeg')
        self.run = run
        self.synthesize_mp3 = synthesize_mp3
        self.timeout = timeout
        self.max_retries = max(1, int(max_retries))
        self.retry_delay = max(0.0, float(retry_delay))

    def synthesize(self, text: str) -> GeneratedAudio:
        if not text.strip() or len(text) > 6000:
            raise ValueError('narration must contain 1..6000 characters')
        if not self.converter:
            raise ValueError('install ffmpeg for Edge TTS audio encoding')
        identity = json.dumps([EDGE_TTS_VERSION, self.voice, self.rate, text], ensure_ascii=False)
        key = hashlib.sha256(identity.encode()).hexdigest()
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        metadata = self.cache_dir / f'{key}.json'
        destination = self.cache_dir / f'{key}.m4a'
        if metadata.exists() and destination.exists():
            try:
                cached = json.loads(metadata.read_text())
                digest = hashlib.sha256(destination.read_bytes()).hexdigest()
                if cached['sha256'] == digest and cached['duration_seconds'] > 0:
                    return GeneratedAudio(destination, cached['duration_seconds'], digest)
            except (ValueError, KeyError, TypeError):
                pass
        with tempfile.TemporaryDirectory(dir=self.cache_dir) as temporary:
            directory = Path(temporary)
            mp3 = directory / 'voice.mp3'
            wav = directory / 'voice.wav'
            encoded = directory / 'voice.m4a'
            last_error: Exception | None = None
            for attempt in range(self.max_retries):
                try:
                    mp3.unlink(missing_ok=True)
                    self.synthesize_mp3(text, self.voice, self.rate, mp3, self.timeout)
                    if not mp3.exists() or not mp3.stat().st_size:
                        raise ValueError('Edge TTS produced an empty MP3 file')
                    last_error = None
                    break
                except Exception as error:
                    last_error = error
                    if attempt + 1 < self.max_retries and self.retry_delay > 0:
                        time.sleep(self.retry_delay * (2 ** attempt))
            if last_error is not None:
                raise last_error
            self.run([self.converter, '-y', '-loglevel', 'error', '-i', str(mp3),
                      '-ar', '24000', '-ac', '1', '-c:a', 'pcm_s16le', str(wav)],
                     check=True, timeout=120, capture_output=True)
            duration = validate_wav(wav)
            self.run([self.converter, '-y', '-loglevel', 'error', '-i', str(wav),
                      '-c:a', 'aac', '-b:a', '64k', '-movflags', '+faststart', str(encoded)],
                     check=True, timeout=120, capture_output=True)
            if not encoded.exists() or not encoded.stat().st_size:
                raise ValueError('AAC encoding produced an empty file')
            digest = hashlib.sha256(encoded.read_bytes()).hexdigest()
            encoded.replace(destination)
            temporary_metadata = directory / 'metadata.json'
            temporary_metadata.write_text(json.dumps({
                'sha256': digest,
                'duration_seconds': duration,
                'voice': self.voice,
                'rate': self.rate,
                'tts_version': EDGE_TTS_VERSION,
            }))
            temporary_metadata.replace(metadata)
        return GeneratedAudio(destination, duration, digest)
