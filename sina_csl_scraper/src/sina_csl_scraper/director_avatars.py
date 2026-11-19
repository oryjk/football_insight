"""Sync real player avatars for f_i_director_candidates into MinIO.

Three-tier discovery strategy (prefer no avatar over wrong/fake images):
1. Reuse existing MinIO avatar_storage_url from f_i_squad_players by player_name.
2. Query TheSportsDB public searchplayers API (strSport == 'Soccer', prefer strCutout over strThumb).
3. Fallback to Wikipedia REST Summary API (verify football keywords in description/extract).
"""
from __future__ import annotations

import logging
import re
import time
import unicodedata
from dataclasses import dataclass
from typing import Callable, Protocol
from urllib.parse import quote

import requests

from .assets import AssetFetcher, AssetTarget
from .avatar_thumbs import upload_thumbnails

logger = logging.getLogger(__name__)

THESPORTSDB_SEARCH_URL = "https://www.thesportsdb.com/api/v1/json/3/searchplayers.php"
WIKIPEDIA_SUMMARY_URL = "https://en.wikipedia.org/api/rest_v1/page/summary/{title}"
DEFAULT_USER_AGENT = "FootballInsightBot/1.0 (https://match.oryjk.cn; director-avatar-sync)"

ALLOWED_IMAGE_CONTENT_TYPES: dict[str, str] = {
    "image/jpeg": ".jpg",
    "image/png": ".png",
    "image/webp": ".webp",
}

FOOTBALL_KEYWORDS: tuple[str, ...] = (
    "football",
    "soccer",
    "midfielder",
    "forward",
    "defender",
    "goalkeeper",
)


@dataclass(frozen=True)
class CandidateSearchHint:
    search_names: tuple[str, ...]
    wiki_titles: tuple[str, ...] = ()


@dataclass(frozen=True)
class DirectorCandidate:
    code: str
    player_name: str
    country: str
    league_tag: str
    current_club: str
    avatar_url: str | None = None


