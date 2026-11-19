import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type DragEvent,
} from "react";
import {
  ArrowDown,
  ArrowUp,
  ChevronDown,
  ChevronUp,
  Eye,
  EyeOff,
  GripVertical,
  Image as ImageIcon,
  RefreshCw,
  RotateCcw,
  Save,
  Search,
} from "lucide-react";
import type { AdminApi, AdminPageLayout } from "../../api";
import { errorMessage, isAborted } from "../../apiClient";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "../../components/ui/alert-dialog";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Switch } from "../../components/ui/switch";
import { BackgroundImageField } from "./BackgroundImageField";
import { PageLayoutPreview } from "./PageLayoutPreview";
import {
  buildSavePayload,
  clonePageLayout,
  filterAndGroupPages,
  isLayoutDirty,
  moveBlock,
  normalizeImageUrlInput,
  reorderBlocks,
  setAllBlocksVisibility,
  setBlockBackground,
  toggleBlockVisibility,
  validateDraft,
} from "./pageLayoutsModel";

function formatTimestamp(value: string | null): string {
  if (!value) return "使用默认布局";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString("zh-CN", { hour12: false });
}

export function PageLayoutsPage({
  api,
  token,
  onBusyChange,
}: {
  api: AdminApi;
  token: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const [layouts, setLayouts] = useState<AdminPageLayout[]>([]);
  const [selectedKey, setSelectedKey] = useState<string>("home");
  const [draft, setDraft] = useState<AdminPageLayout | null>(null);
  const [keyword, setKeyword] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [uploadingCount, setUploadingCount] = useState(0);
  const [loadError, setLoadError] = useState("");
  const [actionError, setActionError] = useState("");
  const [actionSuccess, setActionSuccess] = useState("");
  const [expandedBlockBg, setExpandedBlockBg] = useState<string | null>(null);
  const [draggingIndex, setDraggingIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);
  const [resetDialogOpen, setResetDialogOpen] = useState(false);
  const [pendingPageKey, setPendingPageKey] = useState<string | null>(null);
  const [reloadSeq, setReloadSeq] = useState(0);

  const actionLock = useRef(false);
  const busy = saving || uploadingCount > 0;

  useEffect(() => {
    onBusyChange(busy);
    return () => onBusyChange(false);
  }, [busy, onBusyChange]);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setLoadError("");
    api
      .pageLayouts(token, controller.signal)
      .then((res) => {
        if (controller.signal.aborted) return;
        const items = Array.isArray(res.items) ? res.items : [];
        setLayouts(items);
        const preferred =
          items.find((item) => item.page_key === selectedKey) ?? items[0];
        if (preferred) {
          setSelectedKey(preferred.page_key);
          setDraft(clonePageLayout(preferred));
        } else {
          setDraft(null);
        }
      })
      .catch((error) => {
        if (!isAborted(error)) {
          setLoadError(errorMessage(error));
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [api, token, reloadSeq]);

  const savedSelected = useMemo(
    () => layouts.find((item) => item.page_key === selectedKey) ?? null,
    [layouts, selectedKey],
  );

  const dirty = useMemo(
    () => isLayoutDirty(savedSelected, draft),
    [savedSelected, draft],
  );

  const groupedPages = useMemo(
    () => filterAndGroupPages(layouts, keyword),
    [layouts, keyword],
  );

  const visibleCount = useMemo(
    () => (draft ? draft.blocks.filter((b) => b.visible).length : 0),
    [draft],
  );

  const handleUploadBusyChange = useCallback((isUploading: boolean) => {
    setUploadingCount((count) => Math.max(0, count + (isUploading ? 1 : -1)));
  }, []);

  function switchToPage(pageKey: string) {
    const target = layouts.find((item) => item.page_key === pageKey);
    if (!target) return;
    setSelectedKey(target.page_key);
    setDraft(clonePageLayout(target));
    setExpandedBlockBg(null);
    setActionError("");
    setActionSuccess("");
  }

  function requestSelectPage(pageKey: string) {
    if (pageKey === selectedKey || busy) return;
    if (dirty) {
      setPendingPageKey(pageKey);
      return;
    }
    switchToPage(pageKey);
  }

  function confirmDiscardAndSwitch() {
    if (!pendingPageKey) return;
    const nextKey = pendingPageKey;
    setPendingPageKey(null);
    switchToPage(nextKey);
  }

  function revertDraft() {
    if (!savedSelected || busy) return;
    setDraft(clonePageLayout(savedSelected));
    setActionError("");
    setActionSuccess("");
  }

  async function handleSave() {
    if (!draft || actionLock.current || busy) return;
    const validationError = validateDraft(draft);
    if (validationError) {
      setActionError(validationError);
      setActionSuccess("");
      return;
    }
    actionLock.current = true;
    setSaving(true);
    setActionError("");
    setActionSuccess("");
    try {
      const saved = await api.savePageLayout(
        token,
        draft.page_key,
        buildSavePayload(draft),
      );
      setLayouts((prev) =>
        prev.map((item) =>
          item.page_key === saved.page_key ? saved : item,
        ),
      );
      setDraft(clonePageLayout(saved));
      setActionSuccess(
        `「${saved.title}」已保存并生效（版本 v${saved.version}），小程序刷新页面即可看到最新布局。`,
      );
    } catch (error) {
      if (!isAborted(error)) {
        setActionError(errorMessage(error));
      }
    } finally {
      actionLock.current = false;
      setSaving(false);
    }
  }

  async function handleResetDefault() {
    if (!draft || actionLock.current || busy) return;
    setResetDialogOpen(false);
    actionLock.current = true;
    setSaving(true);
    setActionError("");
    setActionSuccess("");
    try {
      const resetLayout = await api.resetPageLayout(token, draft.page_key);
      setLayouts((prev) =>
        prev.map((item) =>
          item.page_key === resetLayout.page_key ? resetLayout : item,
        ),
      );
      setDraft(clonePageLayout(resetLayout));
      setExpandedBlockBg(null);
      setActionSuccess(`「${resetLayout.title}」已恢复默认布局与默认背景。`);
    } catch (error) {
      if (!isAborted(error)) {
        setActionError(errorMessage(error));
      }
    } finally {
      actionLock.current = false;
      setSaving(false);
    }
  }

  function handleDragStart(event: DragEvent<HTMLLIElement>, index: number) {
    if (busy) {
      event.preventDefault();
      return;
    }
    setDraggingIndex(index);
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", String(index));
  }

  function handleDragOver(event: DragEvent<HTMLLIElement>, index: number) {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
    if (dragOverIndex !== index) {
      setDragOverIndex(index);
    }
  }

  function handleDrop(event: DragEvent<HTMLLIElement>, toIndex: number) {
    event.preventDefault();
    const fromIndex = draggingIndex;
    setDraggingIndex(null);
    setDragOverIndex(null);
    if (fromIndex === null || fromIndex === toIndex || !draft) return;
    setActionSuccess("");
    setDraft({
      ...draft,
      blocks: reorderBlocks(draft.blocks, fromIndex, toIndex),
    });
  }

  function handleDragEnd() {
    setDraggingIndex(null);
    setDragOverIndex(null);
  }

  if (loading && layouts.length === 0) {
    return (
      <p role="status" className="empty-state">
        正在加载小程序页面装修配置…
      </p>
    );
  }

  if (loadError && layouts.length === 0) {
    return (
      <div className="page-layouts-load-error">
        <p role="alert" className="operation-error">
          {loadError}
        </p>
        <Button
          type="button"
          variant="secondary"
          onClick={() => setReloadSeq((n) => n + 1)}
        >
          <RefreshCw />
          重新加载
        </Button>
      </div>
    );
  }

  return (
    <section className="page-layouts-workspace">
      <div className="page-section-header">
        <div>
          <h2>页面装修</h2>
          <p className="page-description">
            自由拖拽调整小程序全部 {layouts.length} 个页面的卡片顺序、控制显示与隐藏，并可选设置页面背景图与单张卡片背景图。
          </p>
        </div>
        <div className="page-actions">
          <Button
            type="button"
            variant="secondary"
            disabled={busy || loading}
            onClick={() => setReloadSeq((n) => n + 1)}
          >
            <RefreshCw />
            刷新列表
          </Button>
        </div>
      </div>

      {actionError && (
        <p role="alert" className="operation-error">
          {actionError}
        </p>
      )}
      {actionSuccess && (
        <p role="status" className="operation-success">
          {actionSuccess}
        </p>
      )}

      <div className="page-layouts-grid">
        <nav className="page-layouts-catalog" aria-label="小程序页面列表">
          <div className="page-layouts-search">
            <Search aria-hidden="true" />
            <Input
              value={keyword}
              onChange={(e) => setKeyword(e.target.value)}
              placeholder="搜索页面名称或路由…"
              aria-label="搜索小程序页面"
            />
          </div>

          <div className="page-layouts-group-list">
            {groupedPages.length === 0 ? (
              <p className="empty-state">没有匹配的页面</p>
            ) : (
              groupedPages.map((section) => (
                <div key={section.group} className="page-layouts-group">
                  <p className="page-layouts-group-title">
                    {section.label}（{section.items.length}）
                  </p>
                  <ul className="page-layouts-page-list">
                    {section.items.map((item) => {
                      const active = item.page_key === selectedKey;
                      const shown = item.blocks.filter((b) => b.visible).length;
                      return (
                        <li key={item.page_key}>
                          <button
                            type="button"
                            className="page-layouts-page-item"
                            aria-current={active ? "page" : undefined}
                            disabled={busy}
                            onClick={() => requestSelectPage(item.page_key)}
                          >
                            <div className="page-layouts-page-item-top">
                              <strong>{item.title}</strong>
                              {item.customized && (
                                <Badge data-tone="success">已装修</Badge>
                              )}
                            </div>
                            <div className="page-layouts-page-item-meta">
                              <span className="technical-text">
                                {item.page_key}
                              </span>
                              <span>
                                {shown}/{item.blocks.length} 卡片
                              </span>
                            </div>
                          </button>
                        </li>
                      );
                    })}
                  </ul>
                </div>
              ))
            )}
          </div>
        </nav>

        {draft ? (
          <div className="page-layouts-editor">
            <div className="page-layouts-editor-header">
              <div className="page-layouts-editor-title">
                <div className="page-layouts-editor-title-row">
                  <h3>{draft.title}</h3>
                  <Badge data-tone={draft.customized ? "success" : undefined}>
                    {draft.customized ? `已装修 v${draft.version}` : "默认布局"}
                  </Badge>
                  {dirty && <Badge data-tone="warning">未保存改动</Badge>}
                </div>
                <p className="field-hint">
                  {draft.description} ·{" "}
                  <span className="technical-text">{draft.route_path}</span> ·{" "}
                  最近更新：{formatTimestamp(draft.updated_at)}
                </p>
              </div>

              <div className="page-layouts-editor-actions">
                <Button
                  type="button"
                  variant="ghost"
                  disabled={busy || !dirty}
                  onClick={revertDraft}
                >
                  放弃改动
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  disabled={busy || (!draft.customized && !dirty)}
                  onClick={() => setResetDialogOpen(true)}
                >
                  <RotateCcw />
                  恢复默认
                </Button>
                <Button
                  type="button"
                  variant="primary"
                  disabled={busy || !dirty}
                  onClick={handleSave}
                >
                  <Save />
                  {saving ? "保存中…" : "保存并发布"}
                </Button>
              </div>
            </div>

            <div className="page-layouts-card-panel">
              <BackgroundImageField
                id={`page-bg-${draft.page_key}`}
                label="页面背景图（可选）"
                hint="设置后将作为整个页面的背景层（自动叠加可读性遮罩），留空则使用默认背景。"
                value={draft.background_image_url}
                disabled={busy}
                api={api}
                token={token}
                onChange={(next) => {
                  setActionSuccess("");
                  setDraft({ ...draft, background_image_url: next });
                }}
                onUploadBusyChange={handleUploadBusyChange}
              />
            </div>

            <div className="page-layouts-card-panel">
              <div className="page-layouts-blocks-header">
                <div>
                  <h4>
                    页面卡片配置（已显示 {visibleCount}/{draft.blocks.length}）
                  </h4>
                  <p className="field-hint">
                    按住拖拽手柄或使用上/下按钮调整顺序；所有卡片均可自由隐藏或单独设置卡片背景图。
                  </p>
                </div>
                <div className="page-layouts-blocks-bulk">
                  <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    disabled={busy}
                    onClick={() => {
                      setActionSuccess("");
                      setDraft({
                        ...draft,
                        blocks: setAllBlocksVisibility(draft.blocks, true),
                      });
                    }}
                  >
                    <Eye />
                    全部显示
                  </Button>
                  <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    disabled={busy}
                    onClick={() => {
                      setActionSuccess("");
                      setDraft({
                        ...draft,
                        blocks: setAllBlocksVisibility(draft.blocks, false),
                      });
                    }}
                  >
                    <EyeOff />
                    全部隐藏
                  </Button>
                </div>
              </div>

              <ul className="page-layouts-block-list">
                {draft.blocks.map((block, index) => {
                  const isExpanded = expandedBlockBg === block.key;
                  const hasCardBg = Boolean(
                    normalizeImageUrlInput(block.background_image_url),
                  );
                  const rowClass = [
                    "page-layouts-block-item",
                    !block.visible ? "page-layouts-block-item--hidden" : "",
                    draggingIndex === index
                      ? "page-layouts-block-item--dragging"
                      : "",
                    dragOverIndex === index && draggingIndex !== index
                      ? "page-layouts-block-item--drag-over"
                      : "",
                  ]
                    .filter(Boolean)
                    .join(" ");

                  return (
                    <li
                      key={block.key}
                      className={rowClass}
                      draggable={!busy}
                      onDragStart={(e) => handleDragStart(e, index)}
                      onDragOver={(e) => handleDragOver(e, index)}
                      onDrop={(e) => handleDrop(e, index)}
                      onDragEnd={handleDragEnd}
                    >
                      <div className="page-layouts-block-row">
                        <span
                          className="page-layouts-drag-handle"
                          title="按住拖拽排序"
                          aria-hidden="true"
                        >
                          <GripVertical />
                        </span>

                        <span className="page-layouts-block-order">
                          {index + 1}
                        </span>

                        <div className="page-layouts-block-info">
                          <div className="page-layouts-block-title-row">
                            <strong>{block.label}</strong>
                            <span className="technical-text page-layouts-block-key">
                              {block.key}
                            </span>
                            {!block.visible && <Badge>已隐藏</Badge>}
                            {hasCardBg && (
                              <Badge data-tone="success">已设背景</Badge>
                            )}
                          </div>
                          <p className="field-hint">{block.description}</p>
                        </div>

                        <div className="page-layouts-block-controls">
                          <Button
                            type="button"
                            variant="ghost"
                            size="icon-sm"
                            aria-label={`上移 ${block.label}`}
                            title="上移"
                            disabled={busy || index === 0}
                            onClick={() => {
                              setActionSuccess("");
                              setDraft({
                                ...draft,
                                blocks: moveBlock(draft.blocks, index, -1),
                              });
                            }}
                          >
                            <ArrowUp />
                          </Button>
                          <Button
                            type="button"
                            variant="ghost"
                            size="icon-sm"
                            aria-label={`下移 ${block.label}`}
                            title="下移"
                            disabled={busy || index === draft.blocks.length - 1}
                            onClick={() => {
                              setActionSuccess("");
                              setDraft({
                                ...draft,
                                blocks: moveBlock(draft.blocks, index, 1),
                              });
                            }}
                          >
                            <ArrowDown />
                          </Button>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            aria-expanded={isExpanded}
                            disabled={busy}
                            onClick={() =>
                              setExpandedBlockBg(isExpanded ? null : block.key)
                            }
                          >
                            <ImageIcon />
                            背景图
                            {isExpanded ? <ChevronUp /> : <ChevronDown />}
                          </Button>
                          <label className="page-layouts-switch-wrap">
                            <Switch
                              checked={block.visible}
                              disabled={busy}
                              aria-label={`切换 ${block.label} 显示状态`}
                              onCheckedChange={(checked) => {
                                setActionSuccess("");
                                setDraft({
                                  ...draft,
                                  blocks: toggleBlockVisibility(
                                    draft.blocks,
                                    block.key,
                                    checked,
                                  ),
                                });
                              }}
                            />
                            <span>{block.visible ? "显示" : "隐藏"}</span>
                          </label>
                        </div>
                      </div>

                      {isExpanded && (
                        <div className="page-layouts-block-bg-drawer">
                          <BackgroundImageField
                            id={`block-bg-${draft.page_key}-${block.key}`}
                            label={`「${block.label}」卡片背景图（可选）`}
                            hint="设置后仅作用于该卡片容器，自动叠加柔光底色保证文字清晰可读。"
                            value={block.background_image_url}
                            disabled={busy}
                            api={api}
                            token={token}
                            onChange={(next) => {
                              setActionSuccess("");
                              setDraft({
                                ...draft,
                                blocks: setBlockBackground(
                                  draft.blocks,
                                  block.key,
                                  next,
                                ),
                              });
                            }}
                            onUploadBusyChange={handleUploadBusyChange}
                          />
                        </div>
                      )}
                    </li>
                  );
                })}
              </ul>
            </div>
          </div>
        ) : (
          <p className="empty-state">请选择左侧页面开始装修。</p>
        )}

        {draft && <PageLayoutPreview draft={draft} />}
      </div>

      <AlertDialog open={resetDialogOpen} onOpenChange={setResetDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认恢复默认布局？</AlertDialogTitle>
            <AlertDialogDescription>
              将清除「{draft?.title}」的自定义卡片顺序、隐藏配置以及页面/卡片背景图，恢复为系统默认设置并立即生效。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={handleResetDefault}
            >
              确认恢复默认
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={pendingPageKey !== null}
        onOpenChange={(open) => {
          if (!open) setPendingPageKey(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>当前页面有未保存改动</AlertDialogTitle>
            <AlertDialogDescription>
              切换页面将放弃「{draft?.title}」尚未保存的卡片排序、显隐或背景图修改，确定切换吗？
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>留在当前页</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={confirmDiscardAndSwitch}
            >
              放弃改动并切换
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  );
}
