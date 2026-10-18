import { useEffect, useRef, useState } from "react";
import type { AdminApi } from "../../api";
import { errorMessage } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import {
  DEFAULT_BRAND_SUBTITLE,
  MAX_SUBTITLE_CHARS,
  canSaveSubtitle,
  validateSubtitle,
} from "./brandModel";

export function BrandPage({
  api,
  token,
  onBusyChange,
}: {
  api: AdminApi;
  token: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const [saved, setSaved] = useState("");
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const savingLock = useRef(false);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError("");
    api
      .brandSubtitle(token, controller.signal)
      .then((value) => {
        if (controller.signal.aborted) return;
        const current = value.subtitle ?? "";
        setSaved(current);
        setInput(current);
      })
      .catch((reason) => {
        if (!controller.signal.aborted) setError(errorMessage(reason));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [api, token]);
  // Leaving the page releases the navigation lock.
  useEffect(() => () => onBusyChange(false), [onBusyChange]);

  const invalid = validateSubtitle(input);
  const shown = input.trim() || DEFAULT_BRAND_SUBTITLE;

  async function save() {
    if (savingLock.current || !canSaveSubtitle(input, saved)) return;
    savingLock.current = true;
    setSaving(true);
    onBusyChange(true);
    setError("");
    setNotice("");
    try {
      // The server response is the source of truth after a save.
      const value = await api.setBrandSubtitle(token, input);
      const current = value.subtitle ?? "";
      setSaved(current);
      setInput(current);
      setNotice("已保存，小程序下次拉取公共配置时生效（通常 2 分钟内）。");
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      savingLock.current = false;
      setSaving(false);
      onBusyChange(false);
    }
  }

  return (
    <div className="brand-settings">
      <p className="field-hint">
        小程序顶部品牌栏和启动画面显示的副标题。留空则使用默认文案「
        {DEFAULT_BRAND_SUBTITLE}」。
      </p>
      <div className="brand-settings-field">
        <label htmlFor="brand-subtitle">副标题</label>
        <Input
          id="brand-subtitle"
          value={input}
          placeholder={DEFAULT_BRAND_SUBTITLE}
          disabled={loading || saving}
          aria-invalid={invalid !== null}
          onChange={(event) => {
            setInput(event.target.value);
            setNotice("");
          }}
        />
        <span className="field-hint">
          {[...input.trim()].length}/{MAX_SUBTITLE_CHARS}
          {invalid && ` · ${invalid}`}
        </span>
      </div>
      <p className="brand-settings-preview">预览：足球洞察 · {shown}</p>
      <div>
        <Button
          disabled={loading || saving || !canSaveSubtitle(input, saved)}
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
      {notice && <p role="status">{notice}</p>}
    </div>
  );
}
