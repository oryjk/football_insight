import { useRef, useState, type ChangeEvent } from "react";
import { Image as ImageIcon, Upload, X } from "lucide-react";
import type { AdminApi } from "../../api";
import { errorMessage, isAborted } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import {
  normalizeImageUrlInput,
  validateImageUrlInput,
  validateUploadImageFile,
} from "./pageLayoutsModel";

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === "string") resolve(reader.result);
      else reject(new Error("读取图片失败"));
    };
    reader.onerror = () => reject(new Error("读取图片失败"));
    reader.readAsDataURL(file);
  });
}

export function BackgroundImageField({
  id,
  label,
  hint,
  value,
  disabled,
  api,
  token,
  onChange,
  onUploadBusyChange,
}: {
  id: string;
  label: string;
  hint?: string;
  value: string | null;
  disabled?: boolean;
  api: AdminApi;
  token: string;
  onChange: (next: string | null) => void;
  onUploadBusyChange?: (busy: boolean) => void;
}) {
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [uploading, setUploading] = useState(false);
  const [uploadError, setUploadError] = useState("");
  const [previewFailed, setPreviewFailed] = useState(false);

  const normalized = normalizeImageUrlInput(value);
  const urlError = validateImageUrlInput(value);
  const hasValidUrl = Boolean(normalized && !urlError);

  async function handleFileChange(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;

    const validationError = validateUploadImageFile(file);
    if (validationError) {
      setUploadError(validationError);
      return;
    }

    setUploadError("");
    setUploading(true);
    onUploadBusyChange?.(true);
    try {
      const dataUrl = await readFileAsDataUrl(file);
      const uploaded = await api.uploadLayoutImage(token, dataUrl);
      setPreviewFailed(false);
      onChange(uploaded.url);
    } catch (error) {
      if (!isAborted(error)) {
        setUploadError(errorMessage(error));
      }
    } finally {
      setUploading(false);
      onUploadBusyChange?.(false);
    }
  }

  return (
    <div className="page-layouts-bg-field">
      <div className="page-layouts-bg-field-header">
        <Label htmlFor={id}>{label}</Label>
        {normalized && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            disabled={disabled || uploading}
            onClick={() => {
              setUploadError("");
              setPreviewFailed(false);
              onChange(null);
            }}
          >
            <X />
            清除背景图
          </Button>
        )}
      </div>

      <div className="page-layouts-bg-input-row">
        <Input
          id={id}
          type="url"
          value={value ?? ""}
          placeholder="可选：粘贴 https:// 图片链接，或点击右侧上传"
          disabled={disabled || uploading}
          onChange={(event) => {
            setUploadError("");
            setPreviewFailed(false);
            const raw = event.target.value;
            onChange(raw.trim().length === 0 ? null : raw);
          }}
        />
        <input
          ref={fileInputRef}
          type="file"
          accept="image/png,image/jpeg,image/webp,image/gif"
          className="page-layouts-file-input"
          onChange={handleFileChange}
        />
        <Button
          type="button"
          variant="secondary"
          disabled={disabled || uploading}
          onClick={() => fileInputRef.current?.click()}
        >
          <Upload />
          {uploading ? "上传中…" : "上传图片"}
        </Button>
      </div>

      {hint && <p className="field-hint">{hint}</p>}
      {urlError && (
        <p role="alert" className="field-error">
          {urlError}
        </p>
      )}
      {uploadError && (
        <p role="alert" className="field-error">
          {uploadError}
        </p>
      )}

      {hasValidUrl && normalized && (
        <div className="page-layouts-bg-thumb-wrap">
          {previewFailed ? (
            <div className="page-layouts-bg-thumb-fallback">
              <ImageIcon />
              <span>图片缩略图加载失败，请确认链接可公开访问</span>
            </div>
          ) : (
            <img
              src={normalized}
              alt={`${label}预览`}
              className="page-layouts-bg-thumb"
              onError={() => setPreviewFailed(true)}
            />
          )}
        </div>
      )}
    </div>
  );
}