# Exact English / romanized search names and Wikipedia titles for all 58 seed candidates
# in football_insight_service_backend_rs/migrations/20261119130000_seed_director_candidates.sql
DIRECTOR_SEARCH_HINTS: dict[str, CandidateSearchHint] = {
    # K League 1 (21)
    "k1-cesinha": CandidateSearchHint(
        search_names=("Cesinha", "César Fernando Silva dos Santos"),
        wiki_titles=("Cesinha (footballer, born 1989)",),
    ),
    "k1-oberdan": CandidateSearchHint(
        search_names=("Oberdan", "Oberdan Alionço de Lima"),
        wiki_titles=("Oberdan (footballer, born 1995)",),
    ),
    "k1-anderson-oliveira": CandidateSearchHint(
        search_names=("Anderson Oliveira", "Anderson de Oliveira da Silva"),
        wiki_titles=("Anderson Oliveira (footballer, born 1998)",),
    ),
    "k1-stefan-mugosa": CandidateSearchHint(
        search_names=("Stefan Mugoša", "Stefan Mugosa"),
        wiki_titles=("Stefan Mugoša",),
    ),
    "k1-stanislav-iljutcenko": CandidateSearchHint(
        search_names=("Stanislav Iljutcenko",),
        wiki_titles=("Stanislav Iljutcenko",),
    ),
    "k1-jasir-asani": CandidateSearchHint(
        search_names=("Jasir Asani",),
        wiki_titles=("Jasir Asani",),
    ),
    "k1-yago-cariello": CandidateSearchHint(
        search_names=("Yago Cariello",),
        wiki_titles=("Yago Cariello",),
    ),
    "k1-darijan-bojanic": CandidateSearchHint(
        search_names=("Darijan Bojanić", "Darijan Bojanic"),
        wiki_titles=("Darijan Bojanić",),
    ),
    "k1-gustav-ludwigson": CandidateSearchHint(
        search_names=("Gustav Ludwigson",),
        wiki_titles=("Gustav Ludwigson",),
    ),
    "k1-jesse-lingard": CandidateSearchHint(
        search_names=("Jesse Lingard",),
        wiki_titles=("Jesse Lingard",),
    ),
    "k1-yazan-al-arab": CandidateSearchHint(
        search_names=("Yazan Al-Arab",),
        wiki_titles=("Yazan Al-Arab",),
    ),
    "k1-hernandes": CandidateSearchHint(
        search_names=("Hernandes Rodrigues", "Hernandes"),
        wiki_titles=("Hernandes (footballer, born 1999)",),
    ),
    "k1-tiago-orobo": CandidateSearchHint(
        search_names=("Tiago Orobó", "Tiago Orobo"),
        wiki_titles=("Tiago Orobó",),
    ),
    "k1-wanderson": CandidateSearchHint(
        search_names=("Wanderson Carvalho", "Wanderson"),
        wiki_titles=("Wanderson (footballer, born 1989)",),
    ),
    "k1-gerso-fernandes": CandidateSearchHint(
        search_names=("Gerso Fernandes",),
        wiki_titles=("Gerso Fernandes",),
    ),
    "k1-edgar-bruno": CandidateSearchHint(
        search_names=("Edgar Bruno da Silva", "Edgar"),
        wiki_titles=("Edgar Bruno da Silva",),
    ),
    "k1-joo-min-kyu": CandidateSearchHint(
        search_names=("Joo Min-kyu",),
        wiki_titles=("Joo Min-kyu",),
    ),
    "k1-lee-dong-gyeong": CandidateSearchHint(
        search_names=("Lee Dong-gyeong",),
        wiki_titles=("Lee Dong-gyeong",),
    ),
    "k1-song-min-kyu": CandidateSearchHint(
        search_names=("Song Min-kyu",),
        wiki_titles=("Song Min-kyu",),
    ),
    "k1-um-won-sang": CandidateSearchHint(
        search_names=("Um Won-sang",),
        wiki_titles=("Um Won-sang",),
    ),
    "k1-kim-young-gwon": CandidateSearchHint(
        search_names=("Kim Young-gwon",),
        wiki_titles=("Kim Young-gwon",),
    ),
    # K League 2 (5)
    "k2-bruno-mota": CandidateSearchHint(
        search_names=("Bruno Mota",),
        wiki_titles=("Bruno Mota (footballer, born 1995)",),
    ),
    "k2-valdivia": CandidateSearchHint(
        search_names=("Valdívia", "Valdivia", "Wanderson Ferreira de Oliveira"),
        wiki_titles=("Valdívia (footballer, born 1994)",),
    ),
    "k2-luis-mina": CandidateSearchHint(
        search_names=("Luis Mina", "Luis Fabián Mina"),
        wiki_titles=("Luis Mina",),
    ),
    "k2-matheus-oliveira": CandidateSearchHint(
        search_names=("Matheus Oliveira", "Matheus Oliveira Santos"),
        wiki_titles=("Matheus Oliveira (footballer, born 1997)",),
    ),
    "k2-rodrigo-bassani": CandidateSearchHint(
        search_names=("Rodrigo Bassani",),
        wiki_titles=("Rodrigo Bassani",),
    ),
    # J1 League (6)
    "j1-anderson-lopes": CandidateSearchHint(
        search_names=("Anderson Lopes",),
        wiki_titles=("Anderson Lopes",),
    ),
    "j1-leo-ceara": CandidateSearchHint(
        search_names=("Léo Ceará", "Leo Ceara"),
        wiki_titles=("Léo Ceará",),
    ),
    "j1-matheus-savio": CandidateSearchHint(
        search_names=("Matheus Sávio", "Matheus Savio"),
        wiki_titles=("Matheus Sávio",),
    ),
    "j1-marcinho": CandidateSearchHint(
        search_names=("Marcinho", "Márcio Augusto da Silva Barbosa"),
        wiki_titles=("Marcinho (footballer, born May 1995)",),
    ),
    "j1-rafael-elias": CandidateSearchHint(
        search_names=("Rafael Elias", "Papagaio"),
        wiki_titles=("Rafael Elias",),
    ),
    "j1-marius-hoibraten": CandidateSearchHint(
        search_names=("Marius Høibråten", "Marius Hoibraten"),
        wiki_titles=("Marius Høibråten",),
    ),
    # Brazil (5)
    "bra-erick-pulga": CandidateSearchHint(
        search_names=("Erick Pulga",),
        wiki_titles=("Erick Pulga",),
    ),
    "bra-anselmo-ramon": CandidateSearchHint(
        search_names=("Anselmo Ramon",),
        wiki_titles=("Anselmo Ramon",),
    ),
    "bra-guilherme-augusto": CandidateSearchHint(
        search_names=("Guilherme Augusto", "Guilherme"),
        wiki_titles=("Guilherme (footballer, born April 1995)",),
    ),
    "bra-matheus-peixoto": CandidateSearchHint(
        search_names=("Matheus Peixoto",),
        wiki_titles=("Matheus Peixoto",),
    ),
    "bra-lucas-halter": CandidateSearchHint(
        search_names=("Lucas Halter",),
        wiki_titles=("Lucas Halter",),
    ),
    # Europe (4)
    "eur-aleksandar-pesic": CandidateSearchHint(
        search_names=("Aleksandar Pešić", "Aleksandar Pesic"),
        wiki_titles=("Aleksandar Pešić",),
    ),
    "eur-mirko-ivanic": CandidateSearchHint(
        search_names=("Mirko Ivanić", "Mirko Ivanic"),
        wiki_titles=("Mirko Ivanić",),
    ),
    "eur-josip-elez": CandidateSearchHint(
        search_names=("Josip Elez",),
        wiki_titles=("Josip Elez",),
    ),
    "eur-nikola-cumic": CandidateSearchHint(
        search_names=("Nikola Čumić", "Nikola Cumic"),
        wiki_titles=("Nikola Čumić",),
    ),
    # Free-agent foreign targets (3)
    "free-jurgen-locadia": CandidateSearchHint(
        search_names=("Jürgen Locadia", "Jurgen Locadia"),
        wiki_titles=("Jürgen Locadia",),
    ),
    "free-christian-bassogog": CandidateSearchHint(
        search_names=("Christian Bassogog",),
        wiki_titles=("Christian Bassogog",),
    ),
    "free-gabriel-pires": CandidateSearchHint(
        search_names=("Gabriel Pires",),
        wiki_titles=("Gabriel Pires",),
    ),
    # Domestic targets (14)
    "csl-han-jiaqi": CandidateSearchHint(
        search_names=("Han Jiaqi",),
        wiki_titles=("Han Jiaqi",),
    ),
    "csl-liu-dianzuo": CandidateSearchHint(
        search_names=("Liu Dianzuo",),
        wiki_titles=("Liu Dianzuo",),
    ),
    "csl-wu-shaocong": CandidateSearchHint(
        search_names=("Wu Shaocong",),
        wiki_titles=("Wu Shaocong",),
    ),
    "csl-gao-zhunyi": CandidateSearchHint(
        search_names=("Gao Zhunyi",),
        wiki_titles=("Gao Zhunyi",),
    ),
    "csl-yang-shuai": CandidateSearchHint(
        search_names=("Yang Shuai",),
        wiki_titles=("Yang Shuai",),
    ),
    "csl-li-shuai": CandidateSearchHint(
        search_names=("Li Shuai",),
        wiki_titles=("Li Shuai (footballer, born 1995)",),
    ),
    "csl-xie-pengfei": CandidateSearchHint(
        search_names=("Xie Pengfei",),
        wiki_titles=("Xie Pengfei",),
    ),
    "csl-dai-weijun": CandidateSearchHint(
        search_names=("Dai Wai Tsun", "Dai Weijun"),
        wiki_titles=("Dai Wai Tsun",),
    ),
    "csl-liao-lisheng": CandidateSearchHint(
        search_names=("Liao Lisheng",),
        wiki_titles=("Liao Lisheng",),
    ),
    "cl1-xu-bin": CandidateSearchHint(
        search_names=("Xu Bin",),
        wiki_titles=("Xu Bin (footballer)",),
    ),
    "csl-lin-liangming": CandidateSearchHint(
        search_names=("Lin Liangming",),
        wiki_titles=("Lin Liangming",),
    ),
    "csl-xie-wenneng": CandidateSearchHint(
        search_names=("Xie Wenneng",),
        wiki_titles=("Xie Wenneng",),
    ),
    "cl1-xiang-yuwang": CandidateSearchHint(
        search_names=("Xiang Yuwang",),
        wiki_titles=("Xiang Yuwang",),
    ),
    "csl-wang-ziming": CandidateSearchHint(
        search_names=("Wang Ziming",),
        wiki_titles=("Wang Ziming (footballer, born 1996)",),
    ),
}


