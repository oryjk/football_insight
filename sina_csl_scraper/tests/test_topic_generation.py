import json
from datetime import datetime

import pytest
import requests

NOW = datetime.fromisoformat('2026-10-13T12:00:00+08:00')


def sources():
    from sina_csl_scraper.topics.models import SourceArticle
    return [SourceArticle('成都蓉城中超备战', 'https://club.example/a', '俱乐部', NOW, '成都蓉城一线队今日开展中超备战。' * 10), SourceArticle('成都蓉城希拉谷青训', 'https://club.example/b', '俱乐部', NOW, '成都蓉城希拉谷参加中乙联赛。' * 10)]


def topic(source_id, **overrides):
    return {**{'title': '成都蓉城备战中超', 'summary': '一线队开展训练。', 'paragraphs': ['成都蓉城一线队今日开展中超备战。'], 'source_ids': [source_id]}, **overrides}


class Response:
    def __init__(self, content):
        self.content = content
    def raise_for_status(self):
        pass
    def json(self):
        return {'choices': [{'message': {'content': self.content}}]}


class Session:
    def __init__(self, content):
        self.content, self.calls = content, []
    def post(self, url, **kwargs):
        self.calls.append((url, kwargs))
        if isinstance(self.content, Exception):
            raise self.content
        return Response(self.content)


def test_generation_real_provenance_stable_id_and_grounded_prompt():
    from sina_csl_scraper.topics.generation import TopicGenerator
    originals = sources()
    session = Session(json.dumps({'topics': [topic(originals[0].source_id)]}))
    generator = TopicGenerator('https://llm.example/v1', 'news-model', 'secret', session=session)
    result = generator.generate(originals, 77680, NOW)
    assert len(result) == 1
    assert result[0]['sources'] == [{'title': '成都蓉城中超备战', 'url': 'https://club.example/a', 'publisher': '俱乐部', 'published_at': '2026-10-13T12:00:00+08:00'}]
    assert result[0]['audio'] == {'status': 'pending', 'url': None, 'dialect': 'sichuan', 'duration_seconds': None}
    assert result[0]['team_id'] == 77680
    assert result[0]['id'] == generator.generate(originals, 77680, NOW)[0]['id']
    url, kwargs = session.calls[0]
    assert url == 'https://llm.example/v1/chat/completions'
    assert originals[0].body in kwargs['json']['messages'][1]['content']
    assert '希拉谷' in kwargs['json']['messages'][0]['content']


@pytest.mark.parametrize('bad', ['not json', '```json\n{}\n```', '{"topics":[{"title":"x"}]}'])
def test_invalid_json_structure_rejects(bad):
    from sina_csl_scraper.topics.generation import TopicGenerator
    with pytest.raises(ValueError):
        TopicGenerator('https://llm.example', 'm', session=Session(bad)).generate(sources(), 77680, NOW)


@pytest.mark.parametrize('overrides', [{'source_ids': ['unknown']}, {'source_ids': []}, {'source_ids': ['https://fake.example']}, {'paragraphs': []}, {'summary': ''}, {'title': '<script>蓉城</script>'}, {'title': 'x' * 121}, {'title': '其它球队训练', 'summary': '其它球队', 'paragraphs': ['其它球队']}])
def test_invalid_grounding_content_rejects_entire_batch(overrides):
    from sina_csl_scraper.topics.generation import TopicGenerator
    originals = sources()
    content = {'topics': [topic(originals[0].source_id), topic(originals[0].source_id, **overrides)]}
    with pytest.raises(ValueError):
        TopicGenerator('https://llm.example', 'm', session=Session(json.dumps(content))).generate(originals, 77680, NOW)


def test_empty_sources_skip_llm_and_empty_generation_does_not_fill():
    from sina_csl_scraper.topics.generation import TopicGenerator
    session = Session('{"topics":[]}')
    generator = TopicGenerator('https://llm.example/v1/chat/completions', 'm', session=session)
    assert generator.generate([], 77680, NOW) == []
    assert session.calls == []
    assert generator.generate(sources(), 77680, NOW) == []
    assert len(session.calls) == 1


def test_duplicates_collapse_and_above_limit_rejects():
    from sina_csl_scraper.topics.generation import TopicGenerator
    original = sources()[0]
    content = json.dumps({'topics': [topic(original.source_id), topic(original.source_id)]})
    assert len(TopicGenerator('https://llm.example', 'm', session=Session(content)).generate([original], 77680, NOW)) == 1
    content = json.dumps({'topics': [topic(original.source_id, title=f'蓉城报道{i}') for i in range(6)]})
    with pytest.raises(ValueError):
        TopicGenerator('https://llm.example', 'm', session=Session(content)).generate([original], 77680, NOW)


