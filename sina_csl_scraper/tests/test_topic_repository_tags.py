"""Opt-in repository integration tests against a disposable database only."""
import copy
import os
from uuid import uuid4
from urllib.parse import urlsplit

import psycopg
from psycopg.conninfo import make_conninfo
from psycopg.types.json import Jsonb
import pytest

from sina_csl_scraper.topics.repository import TopicRepository


@pytest.fixture
def repository():
    url = os.environ.get('TOPICS_TEST_DATABASE_URL')
    if not url:
        pytest.skip('requires explicit disposable PostgreSQL database')
    assert urlsplit(url).path.endswith('_test'), 'refusing non-test database'
    schema = 'news_tags_' + uuid4().hex
    with psycopg.connect(url) as connection:
        connection.execute(f'CREATE SCHEMA {schema}')
        connection.execute(f'CREATE TABLE {schema}.f_i_daily_topic_editions (id TEXT PRIMARY KEY,team_id BIGINT,payload JSONB,published_at TIMESTAMPTZ)')
    repo = TopicRepository(make_conninfo(url, options=f'-c search_path={schema}'))
    yield repo
    with psycopg.connect(url) as connection:
        connection.execute(f'DROP SCHEMA {schema} CASCADE')


def test_snapshot_source_future_clock_cannot_block_deduplication(repository):
    payload = {'items': [{'sources': [{'url':'https://official.example/query','published_at':'2099-01-01T00:00:00+08:00'}]}]}
    with psycopg.connect(repository.database_url) as c:
        c.execute('INSERT INTO f_i_daily_topic_editions VALUES(%s,%s,%s,now())',('snapshot',77680,Jsonb(payload)))
    assert repository.published_sources(77680) == ({'https://official.example/query'},None)


def test_patch_tags_changes_only_tags_and_protects_concurrent_text(repository):
    item = {'id':'article','title':'成都蓉城比赛','summary':'比赛摘要','paragraphs':['成都蓉城对阵天津津门虎。'],
        'published_at':'2026-10-13T08:00:00+08:00','audio':{'status':'ready','url':'https://assets.example/audio.m4a'},'images':[]}
    with psycopg.connect(repository.database_url) as c:
        c.execute('INSERT INTO f_i_daily_topic_editions VALUES(%s,%s,%s,now())',('edition',77680,Jsonb({'items':[item]})))
    update = copy.deepcopy(item)
    update['team_tags'] = [{'team_id':77680,'name':'成都蓉城'},{'team_id':148,'name':'天津津门虎'}]
    assert repository.patch_tags(77680,[update]) == 1
    with psycopg.connect(repository.database_url) as c:
        actual = c.execute('SELECT payload FROM f_i_daily_topic_editions').fetchone()[0]['items'][0]
    assert actual == update
    assert repository.patch_tags(77680,[{**update,'paragraphs':['过期的草稿'],'team_tags':[]}]) == 0
    with psycopg.connect(repository.database_url) as c:
        assert c.execute('SELECT payload FROM f_i_daily_topic_editions').fetchone()[0]['items'][0] == update
