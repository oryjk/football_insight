import { useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";
import type {
  AdminApi,
  AdminTelegraphComment,
  AdminTelegraphCommentStatus,
  AdminTelegraphCommentStatusFilter,
} from "../../api";
import { errorMessage } from "../../apiClient";
import { Button } from "../../components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../../components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/table";
import { PurchaseAvatar } from "../products/ProductUserAvatar";
import {
  commentCreatedLabel,
  commentEntryLabel,
  commentStatusLabel,
} from "./telegraphCommentsModel";

export function TelegraphCommentsPage({
  api,
  token,
  onBusyChange,
}: {
  api: AdminApi;
  token: string;
  onBusyChange?: (busy: boolean) => void;
}) {
  const [filter, setFilter] = useState<AdminTelegraphCommentStatusFilter>("all");
  const [commentsEnabled, setCommentsEnabled] = useState(true);
  const [items, setItems] = useState<AdminTelegraphComment[]>([]);
  const [loading, setLoading] = useState(true);
  const [updatingConfig, setUpdatingConfig] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<AdminTelegraphComment | null>(
    null,
  );
  const [error, setError] = useState("");
  const [revision, setRevision] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError("");
    api
      .telegraphComments(token, filter, 50, controller.signal)
      .then((res) => {
        if (controller.signal.aborted) return;
        setCommentsEnabled(res.comments_enabled);
        setItems(res.items);
      })
      .catch((reason) => {
        if (!controller.signal.aborted) setError(errorMessage(reason));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [api, token, filter, revision]);

  useEffect(() => {
    onBusyChange?.(updatingConfig || busyId !== null);
  }, [onBusyChange, updatingConfig, busyId]);

  useEffect(() => () => onBusyChange?.(false), [onBusyChange]);

  async function toggleCommentsEnabled() {
    setUpdatingConfig(true);
    setError("");
    try {
      const res = await api.setTelegraphCommentsConfig(token, !commentsEnabled);
      setCommentsEnabled(res.comments_enabled);
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setUpdatingConfig(false);
    }
  }

  async function updateStatus(
    commentId: string,
    nextStatus: AdminTelegraphCommentStatus,
  ) {
    setBusyId(commentId);
    setError("");
    try {
      await api.setTelegraphCommentStatus(token, commentId, nextStatus);
      setDeleteTarget((current) => (current?.id === commentId ? null : current));
      setRevision((value) => value + 1);
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setBusyId(null);
    }
  }

  return (
    <div className="topic-readers">
      <div className="topic-readers-toolbar">
        <select
          className="topic-readers-select"
          aria-label="筛选评论状态"
          value={filter}
          disabled={loading}
          onChange={(event) =>
            setFilter(event.target.value as AdminTelegraphCommentStatusFilter)
          }
        >
          <option value="all">全部未删评论</option>
          <option value="visible">仅展示中</option>
          <option value="hidden">仅已隐藏</option>
        </select>

        <Button
          variant={commentsEnabled ? "secondary" : "default"}
          disabled={loading || updatingConfig || busyId !== null}
          onClick={toggleCommentsEnabled}
        >
          {updatingConfig
            ? "保存中…"
            : commentsEnabled
              ? "全站评论：已开启（点击关闭）"
              : "全站评论：已关闭（点击开启）"}
        </Button>

        <Button
          variant="secondary"
          disabled={loading || updatingConfig || busyId !== null}
          aria-label="刷新"
          onClick={() => setRevision((value) => value + 1)}
        >
          <RefreshCw />
          刷新
        </Button>
      </div>

      <p className="field-hint">
        支持隐藏或删除违规电报评论；关闭全站评论开关后，前台详情页将隐藏评论区并拒绝新发言。
      </p>

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

      {!loading && !error && items.length === 0 && (
        <p className="empty-state">暂无电报评论记录。</p>
      )}

      {!loading && items.length > 0 && (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>用户</TableHead>
              <TableHead>所属电报</TableHead>
              <TableHead>评论内容</TableHead>
              <TableHead>状态</TableHead>
              <TableHead>时间</TableHead>
              <TableHead>操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.map((item) => {
              const rowBusy = busyId === item.id;
              return (
                <TableRow key={item.id}>
                  <TableCell>
                    <span className="topic-reader-user">
                      <PurchaseAvatar url={item.author_avatar_url} />
                      {item.author_display_name}
                    </span>
                  </TableCell>
                  <TableCell>
                    {commentEntryLabel(item.entry_title, item.entry_id)}
                  </TableCell>
                  <TableCell>{item.content}</TableCell>
                  <TableCell>{commentStatusLabel(item.status)}</TableCell>
                  <TableCell>{commentCreatedLabel(item.created_at)}</TableCell>
                  <TableCell>
                    <div className="row-actions">
                      {item.status === "visible" ? (
                        <Button
                          variant="secondary"
                          size="sm"
                          disabled={rowBusy || busyId !== null}
                          onClick={() => void updateStatus(item.id, "hidden")}
                        >
                          隐藏
                        </Button>
                      ) : (
                        <Button
                          variant="secondary"
                          size="sm"
                          disabled={rowBusy || busyId !== null}
                          onClick={() => void updateStatus(item.id, "visible")}
                        >
                          恢复展示
                        </Button>
                      )}
                      <Button
                        variant="destructive"
                        size="sm"
                        disabled={rowBusy || busyId !== null}
                        onClick={() => setDeleteTarget(item)}
                      >
                        删除
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      )}

      <Dialog
        open={deleteTarget !== null}
        onOpenChange={(open) => {
          if (!open && busyId === null) {
            setDeleteTarget(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除评论</DialogTitle>
            <DialogDescription>
              确认删除「{deleteTarget?.author_display_name}」的评论吗？删除后不可恢复。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="secondary"
              disabled={busyId !== null}
              onClick={() => setDeleteTarget(null)}
            >
              取消
            </Button>
            <Button
              variant="destructive"
              disabled={!deleteTarget || busyId !== null}
              onClick={() => {
                if (deleteTarget) {
                  void updateStatus(deleteTarget.id, "deleted");
                }
              }}
            >
              {busyId !== null ? "删除中…" : "确认删除"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
