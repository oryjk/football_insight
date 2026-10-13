import { useEffect, useRef, useState, type FormEvent } from "react";
import { KeyRound } from "lucide-react";
import type { AdminApi } from "../../api";
import { ApiError, errorMessage, isAborted } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import {
  newSetupForm,
  setupPayload,
  SetupFormError,
  type SetupForm,
} from "./setupModel";
export function AdminSetupForm({
  api,
  onSuccess,
  onAlreadyInitialized,
}: {
  api: AdminApi;
  onSuccess: () => void;
  onAlreadyInitialized: () => void;
}) {
  const [form, setForm] = useState(newSetupForm);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [fieldError, setFieldError] = useState<{
    field: keyof SetupForm;
    message: string;
  } | null>(null);
  const lock = useRef(false);
  const request = useRef<AbortController | null>(null);
  useEffect(() => () => request.current?.abort(), []);
  async function initialize(event: FormEvent) {
    event.preventDefault();
    if (lock.current) return;
    setError("");
    setFieldError(null);
    let payload;
    try {
      payload = setupPayload(form);
    } catch (error) {
      if (error instanceof SetupFormError)
        setFieldError({ field: error.field, message: error.message });
      else setError(errorMessage(error));
      return;
    }
    lock.current = true;
    setBusy(true);
    const controller = new AbortController();
    request.current = controller;
    try {
      await api.initialize(payload, controller.signal);
      if (!controller.signal.aborted) {
        setForm(newSetupForm());
        onSuccess();
      }
    } catch (error) {
      if (!controller.signal.aborted && !isAborted(error)) {
        if (error instanceof ApiError && error.status === 409) {
          setForm(newSetupForm());
          onAlreadyInitialized();
        } else setError(errorMessage(error));
      }
    } finally {
      lock.current = false;
      if (!controller.signal.aborted) setBusy(false);
    }
  }
  const change = (field: keyof SetupForm, value: string) => {
    setForm((current) => ({ ...current, [field]: value }));
    setFieldError(null);
    setError("");
  };
  const invalid = (field: keyof SetupForm) => ({
    "aria-invalid": fieldError?.field === field,
    "aria-describedby":
      fieldError?.field === field ? `setup-${field}-error` : undefined,
  });
  const validation = (field: keyof SetupForm) =>
    fieldError?.field === field ? (
      <p className="field-error" role="alert" id={`setup-${field}-error`}>
        {fieldError.message}
      </p>
    ) : null;
  return (
    <form
      onSubmit={initialize}
      className="login-form"
      aria-label="管理员初始化"
      aria-busy={busy}
    >
      <div>
        <h2>初始化管理员</h2>
        <p className="page-description">首次使用，请为管理员设置登录密码。</p>
      </div>
      <div className="form-field">
        <Label htmlFor="setup-username">管理员账号</Label>
        <Input
          id="setup-username"
          autoComplete="username"
          value="superadmin"
          readOnly
        />
        <p className="field-hint">初始化账号固定，登录后可在设置中修改。</p>
      </div>
      <div className="form-field">
        <Label htmlFor="setup-password">设置密码</Label>
        <Input
          id="setup-password"
          name="password"
          type="password"
          autoComplete="new-password"
          value={form.password}
          disabled={busy}
          onChange={(event) => change("password", event.target.value)}
          {...invalid("password")}
        />
        <p className="field-hint">8–128 个字符，空格也属于密码。</p>
        {validation("password")}
      </div>
      <div className="form-field">
        <Label htmlFor="setup-confirm-password">确认密码</Label>
        <Input
          id="setup-confirm-password"
          name="confirm_password"
          type="password"
          autoComplete="new-password"
          value={form.confirmPassword}
          disabled={busy}
          onChange={(event) => change("confirmPassword", event.target.value)}
          {...invalid("confirmPassword")}
        />
        {validation("confirmPassword")}
      </div>
      {error && (
        <p className="operation-error" role="alert">
          {error}
        </p>
      )}
      <Button type="submit" size="lg" disabled={busy}>
        <KeyRound />
        {busy ? "初始化中…" : "初始化管理员"}
      </Button>
    </form>
  );
}
