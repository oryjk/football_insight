import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ExternalLink,
  Image as ImageIcon,
  RefreshCw,
  Settings2,
  Smartphone,
} from "lucide-react";
import type { AdminPageLayout } from "../../api";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import {
  PREVIEW_BASE_URL_STORAGE_KEY,
  WECHAT_ONLY_PREVIEW_PAGES,
  buildPreviewApplyMessage,
  buildPreviewIframeUrl,
  buildPreviewNavigateMessage,
  defaultPreviewBaseUrl,
  normalizeImageUrlInput,
  validateImageUrlInput,
} from "./pageLayoutsModel";

type PreviewMode = "h5" | "schematic";

function resolveDefaultBaseUrl(): string {
  const configured =
    typeof import.meta.env.VITE_H5_PREVIEW_URL === "string"
      ? import.meta.env.VITE_H5_PREVIEW_URL
      : "";
  const origin =
    typeof window !== "undefined" ? window.location.origin : undefined;
  const basePath =
    typeof import.meta.env.BASE_URL === "string"
      ? import.meta.env.BASE_URL
      : undefined;
  return defaultPreviewBaseUrl(configured, origin, basePath);
}

function readSavedPreviewBaseUrl(): string {
  try {
    const stored = window.localStorage.getItem(PREVIEW_BASE_URL_STORAGE_KEY);
    if (stored && stored.trim()) return stored.trim();
  } catch {
    // Ignore storage restriction errors.
  }
  return resolveDefaultBaseUrl();
}

