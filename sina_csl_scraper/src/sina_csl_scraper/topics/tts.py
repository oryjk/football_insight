"""Local Qwen inference and verified AAC cache; no remote TTS service."""
from __future__ import annotations

import hashlib
import json
import math
import os
import re
import shutil
import struct
import subprocess
import tempfile
import wave
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

TTS_VERSION = 'qwen-customvoice-0.6b-eric-sichuan-v2'


def split_narration(text: str, max_chars: int = 150) -> list[str]:
    """Bound native model contexts without deleting sentence punctuation."""
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
            raise ValueError('expected 24kHz mono PCM16 Qwen audio')
        frames = audio.getnframes()
        if not frames:
            raise ValueError('empty Qwen audio')
        pcm = audio.readframes(frames)
    samples = [sample[0] for sample in struct.iter_unpack('<h', pcm)]
    rms = math.sqrt(sum(value * value for value in samples) / len(samples))
    if rms < 10:
        raise ValueError('silent Qwen audio')
    if sum(abs(value) >= 32760 for value in samples) / len(samples) > .02:
        raise ValueError('clipped Qwen audio')
    return frames / 24000


@dataclass(frozen=True)
class GeneratedAudio:
    path: Path
    duration_seconds: float
    sha256: str


def discover_model() -> Path:
    configured = os.getenv('FI_QWEN_MODEL_PATH')
    if configured:
        return Path(configured).expanduser().resolve()
    base = Path.home() / '.cache/huggingface/hub/models--Qwen--Qwen3-TTS-12Hz-0.6B-CustomVoice'
    reference = base / 'refs/main'
    if reference.exists():
        return base / 'snapshots' / reference.read_text().strip()
    snapshots = sorted((base / 'snapshots').glob('*'))
    if len(snapshots) == 1:
        return snapshots[0]
    raise ValueError('set FI_QWEN_MODEL_PATH to a local Qwen CustomVoice snapshot')


class LocalQwenTts:
    def __init__(self, *, model_path: Path, runner: Path, cache_dir: Path,
                 converter: str | None = None, run: Callable = subprocess.run,
                 timeout: int = 1200, device: str = 'metal', dtype: str = 'bf16'):
        self.model_path = model_path.expanduser().resolve()
        self.runner = runner.expanduser()
        self.cache_dir = cache_dir
        self.converter = converter or shutil.which('ffmpeg') or shutil.which('afconvert')
        self.run = run
        self.timeout = timeout
        self.device = device
        self.dtype = dtype

    def synthesize(self, text: str) -> GeneratedAudio:
        if not text.strip() or len(text) > 6000:
            raise ValueError('narration must contain 1..6000 characters')
        if not (self.model_path / 'config.json').is_file():
            raise ValueError('local Qwen model config missing')
        if not self.converter:
            raise ValueError('install ffmpeg (macOS can use afconvert)')
        identity = json.dumps([TTS_VERSION, str(self.model_path), self.device, self.dtype, text], ensure_ascii=False)
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
            request = directory / 'request.json'
            wav = directory / 'voice.wav'
            encoded = directory / 'voice.m4a'
            request.write_text(json.dumps({'parts': split_narration(text), 'voice': 'eric',
                                          'language': 'sichuan_dialect', 'device': self.device,
                                          'dtype': self.dtype}, ensure_ascii=False))
            self.run([str(self.runner), str(self.model_path), str(request), str(wav)],
                     check=True, timeout=self.timeout, capture_output=True)
            duration = validate_wav(wav)
            if Path(self.converter).name == 'afconvert':
                args = [self.converter, '-f', 'm4af', '-d', 'aac', '-b', '64000', str(wav), str(encoded)]
            else:
                args = [self.converter, '-y', '-loglevel', 'error', '-i', str(wav), '-c:a', 'aac',
                        '-b:a', '64k', '-movflags', '+faststart', str(encoded)]
            self.run(args, check=True, timeout=120, capture_output=True)
            if not encoded.exists() or not encoded.stat().st_size:
                raise ValueError('AAC encoding produced an empty file')
            digest = hashlib.sha256(encoded.read_bytes()).hexdigest()
            encoded.replace(destination)
            temporary_metadata = directory / 'metadata.json'
            temporary_metadata.write_text(json.dumps({'sha256': digest, 'duration_seconds': duration,
                                                      'model': str(self.model_path), 'tts_version': TTS_VERSION}))
            temporary_metadata.replace(metadata)
        return GeneratedAudio(destination, duration, digest)
