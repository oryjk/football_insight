import { useEffect, useRef, useState, type FormEvent } from "react";
import { Save } from "lucide-react";
import type { AdminApi } from "../../api";
import { errorMessage, isAborted } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Switch } from "../../components/ui/switch";
import { Textarea } from "../../components/ui/textarea";
import {
  qrUrlError,
  StudentConfigError,
  studentConfigPayload,
  type StudentConfig,
  type StudentServiceCode,
  type StudentServiceConfig,
} from "./studentConfigModel";

function ContactQrPreview({ url }: { url: string }) {
  const [failed, setFailed] = useState(false);
  return failed ? (
    <p className="field-hint" role="status">二维码图片加载失败，请检查链接是否可公开访问。</p>
  ) : (
    <img className="student-contact-qr" src={url} alt="联系微信二维码预览" referrerPolicy="no-referrer" onError={() => setFailed(true)} />
  );
}

export function StudentConfigPage({ api, token, onBusyChange }: {
  api: AdminApi;
  token: string;
  onBusyChange: (busy: boolean) => void;
}) {
  // A form becomes editable only after a successful read; no writable defaults.
  const [form, setForm] = useState<StudentConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [revision, setRevision] = useState(0);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [fieldError, setFieldError] = useState<StudentConfigError | null>(null);
  const writeLock = useRef(false);
  const writeRequest = useRef<AbortController | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setForm(null);
    setError("");
    api.studentConfig(token, controller.signal)
      .then((value) => { if (!controller.signal.aborted) setForm(value); })
      .catch((reason) => { if (!controller.signal.aborted && !isAborted(reason)) setError(errorMessage(reason)); })
      .finally(() => { if (!controller.signal.aborted) setLoading(false); });
    return () => controller.abort();
  }, [api, token, revision]);
  useEffect(() => () => {
    writeRequest.current?.abort();
    onBusyChange(false);
  }, [onBusyChange]);

  function clearFeedback() {
    setError("");
    setNotice("");
    setFieldError(null);
  }
  function changeService(code: StudentServiceCode, patch: Partial<Pick<StudentServiceConfig, "description" | "product_id" | "enabled">>) {
    if (writeLock.current) return;
    setForm((current) => current && ({ ...current, services: current.services.map((service) => service.code === code ? { ...service, ...patch } : service) }));
    clearFeedback();
  }
  async function save(event: FormEvent) {
    event.preventDefault();
    if (!form || loading || writeLock.current) return;
    clearFeedback();
    let payload: StudentConfig;
    try {
      payload = studentConfigPayload(form);
    } catch (reason) {
      if (reason instanceof StudentConfigError) setFieldError(reason);
      else setError(errorMessage(reason));
      return;
    }
    writeLock.current = true;
    setSaving(true);
    onBusyChange(true);
    const controller = new AbortController();
    writeRequest.current = controller;
    try {
      const saved = await api.setStudentConfig(token, payload, controller.signal);
      if (!controller.signal.aborted && writeRequest.current === controller) {
        setForm(saved);
        setNotice("学员功能配置已保存，小程序下次打开学员功能页时获取最新配置。");
      }
    } catch (reason) {
      if (!controller.signal.aborted && !isAborted(reason) && writeRequest.current === controller) setError(errorMessage(reason));
    } finally {
      if (writeRequest.current === controller) {
        writeLock.current = false;
        writeRequest.current = null;
        if (!controller.signal.aborted) {
          setSaving(false);
          onBusyChange(false);
        }
      }
    }
  }
  const invalid = (field: string) => ({
    "aria-invalid": fieldError?.field === field,
    "aria-describedby": fieldError?.field === field ? `student-${field}-error` : undefined,
  });
  const validation = (field: string) => fieldError?.field === field ? (
    <p id={`student-${field}-error`} className="field-error" role="alert">{fieldError.message}</p>
  ) : null;
  const qrUrl = form?.contact_qr_url.trim() ?? "";

  return (
    <section className="student-config-workspace" aria-label="学员功能配置">
      <div className="page-section-header">
        <div>
          <h2>学员功能配置</h2>
          <p className="page-description">维护服务说明、联系微信及微信支付商品。学员在小程序付款后联系负责人办理。</p>
        </div>
      </div>
      {loading ? <p className="empty-state" role="status">正在加载学员功能配置…</p> : !form ? (
        <div className="student-config-feedback">
          <p className="operation-error" role="alert">{error || "配置读取失败，请重新加载"}</p>
          <Button variant="secondary" onClick={() => setRevision((value) => value + 1)}>重新加载</Button>
        </div>
      ) : (
        <form className="student-config-form" onSubmit={save} aria-busy={saving} noValidate>
          <div className="student-config-contact">
            <div className="form-field">
              <Label htmlFor="student-contact-qr">联系微信二维码链接</Label>
              <Input id="student-contact-qr" type="url" inputMode="url" value={form.contact_qr_url} disabled={saving} placeholder="https://…" onChange={(event) => {
                if (writeLock.current) return;
                setForm({ ...form, contact_qr_url: event.target.value });
                clearFeedback();
              }} {...invalid("contact_qr_url")} />
              <p className="field-hint">填写已上传至 MinIO、可公开访问的 HTTPS 图片地址，最多 2048 个字符。停用草稿可留空。</p>
              {validation("contact_qr_url")}
            </div>
            {qrUrl && !qrUrlError(qrUrl) ? <ContactQrPreview key={qrUrl} url={qrUrl} /> : null}
          </div>
          <p className="field-hint">名称、金额和单位固定。请先在微信后台发布对应 99 元、699 元的支付商品，再填入商品 ID；商品价格须与此处一致。</p>
          <div className="student-service-list">
            {form.services.map((service) => (
              <fieldset className="student-service-fields" key={service.code} disabled={saving}>
                <legend>{service.title} · {service.price_cents / 100} 元/{service.unit}</legend>
                <p className="field-hint technical-text">服务代码：{service.code}</p>
                <div className="form-field">
                  <Label htmlFor={`student-${service.code}-description`}>服务说明</Label>
                  <Textarea id={`student-${service.code}-description`} rows={6} value={service.description} onChange={(event) => changeService(service.code, { description: event.target.value })} {...invalid(`${service.code}.description`)} />
                  <p className="field-hint">{[...service.description.trim()].length}/4000 个字符，支持换行。</p>
                  {validation(`${service.code}.description`)}
                </div>
                <div className="form-field">
                  <Label htmlFor={`student-${service.code}-product`}>已发布微信商品 ID</Label>
                  <Input id={`student-${service.code}-product`} autoComplete="off" value={service.product_id} onChange={(event) => changeService(service.code, { product_id: event.target.value })} {...invalid(`${service.code}.product_id`)} />
                  <p className="field-hint">最多 200 个字符，不含空白；对应 {service.price_cents / 100} 元商品。</p>
                  {validation(`${service.code}.product_id`)}
                </div>
                <div className="student-service-toggle">
                  <Switch id={`student-${service.code}-enabled`} checked={service.enabled} disabled={saving} onCheckedChange={(enabled) => changeService(service.code, { enabled })} />
                  <Label htmlFor={`student-${service.code}-enabled`}>开放购买</Label>
                </div>
                <p className="field-hint">开放前须填写服务说明、商品 ID 和联系二维码。关闭后可保存未完成的草稿。</p>
              </fieldset>
            ))}
          </div>
          {error ? <p className="operation-error" role="alert">{error}</p> : null}
          {notice ? <p className="operation-success" role="status">{notice}</p> : null}
          <div className="settings-form-actions">
            <Button type="submit" disabled={saving}><Save />{saving ? "保存中…" : "保存学员功能配置"}</Button>
            {saving ? <span className="field-hint" role="status">正在保存，请等待完成后切换页面或退出。</span> : null}
          </div>
        </form>
      )}
    </section>
  );
}
