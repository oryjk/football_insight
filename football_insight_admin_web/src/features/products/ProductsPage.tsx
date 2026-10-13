import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  ChevronLeft,
  ChevronRight,
  Eye,
  Pencil,
  Plus,
  RefreshCw,
  Search,
  Trash2,
} from "lucide-react";
import type { AdminApi, Product, ProductFilters, ProductPage } from "../../api";
import { errorMessage, isAborted } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Badge } from "../../components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../../components/ui/dialog";
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
import {
  FormError,
  centsToAmount,
  formToPayload,
  formatTime,
  membershipTiers,
  newProductForm,
  productToForm,
} from "./productsModel";
import { ProductFields } from "./ProductFields";
type DialogMode = "create" | "edit" | "detail" | "toggle" | "delete";
interface ActiveDialog {
  mode: DialogMode;
  key?: string;
}
const initialFilters: ProductFilters = {
  query: "",
  membership_tier: "",
  enabled: "",
  page: 1,
  page_size: 20,
};
export function ProductsPage({
  api,
  token,
  onBusyChange,
}: {
  api: AdminApi;
  token: string;
  onBusyChange?: (busy: boolean) => void;
}) {
  const [filters, setFilters] = useState(initialFilters);
  const [query, setQuery] = useState("");
  const [data, setData] = useState<ProductPage | null>(null);
  const [loading, setLoading] = useState(true);
  const [listError, setListError] = useState("");
  const [notice, setNotice] = useState("");
  const [revision, setRevision] = useState(0);
  const [dialog, setDialog] = useState<ActiveDialog | null>(null);
  const [product, setProduct] = useState<Product | null>(null);
  const [form, setForm] = useState(newProductForm);
  const [detailLoading, setDetailLoading] = useState(false);
  const [dialogError, setDialogError] = useState("");
  const [fieldError, setFieldError] = useState<{
    field: string;
    message: string;
  } | null>(null);
  const [saving, setSaving] = useState(false);
  const saveLock = useRef(false);
  const writeController = useRef<AbortController | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setListError("");
    setData(null);
    api
      .products(token, filters, controller.signal)
      .then((page) => {
        if (controller.signal.aborted) return;
        if (page.items.length === 0 && filters.page > 1 && page.total > 0) {
          setFilters((current) => ({
            ...current,
            page: Math.max(1, Math.ceil(page.total / current.page_size)),
          }));
          return;
        }
        if (page.items.length === 0 && filters.page > 1 && page.total === 0) {
          setFilters((current) => ({ ...current, page: 1 }));
          return;
        }
        setData(page);
      })
      .catch((error) => {
        if (!controller.signal.aborted) setListError(errorMessage(error));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [api, token, filters, revision]);
  useEffect(() => {
    setDialogError("");
    setFieldError(null);
    setProduct(null);
    if (!dialog) return;
    if (dialog.mode === "create") {
      setForm(newProductForm());
      setDetailLoading(false);
      return;
    }
    const controller = new AbortController();
    setDetailLoading(true);
    api
      .product(token, dialog.key!, controller.signal)
      .then((value) => {
        if (!controller.signal.aborted) {
          setProduct(value);
          setForm(productToForm(value));
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) setDialogError(errorMessage(error));
      })
      .finally(() => {
        if (!controller.signal.aborted) setDetailLoading(false);
      });
    return () => controller.abort();
  }, [api, token, dialog]);
  useEffect(
    () => () => {
      writeController.current?.abort();
      onBusyChange?.(false);
    },
    [onBusyChange],
  );
  function closeDialog() {
    if (!saveLock.current) setDialog(null);
  }
  function refresh() {
    setRevision((v) => v + 1);
  }
  function search(event: FormEvent) {
    event.preventDefault();
    setFilters((current) => ({ ...current, query: query.trim(), page: 1 }));
  }
  async function save() {
    if (saveLock.current || !dialog) return;
    setFieldError(null);
    setDialogError("");
    let payload;
    try {
      payload = formToPayload(form);
    } catch (error) {
      if (error instanceof FormError)
        setFieldError({ field: error.field, message: error.message });
      else setDialogError(errorMessage(error));
      return;
    }
    await mutate(async (signal) => {
      const saved = await api.save(
        token,
        payload,
        dialog.mode === "edit" ? dialog.key : undefined,
        signal,
      );
      return `已保存「${saved.name}」（${saved.product_key}）`;
    });
  }
  async function confirm() {
    if (!product || !dialog) return;
    await mutate(async (signal) => {
      if (dialog.mode === "delete") {
        await api.delete(token, product.product_key, signal);
        return `已删除「${product.name}」`;
      }
      const {
        membership_tier,
        name,
        product_id,
        price_cents,
        membership_duration_months,
        enabled,
        remark,
        allowed_user_ids,
      } = product;
      const saved = await api.save(
        token,
        {
          membership_tier,
          name,
          product_id,
          price_cents,
          membership_duration_months: membership_duration_months ?? 12,
          enabled: !enabled,
          remark,
          allowed_user_ids,
        },
        product.product_key,
        signal,
      );
      return `已${saved.enabled ? "启用" : "停用"}「${saved.name}」`;
    });
  }
  async function mutate(action: (signal: AbortSignal) => Promise<string>) {
    if (saveLock.current) return;
    saveLock.current = true;
    onBusyChange?.(true);
    setSaving(true);
    setDialogError("");
    const controller = new AbortController();
    writeController.current = controller;
    try {
      const message = await action(controller.signal);
      if (!controller.signal.aborted) {
        setNotice(message);
        setDialog(null);
        refresh();
      }
    } catch (error) {
      if (!controller.signal.aborted && !isAborted(error))
        setDialogError(errorMessage(error));
    } finally {
      saveLock.current = false;
      if (!controller.signal.aborted) {
        setSaving(false);
        onBusyChange?.(false);
      }
    }
  }
  const pages = Math.max(1, Math.ceil((data?.total ?? 0) / filters.page_size));
  const normalOpen = !!dialog && !["toggle", "delete"].includes(dialog.mode);
  const dangerOpen = !!dialog && ["toggle", "delete"].includes(dialog.mode);
  return (
    <section className="products-workspace" aria-label="商品管理工作区">
      <div className="page-section-header">
        <div>
          <h2>会员商品</h2>
          <p className="page-description">
            管理微信道具映射，金额须与已发布道具及业务实付金额一致。此处不修改公开会员标价或权益。
          </p>
        </div>
        <div className="page-actions">
          <Button variant="secondary" onClick={refresh} disabled={loading}>
            <RefreshCw />
            刷新
          </Button>
          <Button onClick={() => setDialog({ mode: "create" })}>
            <Plus />
            新建商品
          </Button>
        </div>
      </div>
      <form onSubmit={search} className="product-filters">
        <div className="filter-keyword form-field">
          <Label htmlFor="keyword">关键词</Label>
          <Input
            id="keyword"
            placeholder="名称、道具 ID、业务键"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        <div className="form-field">
          <Label htmlFor="tier-filter">会员等级</Label>
          <Select
            value={filters.membership_tier || "all"}
            onValueChange={(value) =>
              setFilters((current) => ({
                ...current,
                membership_tier: value === "all" ? "" : value,
                page: 1,
              }))
            }
          >
            <SelectTrigger id="tier-filter">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部等级</SelectItem>
              {membershipTiers.map((t) => (
                <SelectItem value={t} key={t}>
                  {t}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="form-field">
          <Label htmlFor="enabled-filter">启用状态</Label>
          <Select
            value={filters.enabled || "all"}
            onValueChange={(value) =>
              setFilters((current) => ({
                ...current,
                enabled: value === "all" ? "" : value,
                page: 1,
              }))
            }
          >
            <SelectTrigger id="enabled-filter">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部状态</SelectItem>
              <SelectItem value="true">已启用</SelectItem>
              <SelectItem value="false">已停用</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <Button variant="secondary" type="submit">
          <Search />
          查询
        </Button>
        <Button
          variant="ghost"
          type="button"
          onClick={() => {
            setQuery("");
            setFilters({ ...initialFilters });
          }}
        >
          重置
        </Button>
      </form>
      {notice && (
        <p className="operation-success" role="status">
          {notice}
        </p>
      )}
      {listError && (
        <div className="operation-error" role="alert">
          <span>{listError}</span>
          <Button variant="secondary" onClick={refresh}>
            重试
          </Button>
        </div>
      )}
      {loading && (
        <p className="empty-state" role="status">
          正在加载商品…
        </p>
      )}
      {!loading && data && (
        <>
          <Table className="products-table" aria-label="会员微信道具">
            <TableHeader>
              <TableRow>
                <TableHead>商品名称 / 业务键</TableHead>
                <TableHead>等级</TableHead>
                <TableHead>微信道具 ID</TableHead>
                <TableHead>道具金额</TableHead>
                <TableHead>会员有效期</TableHead>
                <TableHead>状态</TableHead>
                <TableHead>购买范围</TableHead>
                <TableHead>更新时间</TableHead>
                <TableHead className="product-actions-cell">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.items.map((item) => (
                <TableRow key={item.product_key}>
                  <TableCell>
                    <strong>{item.name}</strong>
                    <span className="product-key">{item.product_key}</span>
                  </TableCell>
                  <TableCell>{item.membership_tier}</TableCell>
                  <TableCell className="technical-text">
                    {item.product_id}
                  </TableCell>
                  <TableCell className="amount-cell">
                    ¥{centsToAmount(item.price_cents)}
                  </TableCell>
                  <TableCell>
                    {item.membership_duration_months ?? 12} 月
                  </TableCell>
                  <TableCell>
                    <Badge data-tone={item.enabled ? "success" : "neutral"}>
                      {item.enabled ? "已启用" : "已停用"}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    {item.allowed_user_ids.length
                      ? `指定 ${item.allowed_user_ids.length} 个账号`
                      : "公开"}
                  </TableCell>
                  <TableCell className="date-cell">
                    {formatTime(item.updated_at)}
                  </TableCell>
                  <TableCell className="product-actions-cell">
                    <div className="row-actions">
                      <Button
                        variant="ghost"
                        size="icon"
                        aria-label={`查看 ${item.name}`}
                        onClick={() =>
                          setDialog({ mode: "detail", key: item.product_key })
                        }
                      >
                        <Eye />
                      </Button>
                      <Button
                        variant="secondary"
                        size="icon"
                        aria-label={`编辑 ${item.name}`}
                        onClick={() =>
                          setDialog({ mode: "edit", key: item.product_key })
                        }
                      >
                        <Pencil />
                      </Button>
                      <Button
                        variant={item.enabled ? "destructive" : "secondary"}
                        onClick={() =>
                          setDialog({ mode: "toggle", key: item.product_key })
                        }
                      >
                        {item.enabled ? "停用" : "启用"}
                      </Button>
                      <Button
                        variant="destructive"
                        size="icon"
                        aria-label={`删除 ${item.name}`}
                        onClick={() =>
                          setDialog({ mode: "delete", key: item.product_key })
                        }
                      >
                        <Trash2 />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
              {!data.items.length && (
                <TableRow>
                  <TableCell colSpan={9}>
                    <div className="empty-state">
                      {filters.query ||
                      filters.membership_tier ||
                      filters.enabled
                        ? "没有符合条件的商品，请调整筛选。"
                        : "暂无会员商品，可新建微信道具映射。"}
                    </div>
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
          <div className="product-pagination">
            <span>
              共 {data.total} 条 · 第 {filters.page} / {pages} 页
            </span>
            <div className="pagination-actions">
              <Label htmlFor="page-size">每页</Label>
              <Select
                value={String(filters.page_size)}
                onValueChange={(value) =>
                  setFilters((current) => ({
                    ...current,
                    page_size: Number(value),
                    page: 1,
                  }))
                }
              >
                <SelectTrigger id="page-size">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {[10, 20, 50, 100].map((n) => (
                    <SelectItem key={n} value={String(n)}>
                      {n} 条
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button
                variant="secondary"
                size="icon"
                aria-label="上一页"
                disabled={filters.page <= 1}
                onClick={() =>
                  setFilters((current) => ({
                    ...current,
                    page: current.page - 1,
                  }))
                }
              >
                <ChevronLeft />
              </Button>
              <Button
                variant="secondary"
                size="icon"
                aria-label="下一页"
                disabled={filters.page >= pages}
                onClick={() =>
                  setFilters((current) => ({
                    ...current,
                    page: current.page + 1,
                  }))
                }
              >
                <ChevronRight />
              </Button>
            </div>
          </div>
        </>
      )}
      <Dialog
        open={normalOpen}
        onOpenChange={(open) => {
          if (!open) closeDialog();
        }}
      >
        <DialogContent
          className="product-dialog"
          showCloseButton={!saving}
          onEscapeKeyDown={(event) => {
            if (saving) event.preventDefault();
          }}
          onInteractOutside={(event) => event.preventDefault()}
        >
          <DialogHeader>
            <DialogTitle>
              {dialog?.mode === "create"
                ? "新建商品"
                : dialog?.mode === "edit"
                  ? "编辑商品"
                  : "商品详情"}
            </DialogTitle>
            <DialogDescription>
              {dialog?.mode === "detail"
                ? "查看商品配置与购买范围。"
                : "道具金额与业务实付金额须一致；更改等级或金额会更改业务键，影响后续购买匹配。"}
            </DialogDescription>
          </DialogHeader>
          {detailLoading ? (
            <p role="status">正在读取商品…</p>
          ) : dialog?.mode === "detail" ? (
            product && (
              <dl className="product-details">
                <dt>名称</dt>
                <dd>{product.name}</dd>
                <dt>业务键</dt>
                <dd className="technical-text">{product.product_key}</dd>
                <dt>会员等级</dt>
                <dd>{product.membership_tier}</dd>
                <dt>微信道具 ID</dt>
                <dd className="technical-text">{product.product_id}</dd>
                <dt>道具金额</dt>
                <dd>¥{centsToAmount(product.price_cents)}</dd>
                <dt>会员有效期</dt>
                <dd>{product.membership_duration_months ?? 12} 月</dd>
                <dt>启用状态</dt>
                <dd>{product.enabled ? "已启用" : "已停用"}</dd>
                <dt>购买范围</dt>
                <dd>
                  {product.allowed_user_ids.length
                    ? product.allowed_user_ids.map((id) => (
                        <div className="technical-text" key={id}>
                          {id}
                        </div>
                      ))
                    : "公开"}
                </dd>
                <dt>备注</dt>
                <dd className="multiline-text">{product.remark || "—"}</dd>
                <dt>创建时间</dt>
                <dd>{formatTime(product.created_at)}</dd>
                <dt>更新时间</dt>
                <dd>{formatTime(product.updated_at)}</dd>
              </dl>
            )
          ) : (
            (dialog?.mode === "create" || product) && (
              <form
                id="product-editor"
                onSubmit={(event) => {
                  event.preventDefault();
                  void save();
                }}
              >
                <ProductFields
                  api={api}
                  token={token}
                  value={form}
                  onChange={(value) => {
                    setForm(value);
                    setFieldError(null);
                  }}
                  disabled={saving}
                  fieldError={fieldError}
                />
              </form>
            )
          )}
          {dialogError && (
            <p role="alert" className="operation-error">
              {dialogError}
            </p>
          )}
          <DialogFooter>
            <Button variant="secondary" disabled={saving} onClick={closeDialog}>
              关闭
            </Button>
            {!detailLoading && dialog?.mode === "detail" && product && (
              <Button
                variant="secondary"
                onClick={() =>
                  setDialog({ mode: "edit", key: product.product_key })
                }
              >
                <Pencil />
                编辑
              </Button>
            )}
            {!detailLoading &&
              (dialog?.mode === "create" ||
                (dialog?.mode === "edit" && product)) && (
                <Button type="submit" form="product-editor" disabled={saving}>
                  {saving ? "保存中…" : "保存商品"}
                </Button>
              )}
            {dialogError && !product && dialog?.mode !== "create" && (
              <Button
                variant="secondary"
                onClick={() => {
                  closeDialog();
                  refresh();
                }}
              >
                返回并刷新
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <AlertDialog
        open={dangerOpen}
        onOpenChange={(open) => {
          if (!open) closeDialog();
        }}
      >
        <AlertDialogContent
          onEscapeKeyDown={(event) => {
            if (saving) event.preventDefault();
          }}
        >
          <AlertDialogHeader>
            <AlertDialogTitle>
              {dialog?.mode === "delete"
                ? "确认删除商品"
                : product?.enabled
                  ? "确认停用商品"
                  : "确认启用商品"}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {detailLoading
                ? "正在读取当前商品…"
                : product
                  ? `「${product.name}」 · ${product.product_key}。${dialog?.mode === "delete" ? "删除后无法恢复此映射，" : ""}${product.enabled || dialog?.mode === "delete" ? "此操作影响后续新购买，历史订单保留购买时快照。" : "启用后符合购买范围的账号可使用此道具映射。请确认道具已发布且金额匹配。"}`
                  : "商品无法读取，请返回刷新列表。"}
            </AlertDialogDescription>
          </AlertDialogHeader>
          {dialogError && (
            <p role="alert" className="operation-error">
              {dialogError}
            </p>
          )}
          <AlertDialogFooter>
            <AlertDialogCancel disabled={saving}>取消</AlertDialogCancel>
            <AlertDialogAction
              variant={
                dialog?.mode === "delete" || product?.enabled
                  ? "destructive"
                  : "primary"
              }
              disabled={saving || detailLoading || !product}
              onClick={(event) => {
                event.preventDefault();
                void confirm();
              }}
            >
              {saving
                ? "处理中…"
                : dialog?.mode === "delete"
                  ? "确认删除"
                  : product?.enabled
                    ? "确认停用"
                    : "确认启用"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  );
}
