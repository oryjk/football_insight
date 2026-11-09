import math
import struct
import wave
from pathlib import Path

import pytest

from sina_csl_scraper.topics.tts import EdgeYunyangTts, split_narration, validate_wav


def write_voice(path: Path, silent: bool = False) -> None:
    with wave.open(str(path), 'wb') as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(24000)
        output.writeframes(b''.join(struct.pack('<h', 0 if silent else int(2000 * math.sin(i / 8))) for i in range(24000)))


def test_narration_segments_preserve_every_word_and_respect_bound():
    text = '成都蓉城，雄起！今天关注球队训练。\n\n' + '下场比赛需要认真准备。' * 25
    parts = split_narration(text, max_chars=90)
    assert ''.join(parts) == text.replace('\n', '')
    assert max(map(len, parts)) <= 90


def test_invalid_or_silent_wave_cannot_be_published(tmp_path):
    path = tmp_path / 'voice.wav'
    write_voice(path, silent=True)
    with pytest.raises(ValueError, match='silent'):
        validate_wav(path)
    write_voice(path)
    assert validate_wav(path) == 1.0


def test_tts_rejects_unbounded_or_blank_input_before_starting_process(tmp_path):
    tts = EdgeYunyangTts(cache_dir=tmp_path, converter='ffmpeg')
    for value in ['', ' ' * 20, '蓉' * 6001]:
        with pytest.raises(ValueError):
            tts.synthesize(value)


def test_many_paragraphs_fit_native_segment_capacity_without_losing_words():
    paragraphs = [(('蓉' * 99 + '。') * 2 + '备战' * 36 + '。') for _ in range(20)]
    text = '成都蓉城训练观察。\n' + '\n\n'.join(paragraphs)
    assert len(text.replace('\n', '')) < 5500
    parts = split_narration(text)
    assert ''.join(parts) == text.replace('\n', '')
    assert len(parts) <= 60


def test_edge_yunyang_tts_synthesizes_valid_wav_caches_m4a_and_retries_transient_failure(tmp_path):
    calls = []
    attempts = 0

    def run(args, **kwargs):
        calls.append(args)
        if '-ar' in args and '24000' in args:
            write_voice(Path(args[-1]))
        else:
            Path(args[-1]).write_bytes(b'valid-edge-aac')

    def synthesize_mp3(text: str, voice: str, rate: str, mp3_path: Path, timeout: int) -> None:
        nonlocal attempts
        attempts += 1
        calls.append(['edge_tts', text, voice, rate])
        if attempts == 1:
            raise RuntimeError('transient network error')
        mp3_path.write_bytes(b'fake-mp3-audio')

    tts = EdgeYunyangTts(
        cache_dir=tmp_path / 'cache',
        converter='ffmpeg',
        run=run,
        synthesize_mp3=synthesize_mp3,
        max_retries=3,
        retry_delay=0.0,
    )
    assert tts.voice == 'zh-CN-YunyangNeural'
    first = tts.synthesize('成都蓉城今日备战动态。')
    second = tts.synthesize('成都蓉城今日备战动态。')
    assert first == second
    assert first.duration_seconds == 1.0
    assert first.path.suffix == '.m4a'
    assert attempts == 2
    assert len(calls) == 4  # 2 edge_tts attempts + ffmpeg wav + ffmpeg m4a; second call hits cache
    first.path.write_bytes(b'corrupted')
    tts.synthesize('成都蓉城今日备战动态。')
    assert attempts == 3