export function PageLayoutPreview({ draft }: { draft: AdminPageLayout }) {
  const [mode, setMode] = useState<PreviewMode>("h5");
  const [previewBaseUrl, setPreviewBaseUrl] = useState<string>(
    readSavedPreviewBaseUrl,
  );
  const [urlInput, setUrlInput] = useState<string>(previewBaseUrl);
  const [showUrlConfig, setShowUrlConfig] = useState(false);
  const [reloadNonce, setReloadNonce] = useState(0);
  const [bridgeReady, setBridgeReady] = useState(false);

  const iframeRef = useRef<HTMLIFrameElement | null>(null);
  const mountedRouteRef = useRef<string>(draft.route_path);

  const iframeUrl = useMemo(
    () => buildPreviewIframeUrl(previewBaseUrl, draft.route_path),
    [previewBaseUrl, draft.route_path],
  );

  const isWechatOnly = WECHAT_ONLY_PREVIEW_PAGES.has(draft.page_key);
  const visibleBlocks = useMemo(
    () => draft.blocks.filter((b) => b.visible),
    [draft.blocks],
  );
  const validPageBg = useMemo(() => {
    const url = normalizeImageUrlInput(draft.background_image_url);
    return url && validateImageUrlInput(url) === null ? url : null;
  }, [draft.background_image_url]);

  const sendDraftToIframe = useCallback(() => {
    const win = iframeRef.current?.contentWindow;
    if (!win) return;
    try {
      win.postMessage(buildPreviewApplyMessage(draft), "*");
    } catch {
      // Cross-origin frame may not be ready yet.
    }
  }, [draft]);

  useEffect(() => {
    function handleMessage(event: MessageEvent) {
      if (
        !event.data ||
        typeof event.data !== "object" ||
        (event.data as { type?: string }).type !== "fi-layout-preview:ready"
      ) {
        return;
      }
      setBridgeReady(true);
      sendDraftToIframe();
    }
    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  }, [sendDraftToIframe]);

  // Push live draft updates into the H5 iframe whenever blocks, visibility, or backgrounds change.
  useEffect(() => {
    if (mode !== "h5") return;
    sendDraftToIframe();
    const timer = window.setTimeout(sendDraftToIframe, 300);
    return () => window.clearTimeout(timer);
  }, [mode, sendDraftToIframe]);

  // Notify iframe when switching pages if it is already loaded.
  useEffect(() => {
    if (mode !== "h5") return;
    if (mountedRouteRef.current !== draft.route_path) {
      mountedRouteRef.current = draft.route_path;
      const win = iframeRef.current?.contentWindow;
      if (win && bridgeReady) {
        try {
          win.postMessage(buildPreviewNavigateMessage(draft.route_path), "*");
          win.postMessage(buildPreviewApplyMessage(draft), "*");
        } catch {
          // Reload will occur naturally via iframe src change.
        }
      }
    }
  }, [mode, draft, bridgeReady]);

  function applyPreviewBaseUrl() {
    const trimmed = urlInput.trim().replace(/\/+$/, "");
    if (!trimmed) return;
    setPreviewBaseUrl(trimmed);
    setBridgeReady(false);
    setReloadNonce((n) => n + 1);
    try {
      window.localStorage.setItem(PREVIEW_BASE_URL_STORAGE_KEY, trimmed);
    } catch {
      // Ignore storage failures.
    }
    setShowUrlConfig(false);
  }

  function resetPreviewBaseUrl() {
    const fallback = resolveDefaultBaseUrl();
    setPreviewBaseUrl(fallback);
    setUrlInput(fallback);
    setBridgeReady(false);
    setReloadNonce((n) => n + 1);
    try {
      window.localStorage.removeItem(PREVIEW_BASE_URL_STORAGE_KEY);
    } catch {
      // Ignore storage failures.
    }
    setShowUrlConfig(false);
  }

  return (
    <aside className="page-layouts-preview-panel" aria-label="实时预览">
      <div className="page-layouts-preview-toolbar">
        <div className="page-layouts-preview-modes" role="tablist">
          <Button
            type="button"
            variant={mode === "h5" ? "primary" : "secondary"}
            size="sm"
            role="tab"
            aria-selected={mode === "h5"}
            onClick={() => setMode("h5")}
          >
            <Smartphone />
            H5 实时预览
          </Button>
          <Button
            type="button"
            variant={mode === "schematic" ? "primary" : "secondary"}
            size="sm"
            role="tab"
            aria-selected={mode === "schematic"}
            onClick={() => setMode("schematic")}
          >
            结构预览
          </Button>
        </div>

        <div className="page-layouts-preview-tools">
          {mode === "h5" && (
            <>
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                title="刷新 H5 预览"
                aria-label="刷新 H5 预览"
                onClick={() => {
                  setBridgeReady(false);
                  setReloadNonce((n) => n + 1);
                }}
              >
                <RefreshCw />
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                title="配置 H5 预览地址"
                aria-label="配置 H5 预览地址"
                onClick={() => setShowUrlConfig((v) => !v)}
              >
                <Settings2 />
              </Button>
              {iframeUrl && (
                <a
                  href={iframeUrl}
                  target="_blank"
                  rel="noreferrer"
                  className="page-layouts-external-link"
                  title="在新标签页打开 H5"
                  aria-label="在新标签页打开 H5"
                >
                  <ExternalLink />
                </a>
              )}
            </>
          )}
        </div>
      </div>

      {mode === "h5" && showUrlConfig && (
        <div className="page-layouts-preview-url-box">
          <Input
            value={urlInput}
            onChange={(e) => setUrlInput(e.target.value)}
            placeholder="例如 http://172.16.60.233/football-admin/preview 或 http://localhost:5173"
            aria-label="H5 预览基地址"
          />
          <Button type="button" size="sm" onClick={applyPreviewBaseUrl}>
            应用
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={resetPreviewBaseUrl}
          >
            默认
          </Button>
        </div>
      )}

      <div className="page-layouts-preview-status">
        <span className="technical-text">{draft.route_path}</span>
        {mode === "h5" ? (
          <Badge data-tone={bridgeReady ? "success" : undefined}>
            {bridgeReady ? "实时联动已连接" : "等待 H5 响应"}
          </Badge>
        ) : (
          <Badge>可见 {visibleBlocks.length}/{draft.blocks.length}</Badge>
        )}
      </div>

      {mode === "h5" && isWechatOnly && (
        <p className="field-hint">
          该页面仅在微信小程序运行，H5 端会展示引导提示；可切换到「结构预览」查看卡片顺序与背景图。
        </p>
      )}

      <div className="page-layouts-phone-shell">
        <div className="page-layouts-phone-notch" aria-hidden="true" />
        {mode === "h5" ? (
          <div className="page-layouts-phone-screen">
            <iframe
              key={`${previewBaseUrl}:${reloadNonce}`}
              ref={iframeRef}
              title={`${draft.title} H5 实时预览`}
              src={iframeUrl}
              className="page-layouts-iframe"
              onLoad={() => {
                sendDraftToIframe();
                window.setTimeout(sendDraftToIframe, 400);
              }}
            />
          </div>
        ) : (
          <div
            className="page-layouts-phone-screen page-layouts-schematic-screen"
            style={
              validPageBg
                ? {
                    backgroundImage: `linear-gradient(180deg, rgba(255,255,255,0.55) 0%, rgba(255,255,255,0.88) 100%), url(${validPageBg})`,
                  }
                : undefined
            }
          >
            <div className="page-layouts-schematic-nav">
              <strong>{draft.title}</strong>
              <span>{draft.page_key}</span>
            </div>

            {validPageBg && (
              <div className="page-layouts-schematic-bg-tag">
                <ImageIcon />
                <span>已设置页面背景图</span>
              </div>
            )}

            <div className="page-layouts-schematic-cards">
              {visibleBlocks.length === 0 ? (
                <div className="page-layouts-schematic-empty">
                  当前页面所有卡片均已隐藏
                </div>
              ) : (
                visibleBlocks.map((block, index) => {
                  const blockBg = normalizeImageUrlInput(
                    block.background_image_url,
                  );
                  const validBlockBg =
                    blockBg && validateImageUrlInput(blockBg) === null
                      ? blockBg
                      : null;
                  return (
                    <div
                      key={block.key}
                      className={
                        validBlockBg
                          ? "page-layouts-schematic-card page-layouts-schematic-card--bg"
                          : "page-layouts-schematic-card"
                      }
                      style={
                        validBlockBg
                          ? {
                              backgroundImage: `linear-gradient(180deg, rgba(255,255,255,0.72), rgba(255,255,255,0.86)), url(${validBlockBg})`,
                            }
                          : undefined
                      }
                    >
                      <div className="page-layouts-schematic-card-head">
                        <span className="page-layouts-schematic-index">
                          #{index + 1}
                        </span>
                        <strong>{block.label}</strong>
                        {validBlockBg && (
                          <span className="page-layouts-schematic-bg-pill">
                            卡片背景
                          </span>
                        )}
                      </div>
                      <p className="page-layouts-schematic-card-desc">
                        {block.description}
                      </p>
                    </div>
                  );
                })
              )}
            </div>
          </div>
        )}
      </div>
    </aside>
  );
}
