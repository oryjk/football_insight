import { useEffect, useRef, useState } from "react";
import { RefreshCw, Search } from "lucide-react";
import type { AdminApi, CustomerAccount, ReviewExemptUser } from "../../api";
import { errorMessage } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { PurchaseAvatar } from "../products/ProductUserAvatar";
import { firstReadTime } from "../topics/topicReadersModel";
import {
  exemptName,
  isAlreadyExempt,
  withAdded,
  withRemoved,
} from "./reviewExemptModel";

export function ReviewExemptPage({
  api,
  token,
  onBusyChange,
}: {
  api: AdminApi;
  token: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const [items, setItems] = useState<ReviewExemptUser[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [revision, setRevision] = useState(0);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<CustomerAccount[] | null>(null);
  const [searching, setSearching] = useState(false);
  const [searchError, setSearchError] = useState("");
  const [saving, setSaving] = useState(false);
  const lock = useRef(false);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError("");
    api
      .reviewExemptUsers(token, controller.signal)
      .then((value) => {
        if (!controller.signal.aborted) setItems(value.items);
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

  async function search() {
    const text = query.trim();
    if (!text) return;
    setSearching(true);
    setSearchError("");
    try {
      const page = await api.customerAccounts(token, text, 1);
      setResults(page.items);
    } catch (reason) {
      setSearchError(errorMessage(reason));
    } finally {
      setSearching(false);
    }
  }

  // One write at a time; navigation stays locked until the server answers.
  async function change(action: () => Promise<void>) {
    if (lock.current) return;
    lock.current = true;
    setSaving(true);
    onBusyChange(true);
    setError("");
    try {
      await action();
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      lock.current = false;
      setSaving(false);
      onBusyChange(false);
    }
  }

  const add = (account: CustomerAccount) =>
    change(async () => {
      const added = await api.addReviewExemptUser(token, account.id);
      setItems((current) => withAdded(current, added));
    });
  const remove = (user: ReviewExemptUser) =>
    change(async () => {
      await api.removeReviewExemptUser(token, user.user_id);
      setItems((current) => withRemoved(current, user.user_id));
    });

  return (
    <div className="review-exempt">
      <p className="field-hint">
        版本处于审核中时，小程序会隐藏会员购买等付费功能。名单内的账号登录后不受此限制，可以用来在审核版本上测试完整功能。审核人员的账号请不要加入名单。
      </p>
      {error && (
        <p role="alert" className="operation-error">
          {error}
        </p>
      )}

      <section className="review-exempt-section">
        <div className="topic-readers-toolbar">
          <h2 className="review-exempt-heading">已豁免账号（{items.length}）</h2>
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
        {loading && (
          <p role="status" className="empty-state">
            正在加载…
          </p>
        )}
        {!loading && !error && !items.length && (
          <p className="empty-state">还没有豁免账号。</p>
        )}
        <ul className="review-exempt-list">
          {items.map((user) => (
            <li className="review-exempt-row" key={user.user_id}>
              <span className="topic-reader-user">
                <PurchaseAvatar url={user.avatar_url} />
                <span>
                  <strong>{exemptName(user)}</strong>
                  <span className="field-hint">
                    {user.account_identifier} · 加入于{" "}
                    {firstReadTime(user.created_at)}
                  </span>
                </span>
              </span>
              <Button
                variant="secondary"
                disabled={saving}
                onClick={() => void remove(user)}
              >
                移除
              </Button>
            </li>
          ))}
        </ul>
      </section>

      <section className="review-exempt-section">
        <h2 className="review-exempt-heading">添加账号</h2>
        <form
          className="topic-readers-toolbar"
          onSubmit={(event) => {
            event.preventDefault();
            void search();
          }}
        >
          <Input
            aria-label="搜索账号"
            placeholder="输入昵称或账号标识"
            value={query}
            disabled={saving}
            onChange={(event) => setQuery(event.target.value)}
          />
          <Button type="submit" disabled={searching || saving || !query.trim()}>
            <Search />
            搜索
          </Button>
        </form>
        {searchError && (
          <p role="alert" className="operation-error">
            {searchError}
          </p>
        )}
        {results && !results.length && (
          <p className="empty-state">没有找到匹配的账号。</p>
        )}
        <ul className="review-exempt-list">
          {(results ?? []).map((account) => {
            const added = isAlreadyExempt(account, items);
            return (
              <li className="review-exempt-row" key={account.id}>
                <span className="topic-reader-user">
                  <PurchaseAvatar url={account.avatar_url} />
                  <span>
                    <strong>{exemptName(account)}</strong>
                    <span className="field-hint">
                      {account.account_identifier}
                    </span>
                  </span>
                </span>
                <Button
                  disabled={saving || added}
                  onClick={() => void add(account)}
                >
                  {added ? "已加入" : "加入"}
                </Button>
              </li>
            );
          })}
        </ul>
      </section>
    </div>
  );
}
