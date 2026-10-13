import json
import math
import struct
import wave
from pathlib import Path

import pytest

from sina_csl_scraper.topics.tts import LocalQwenTts, split_narration, validate_wav


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


def test_local_qwen_receives_eric_sichuan_and_reuses_verified_cache(tmp_path):
    model = tmp_path / 'model'
    model.mkdir()
    (model / 'config.json').write_text('{}')
    calls = []

    def run(args, **kwargs):
        calls.append(args)
        if args[0] == 'qwen-runner':
            request = json.loads(Path(args[2]).read_text())
            assert request['voice'] == 'eric'
            assert request['language'] == 'sichuan_dialect'
            assert request['parts'] == ['蓉城雄起！']
            write_voice(Path(args[3]))
        else:
            Path(args[-1]).write_bytes(b'valid-aac-container')

    tts = LocalQwenTts(model_path=model, runner=Path('qwen-runner'), cache_dir=tmp_path / 'cache', converter='ffmpeg', run=run)
    first = tts.synthesize('蓉城雄起！')
    second = tts.synthesize('蓉城雄起！')
    assert first == second
    assert first.duration_seconds == 1.0
    assert first.path.suffix == '.m4a'
    assert len(calls) == 2
    first.path.write_bytes(b'corrupted')
    tts.synthesize('蓉城雄起！')
    assert len(calls) == 4


def test_tts_rejects_unbounded_or_blank_input_before_starting_process(tmp_path):
    tts = LocalQwenTts(model_path=tmp_path, runner=Path('missing'), cache_dir=tmp_path)
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