class DirectorAvatarStore(Protocol):
    def list_candidates(self) -> list[DirectorCandidate]: ...

    def find_squad_avatar(self, player_name: str) -> str | None: ...

    def update_avatar_url(self, code: str, avatar_url: str) -> None: ...


class PostgresDirectorAvatarStore:
    def __init__(self, connection) -> None:
        self.connection = connection

    def list_candidates(self) -> list[DirectorCandidate]:
        rows = self.connection.execute(
            """
            SELECT code, player_name, country, league_tag, current_club, avatar_url
            FROM f_i_director_candidates
            WHERE active = TRUE
            ORDER BY sort_order ASC, id ASC
            """
        ).fetchall()
        return [
            DirectorCandidate(
                code=row[0],
                player_name=row[1],
                country=row[2],
                league_tag=row[3],
                current_club=row[4],
                avatar_url=row[5],
            )
            for row in rows
        ]

    def find_squad_avatar(self, player_name: str) -> str | None:
        row = self.connection.execute(
            """
            SELECT avatar_storage_url
            FROM f_i_squad_players
            WHERE player_name = %s
              AND avatar_storage_url IS NOT NULL
              AND btrim(avatar_storage_url) <> ''
            ORDER BY active DESC, last_season DESC, synced_at DESC
            LIMIT 1
            """,
            (player_name,),
        ).fetchone()
        return row[0] if row else None

    def update_avatar_url(self, code: str, avatar_url: str) -> None:
        self.connection.execute(
            """
            UPDATE f_i_director_candidates
            SET avatar_url = %s, updated_at = NOW()
            WHERE code = %s
            """,
            (avatar_url, code),
        )