def test_throttling_no_repeated_requests_or_secret_leak():
    from sina_csl_scraper.topics.generation import TopicGenerator
    session = Session(requests.HTTPError('429 secret-key'))
    with pytest.raises(ValueError) as exc:
        TopicGenerator('https://llm.example', 'm', 'secret-key', session=session).generate(sources(), 77680, NOW)
    assert 'secret-key' not in str(exc.value)
    assert len(session.calls) == 1


def test_bound_narration_fits_tts_and_provider_changes_cache_identity():
    from sina_csl_scraper.topics.generation import TopicGenerator
    original = sources()[0]
    content = json.dumps({'topics': [topic(original.source_id, paragraphs=['蓉城' + '训练' * 1500, '训练' * 1500])]})
    generator = TopicGenerator('https://llm.example/v1/', 'm', session=Session(content))
    assert generator.base_url == 'https://llm.example/v1'
    with pytest.raises(ValueError):
        generator.generate([original], 77680, NOW)


def test_youth_team_sources_cannot_be_relabeled_first_team():
    from sina_csl_scraper.topics.generation import TopicGenerator
    original = sources()[1]
    content = json.dumps({'topics': [topic(original.source_id)]})
    with pytest.raises(ValueError):
        TopicGenerator('https://llm.example', 'm', session=Session(content)).generate([original], 77680, NOW)


def test_duplicate_content_with_different_refs_collapses():
    from sina_csl_scraper.topics.generation import TopicGenerator
    original = sources()[0]
    from sina_csl_scraper.topics.models import SourceArticle
    another = SourceArticle(original.title, 'https://news.example/same', original.publisher, NOW, original.body)
    content = json.dumps({'topics': [topic(original.source_id), topic(another.source_id)]})
    assert len(TopicGenerator('https://llm.example', 'm', session=Session(content)).generate([original, another], 77680, NOW)) == 1


def test_glm53_uses_bounded_low_reasoning_json_request():
    from sina_csl_scraper.topics.generation import TopicGenerator
    original = sources()[0]
    session = Session(json.dumps({'topics': [topic(original.source_id)]}))
    TopicGenerator('https://open.bigmodel.cn/api/paas/v4', 'glm-5.3', session=session).generate([original], 77680, NOW)
    url, options = session.calls[0]
    assert url == 'https://open.bigmodel.cn/api/paas/v4/chat/completions'
    assert options['json']['thinking'] == {'type': 'enabled'}
    assert options['json']['reasoning_effort'] == 'low'
    assert options['json']['max_tokens'] == 8192
    assert options['json']['response_format'] == {'type': 'json_object'}


def test_provider_quota_error_is_actionable_without_echoing_response_secrets():
    from sina_csl_scraper.topics.generation import TopicGenerator
    class QuotaResponse:
        status_code = 429
        def raise_for_status(self):
            raise requests.HTTPError('429 secret-key', response=self)
        def json(self):
            return {'error': {'code': '1113', 'message': 'secret-key'}}
    class QuotaSession:
        def post(self, *args, **kwargs):
            return QuotaResponse()
    with pytest.raises(ValueError) as error:
        TopicGenerator('https://open.bigmodel.cn/api/paas/v4', 'glm-5.3', 'secret-key', session=QuotaSession()).generate(sources(), 77680, NOW)
    assert 'HTTP 429' in str(error.value)
    assert '1113' in str(error.value)
    assert '额度不足' in str(error.value)
    assert 'secret-key' not in str(error.value)


def test_k3_256k_uses_code_endpoint_without_glm_specific_sampling_options():
    from sina_csl_scraper.topics.generation import TopicGenerator
    original = sources()[0]
    session = Session(json.dumps({'topics': [topic(original.source_id)]}))
    generator = TopicGenerator('https://api.kimi.com/coding/v1', 'k3-256k', session=session)
    items = generator.generate([original], 77680, NOW)
    url, options = session.calls[0]
    assert url == 'https://api.kimi.com/coding/v1/chat/completions'
    assert options['json']['model'] == 'k3-256k'
    assert options['json']['reasoning_effort'] == 'low'
    assert options['json']['max_tokens'] == 8192
    assert 'temperature' not in options['json']
    assert 'thinking' not in options['json']
    assert options['headers']['User-Agent'] == 'FootballInsight/1.0'
    assert items[0]['sources'][0]['url'] == 'https://club.example/a'
