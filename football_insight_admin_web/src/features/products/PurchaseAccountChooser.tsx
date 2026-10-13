import { useCallback, useEffect, useRef, useState } from "react";
import { Check, ChevronLeft, ChevronRight, Search, X } from "lucide-react";
import type { AdminApi, CustomerAccount, CustomerAccountPage } from "../../api";
import { ApiError, errorMessage } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Textarea } from "../../components/ui/textarea";
import { Badge } from "../../components/ui/badge";
import type { ProductForm } from "./productsModel";
import {
  addAccountSelection,
  removeAccountSelection,
  selectedAccountIds,
} from "./purchaseScopeModel";
import { lookupAccountMetadata } from "./accountLookupModel";
export function PurchaseAccountChooser({
  api,
  token,
  value,
  onChange,
  disabled,
  validationError,
}: {
  api: AdminApi;
  token: string;
  value: ProductForm;
  onChange: (value: ProductForm) => void;
  disabled: boolean;
  validationError?: string;
}) {
  const [draftQuery, setDraftQuery] = useState("");
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(1);
  const [revision, setRevision] = useState(0);
  const [result, setResult] = useState<CustomerAccountPage | null>(null);
  const [loading, setLoading] = useState(true);
  const [searchError, setSearchError] = useState("");
  const [lookupIssue, setLookupIssue] = useState("");
  const [selectionError, setSelectionError] = useState("");
  const [known, setKnown] = useState<Record<string, CustomerAccount | null>>(
    {},
  );
  const cache = useRef(new Map<string, CustomerAccount | null>());
  const ids = selectedAccountIds(value.allowedUsers);
  const selectedKey = ids.join(",");
  const selected = new Set(ids);
  const remember = useCallback(
    (entries: Array<[string, CustomerAccount | null]>) => {
      if (!entries.length) return;
      for (const [id, account] of entries) cache.current.set(id, account);
      setKnown((current) => ({ ...current, ...Object.fromEntries(entries) }));
    },
    [],
  );
  useEffect(() => {
    if (disabled) return;
    const controller = new AbortController();
    setLoading(true);
    setSearchError("");
    setResult(null);
    api
      .customerAccounts(token, query, page, controller.signal)
      .then((data) => {
        if (!controller.signal.aborted) {
          setResult(data);
          remember(
            data.items.map((account) => [account.id.toLowerCase(), account]),
          );
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) setSearchError(errorMessage(error));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [api, token, query, page, revision, disabled, remember]);
  useEffect(() => {
    if (disabled || !selectedKey) return;
    const controller = new AbortController();
    setLookupIssue("");
    const missing = selectedKey
      .split(",")
      .filter((id) => !cache.current.has(id))
      .slice(0, 100);
    lookupAccountMetadata(
      missing,
      (id, signal) => api.customerAccount(token, id, signal),
      controller.signal,
    ).then((values) => {
      if (controller.signal.aborted) return;
      const entries: Array<[string, CustomerAccount | null]> = [];
      let failed = false;
      for (const item of values) {
        if (item.value) entries.push([item.id, item.value]);
        else if (item.error instanceof ApiError && item.error.status === 404)
          entries.push([item.id, null]);
        else failed = true;
      }
      remember(entries);
      if (failed)
        setLookupIssue(
          "部分账号暂时无法查询，已有 UUID 已保留，可继续保存或稍后重试。",
        );
    });
    return () => controller.abort();
  }, [api, token, selectedKey, revision, disabled, remember]);
  function search() {
    if (disabled) return;
    setQuery(draftQuery.trim());
    setPage(1);
    setRevision((current) => current + 1);
  }
  function toggle(id: string) {
    if (disabled) return;
    try {
      onChange(
        selected.has(id.toLowerCase())
          ? removeAccountSelection(value, id)
          : addAccountSelection(value, id),
      );
      setSelectionError("");
    } catch (error) {
      setSelectionError(errorMessage(error));
    }
  }
  const title = (account: CustomerAccount) =>
    account.display_name || account.account_identifier || account.id;
  return (
    <div className="purchase-account-chooser">
      <div className="form-field">
        <Label htmlFor="purchase-account-query">搜索购买账号</Label>
        <div className="account-search-actions">
          <Input
            id="purchase-account-query"
            placeholder="按昵称或账号标识搜索"
            value={draftQuery}
            disabled={disabled}
            onChange={(event) => setDraftQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                search();
              }
            }}
          />
          <Button
            type="button"
            variant="secondary"
            onClick={search}
            disabled={disabled}
          >
            <Search />
            搜索账号
          </Button>
        </div>
      </div>
      <div className="account-selected">
        <p className="field-hint">
          已指定 {ids.length} / 100 个账号；查询不到的 UUID 会保留。
        </p>
        <div className="account-selected-chips">
          {ids.map((id) => {
            const account = known[id];
            const label = account ? title(account) : id;
            return (
              <Badge key={id} className="account-selected-chip">
                <span className="account-chip-label">
                  <strong>{label}</strong>
                  <small>
                    {account
                      ? account.account_identifier
                      : known[id] === null
                        ? "账号未查询到 · UUID 已保留"
                        : "UUID 已保留"}
                  </small>
                </span>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  disabled={disabled}
                  aria-label={`移除账号 ${label}`}
                  onClick={() => toggle(id)}
                >
                  <X />
                </Button>
              </Badge>
            );
          })}
        </div>
      </div>
      {lookupIssue && (
        <p className="field-hint" role="status">
          {lookupIssue}
        </p>
      )}
      {selectionError && (
        <p className="field-error" role="alert">
          {selectionError}
        </p>
      )}
      {searchError ? (
        <div className="operation-error" role="alert">
          <span>{searchError}。可重试搜索或手工填写 UUID。</span>
          <Button
            type="button"
            variant="secondary"
            disabled={disabled}
            onClick={() => setRevision((current) => current + 1)}
          >
            重试账号查询
          </Button>
        </div>
      ) : loading ? (
        <p className="account-search-status" role="status">
          正在搜索账号…
        </p>
      ) : (
        result && (
          <>
            <ul
              className="account-search-results"
              aria-label="购买账号搜索结果"
            >
              {result.items.map((account) => (
                <li key={account.id}>
                  <div className="account-result-label">
                    <strong>{title(account)}</strong>
                    <span>
                      {account.account_identifier} · {account.membership_tier} ·{" "}
                      {account.status === "active"
                        ? "正常"
                        : account.status === "disabled"
                          ? "已停用"
                          : account.status}
                    </span>
                    <small className="technical-text">{account.id}</small>
                  </div>
                  <Button
                    type="button"
                    variant={
                      selected.has(account.id.toLowerCase())
                        ? "ghost"
                        : "secondary"
                    }
                    aria-pressed={selected.has(account.id.toLowerCase())}
                    aria-label={`${selected.has(account.id.toLowerCase()) ? "取消选择" : "选择"}账号 ${title(account)}`}
                    disabled={
                      disabled ||
                      (!selected.has(account.id.toLowerCase()) &&
                        ids.length >= 100)
                    }
                    onClick={() => toggle(account.id)}
                  >
                    {selected.has(account.id.toLowerCase()) && <Check />}
                    {selected.has(account.id.toLowerCase()) ? "已选择" : "选择"}
                  </Button>
                </li>
              ))}
            </ul>
            {!result.items.length && (
              <p className="account-search-status">
                没有符合条件的账号，请调整关键词或手工填写 UUID。
              </p>
            )}
            <div className="account-search-pagination">
              <span className="field-hint">
                共 {result.total} 个账号 · 第 {result.page} 页
              </span>
              <div className="pagination-actions">
                <Button
                  type="button"
                  variant="secondary"
                  size="icon"
                  aria-label="账号上一页"
                  disabled={disabled || page <= 1}
                  onClick={() => setPage((current) => current - 1)}
                >
                  <ChevronLeft />
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  size="icon"
                  aria-label="账号下一页"
                  disabled={disabled || page * result.page_size >= result.total}
                  onClick={() => setPage((current) => current + 1)}
                >
                  <ChevronRight />
                </Button>
              </div>
            </div>
          </>
        )
      )}
      <div className="form-field">
        <Label htmlFor="allowed-users">手工填写账号 UUID（可选）</Label>
        <Textarea
          id="allowed-users"
          rows={3}
          value={value.allowedUsers}
          disabled={disabled}
          onChange={(event) =>
            onChange({ ...value, allowedUsers: event.target.value })
          }
          aria-invalid={!!validationError}
          aria-describedby={validationError ? "allowedUsers-error" : undefined}
        />
        <p className="field-hint">
          每行一个
          UUID，可用于未出现在搜索结果中的账号。搜索、选择和手工输入共享同一份购买名单。
        </p>
        {validationError && (
          <p className="field-error" role="alert" id="allowedUsers-error">
            {validationError}
          </p>
        )}
      </div>
    </div>
  );
}