def _normalize_name(name: str) -> str:
    decomposed = unicodedata.normalize("NFKD", name)
    ascii_only = "".join(ch for ch in decomposed if not unicodedata.combining(ch))
    cleaned = re.sub(r"[^a-z0-9]+", " ", ascii_only.lower())
    return " ".join(cleaned.split())


def _name_match_score(query_name: str, candidate_name: str, alternate_names: str | None = None) -> int:
    """Return match score: 2 for exact match, 1 for token match, 0 for no match."""
    query_norm = _normalize_name(query_name)
    if not query_norm:
        return 0

    candidates = [candidate_name]
    if alternate_names:
        candidates.extend(re.split(r"[,;/]", alternate_names))

    query_tokens = set(query_norm.split())
    best = 0
    for raw in candidates:
        cand_norm = _normalize_name(raw)
        if not cand_norm:
            continue
        if cand_norm == query_norm:
            return 2
        cand_tokens = set(cand_norm.split())
        if len(query_tokens) >= 2 and (query_tokens <= cand_tokens or cand_tokens <= query_tokens):
            best = max(best, 1)
        elif len(query_tokens) == 1 and query_norm in cand_tokens:
            best = max(best, 1)
    return best


def _fallback_hint(candidate: DirectorCandidate) -> CandidateSearchHint:
    # Derive search name from code slug if not present in DIRECTOR_SEARCH_HINTS
    parts = candidate.code.split("-", 1)
    slug = parts[1] if len(parts) == 2 else candidate.code
    derived = " ".join(word.capitalize() for word in slug.split("-") if word)
    names = tuple(dict.fromkeys(n for n in (derived, candidate.player_name) if n))
    wiki_titles = (derived,) if derived else ()
    return CandidateSearchHint(search_names=names, wiki_titles=wiki_titles)


