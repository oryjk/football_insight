import { useEffect, useRef, useState } from "react";
import { RefreshCw } from "lucide-react";
import type { AdminApi, SupportBaseMatch } from "../../api";
import { errorMessage } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import {
  canSave,
  displayedCount,
  matchTitle,
  parseBase,
} from "./supportBasesModel";

function MatchRow({
  match,
  busy,
  onSave,
}: {
  match: SupportBaseMatch;
  busy: boolean;
  onSave: (match: SupportBaseMatch, home: number, away: number) => Promise<void>;
}) {
  const [home, setHome] = useState(String(match.home_team.base));
  const [away, setAway] = useState(String(match.away_team.base));
  const [error, setError] = useState("");
  // The server response is the source of truth after a save.
  useEffect(() => {
    setHome(String(match.home_team.base));
    setAway(String(match.away_team.base));
  }, [match.home_team.base, match.away_team.base]);
  const homeShown = displayedCount(match.home_team.real_count, home);
  const awayShown = displayedCount(match.away_team.real_count, away);
  async function save() {
    const homeBase = parseBase(home);
    const awayBase = parseBase(away);
    if (homeBase === null || awayBase === null) return;
    setError("");
    try {
      await onSave(match, homeBase, awayBase);
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }
  return (
    <li className="support-base-row">
      <p className="support-base-title">{matchTitle(match)}</p>
      <div className="support-base-teams">
        {(
          [
            [match.home_team, home, setHome, homeShown],
            [match.away_team, away, setAway, awayShown],
          ] as const
        ).map(([team, value, setValue, shown]) => (
          <div className="support-base-team" key={team.team_id}>
            <strong>{team.team_name}</strong>
            <span className="field-hint">
              真实 {team.real_count} + 基础值
            </span>
            <Input
              inputMode="numeric"
              aria-label={`${team.team_name}基础值`}
              value={value}
              disabled={busy}
              aria-invalid={parseBase(value) === null}
              onChange={(event) => setValue(event.target.value)}
            />
            <span className="support-base-shown">
              显示 {shown === null ? "—" : shown}
            </span>
          </div>
        ))}
        <Button
          disabled={busy || !canSave(match, home, away)}
          onClick={save}
        >
          保存
        </Button>
      </div>
      {error && (
        <p role="alert" className="operation-error">
          {error}
        </p>
      )}
    </li>
  );
}

export function SupportBasesPage({
  api,
  token,
  onBusyChange,
}: {
  api: AdminApi;
  token: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const [matches, setMatches] = useState<SupportBaseMatch[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [revision, setRevision] = useState(0);
  const savingLock = useRef(false);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError("");
    api
      .supportBaseMatches(token, controller.signal)
      .then((value) => {
        if (!controller.signal.aborted) setMatches(value.items);
      })
      .catch((reason) => {
        if (!controller.signal.aborted) setError(errorMessage(reason));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [api, token, revision]);
  // Leaving the page releases the navigation lock.
  useEffect(() => () => onBusyChange(false), [onBusyChange]);

  async function save(match: SupportBaseMatch, home: number, away: number) {
    if (savingLock.current) return;
    savingLock.current = true;
    setSaving(true);
    onBusyChange(true);
    try {
      const updated = await api.setSupportBases(
        token,
        match.match_id,
        home,
        away,
      );
      setMatches((current) =>
        current.map((item) =>
          item.match_id === updated.match_id ? updated : item,
        ),
      );
    } finally {
      savingLock.current = false;
      setSaving(false);
      onBusyChange(false);
    }
  }

  return (
    <div className="support-bases">
      <div className="topic-readers-toolbar">
        <p className="field-hint">
          基础值会加到真实助力数上显示给用户，只影响这场比赛的助力数和占比，不影响赛季排名。显示数 = 真实投票 + 基础值。
        </p>
        <Button
          variant="secondary"
          disabled={loading || saving}
          aria-label="刷新"
          onClick={() => setRevision((value) => value + 1)}
        >
          <RefreshCw />
          刷新
        </Button>
      </div>
      {error && (
        <p role="alert" className="operation-error">
          {error}
        </p>
      )}
      {loading && (
        <p role="status" className="empty-state">
          正在加载…
        </p>
      )}
      {!loading && !error && !matches.length && (
        <p className="empty-state">近两周和未来没有比赛。</p>
      )}
      <ul className="support-base-list">
        {matches.map((match) => (
          <MatchRow key={match.match_id} match={match} busy={saving} onSave={save} />
        ))}
      </ul>
    </div>
  );
}