def _valid_http_image_url(url: object) -> str | None:
    if not isinstance(url, str):
        return None
    trimmed = url.strip()
    if not trimmed.startswith(("http://", "https://")):
        return None
    if trimmed.lower().split("?", 1)[0].endswith(".svg"):
        return None
    return trimmed


def _is_football_summary(payload: dict) -> bool:
    if (payload.get("type") or "").strip().lower() == "disambiguation":
        return False
    text = f"{payload.get('description') or ''} {payload.get('extract') or ''}".lower()
    return any(keyword in text for keyword in FOOTBALL_KEYWORDS)


def normalize_image_content_type(content_type: str) -> str | None:
    base = content_type.split(";", 1)[0].strip().lower()
    if base == "image/jpg":
        base = "image/jpeg"
    return base if base in ALLOWED_IMAGE_CONTENT_TYPES else None


class AvatarDiscoverer:
    """Discover real player portraits from TheSportsDB with Wikipedia REST Summary fallback."""

    def __init__(
        self,
        session: requests.Session | None = None,
        timeout: int = 15,
        hints: dict[str, CandidateSearchHint] | None = None,
    ) -> None:
        self.session = session or requests.Session()
        if session is None:
            self.session.headers["User-Agent"] = DEFAULT_USER_AGENT
        self.timeout = timeout
        self.hints = DIRECTOR_SEARCH_HINTS if hints is None else hints
        self._thesportsdb_blocked = False

    def search_thesportsdb(self, name: str) -> str | None:
        response = self.session.get(
            THESPORTSDB_SEARCH_URL,
            params={"p": name},
            timeout=self.timeout,
        )
        response.raise_for_status()
        payload = response.json()
        players = payload.get("player") or []
        if not isinstance(players, list):
            return None

        best_rank: tuple[int, int] = (0, 0)
        best_url: str | None = None
        for item in players:
            if not isinstance(item, dict):
                continue
            if (item.get("strSport") or "").strip().lower() != "soccer":
                continue
            score = _name_match_score(
                name,
                str(item.get("strPlayer") or ""),
                item.get("strPlayerAlternate"),
            )
            if score == 0:
                continue
            cutout = _valid_http_image_url(item.get("strCutout"))
            thumb = _valid_http_image_url(item.get("strThumb"))
            if cutout:
                rank = (2, score)
                url = cutout
            elif thumb:
                rank = (1, score)
                url = thumb
            else:
                continue
            if rank > best_rank:
                best_rank = rank
                best_url = url
        return best_url

    def search_wikipedia(self, title: str) -> str | None:
        encoded = quote(title.replace(" ", "_"), safe="")
        url = WIKIPEDIA_SUMMARY_URL.format(title=encoded)
        response = self.session.get(
            url,
            headers={"User-Agent": DEFAULT_USER_AGENT},
            timeout=self.timeout,
        )
        if response.status_code == 404:
            return None
        response.raise_for_status()
        payload = response.json()
        if not isinstance(payload, dict) or not _is_football_summary(payload):
            return None

        thumbnail = payload.get("thumbnail")
        if isinstance(thumbnail, dict):
            thumb_url = _valid_http_image_url(thumbnail.get("source"))
            if thumb_url:
                return thumb_url

        original = payload.get("originalimage")
        if isinstance(original, dict):
            original_url = _valid_http_image_url(original.get("source"))
            if original_url:
                return original_url

        return None

    def discover(self, candidate: DirectorCandidate) -> str | None:
        hint = self.hints.get(candidate.code) or _fallback_hint(candidate)

        if not self._thesportsdb_blocked:
            for name in hint.search_names:
                try:
                    found = self.search_thesportsdb(name)
                except requests.HTTPError as exc:
                    status = exc.response.status_code if exc.response is not None else None
                    if status in (401, 403):
                        self._thesportsdb_blocked = True
                        logger.warning(
                            "TheSportsDB returned HTTP %s; disabling TheSportsDB for remaining candidates and using Wikipedia.",
                            status,
                        )
                        break
                    logger.warning("TheSportsDB HTTP %s for %s (%s)", status, candidate.code, name)
                    continue
                except Exception as exc:  # noqa: BLE001 - 单个查询失败继续尝试其他名字或兜底源
                    logger.warning("TheSportsDB lookup failed for %s (%s): %s", candidate.code, name, exc)
                    continue
                if found:
                    return found

        wiki_titles = hint.wiki_titles or hint.search_names[:1]
        for title in wiki_titles:
            try:
                found = self.search_wikipedia(title)
            except Exception as exc:  # noqa: BLE001 - Wikipedia 失败不阻塞
                logger.warning("Wikipedia lookup failed for %s (%s): %s", candidate.code, title, exc)
                continue
            if found:
                return found

        return None


def sync_director_avatars(
    store: DirectorAvatarStore,
    prefix: str,
    fetcher: AssetFetcher,
    target: AssetTarget | None,
    discoverer: AvatarDiscoverer | None = None,
    *,
    force: bool = False,
    dry_run: bool = False,
    limit: int | None = None,
    interval: float = 0.5,
    sleep_fn: Callable[[float], None] = time.sleep,
) -> dict[str, int]:
    """Backfill real portraits for f_i_director_candidates into MinIO (idempotent)."""
    discoverer = discoverer or AvatarDiscoverer()
    summary = {
        "candidates": 0,
        "skipped": 0,
        "reused_from_squad": 0,
        "uploaded": 0,
        "not_found": 0,
        "failed": 0,
    }
    external_requests = 0

    for candidate in store.list_candidates():
        if limit is not None and summary["candidates"] >= limit:
            break
        summary["candidates"] += 1

        if not force and candidate.avatar_url and candidate.avatar_url.strip():
            summary["skipped"] += 1
            continue

        try:
            squad_url = store.find_squad_avatar(candidate.player_name)
            if squad_url and squad_url.strip():
                if not dry_run:
                    store.update_avatar_url(candidate.code, squad_url.strip())
                summary["reused_from_squad"] += 1
                continue

            if interval > 0 and external_requests > 0:
                sleep_fn(interval)
            external_requests += 1

            remote_url = discoverer.discover(candidate)
            if not remote_url:
                summary["not_found"] += 1
                continue

            if dry_run:
                logger.info("[dry-run] would upload %s from %s", candidate.code, remote_url)
                continue

            if target is None:
                raise RuntimeError("MinIO asset target is required when not in dry_run mode")

            content, raw_content_type = fetcher.fetch(remote_url)
            if not content:
                raise ValueError(f"empty image payload from {remote_url}")
            normalized_type = normalize_image_content_type(raw_content_type)
            if normalized_type is None:
                raise ValueError(f"unsupported image content type {raw_content_type!r} from {remote_url}")

            ext = ALLOWED_IMAGE_CONTENT_TYPES[normalized_type]
            object_name = f"{prefix.strip('/')}/players/director-{candidate.code}{ext}"
            storage_url = target.upload_bytes(object_name, content, normalized_type)
            try:
                upload_thumbnails(target, object_name, content)
            except Exception:  # noqa: BLE001 - 小图是增强，失败不影响原图上传
                logger.warning("director avatar thumbnails failed for %s", object_name, exc_info=True)
            store.update_avatar_url(candidate.code, storage_url)
            summary["uploaded"] += 1
        except Exception:  # noqa: BLE001 - 单人失败记录日志并继续
            logger.warning(
                "director avatar sync failed for %s (%s)",
                candidate.code,
                candidate.player_name,
                exc_info=True,
            )
            summary["failed"] += 1

    return summary
