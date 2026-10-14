import { useEffect, useState } from "react";
import {
  ChevronLeft,
  ChevronRight,
  RefreshCw,
  ChevronDown,
} from "lucide-react";
import type {
  AdminApi,
  Product,
  ProductPurchasePage,
  MembershipOrigin,
} from "../../api";
import { errorMessage } from "../../apiClient";
import { MembershipOriginBadge } from "./MembershipOriginBadge";
import { MembershipOriginFilters } from "./MembershipOriginFilters";
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "../../components/ui/collapsible";
import { ProductMembers } from "./ProductMembers";
import { PurchaseAvatar } from "./ProductUserAvatar";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { Label } from "../../components/ui/label";
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
  purchaseAmount,
  purchaseDate,
  purchaseExpiry,
  purchaseSummaryValues,
  assertPurchaseIdentity,
  PurchaseIdentityError,
} from "./purchasesModel";
export function ProductPurchasesDialog({
  api,
  token,
  product,
  onClose,
  onReloadProduct,
}: {
  api: AdminApi;
  token: string;
  product: Pick<Product, "product_key" | "name" | "catalog_product_id">;
  onClose: () => void;
  onReloadProduct: () => void;
}) {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [revision, setRevision] = useState(0);
  const [data, setData] = useState<ProductPurchasePage | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [identityChanged, setIdentityChanged] = useState(false);
  const [origins, setOrigins] = useState<MembershipOrigin[]>([
    "purchase",
    "referral",
    "unknown",
  ]);
  const [ordersOpen, setOrdersOpen] = useState(false);
  const hasPurchase = origins.includes("purchase");
  useEffect(() => {
    if (!hasPurchase || !ordersOpen) return;
    const controller = new AbortController();
    setLoading(true);
    setError("");
    setData(null);
    setIdentityChanged(false);
    try {
      assertPurchaseIdentity(
        product.catalog_product_id,
        product.catalog_product_id,
      );
    } catch (error) {
      setError(errorMessage(error));
      setIdentityChanged(true);
      setLoading(false);
      return () => controller.abort();
    }
    api
      .productPurchases(
        token,
        product.product_key,
        page,
        pageSize,
        controller.signal,
      )
      .then((value) => {
        if (controller.signal.aborted) return;
        assertPurchaseIdentity(
          product.catalog_product_id,
          value.catalog_product_id,
        );
        const lastPage = Math.max(1, Math.ceil(value.total / value.page_size));
        if (page > lastPage) {
          setPage(lastPage);
          return;
        }
        setData(value);
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setError(errorMessage(error));
          setIdentityChanged(error instanceof PurchaseIdentityError);
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [
    api,
    token,
    product.product_key,
    product.catalog_product_id,
    page,
    pageSize,
    revision,
    hasPurchase,
    ordersOpen,
  ]);
  const summary = data ? purchaseSummaryValues(data.summary) : null;
  const pages = data ? Math.max(1, Math.ceil(data.total / data.page_size)) : 1;
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent className="purchase-history-dialog">
        <DialogHeader>
          <DialogTitle>购买记录</DialogTitle>
          <DialogDescription>
            <span>{product.name} · </span>
            <span className="technical-text">{product.product_key}</span>
          </DialogDescription>
        </DialogHeader>
        <MembershipOriginFilters origins={origins} onChange={setOrigins} />
        {hasPurchase && (
          <Collapsible
            open={ordersOpen}
            onOpenChange={setOrdersOpen}
            className="purchase-order-details"
          >
            <CollapsibleTrigger asChild>
              <Button variant="secondary" className="purchase-order-toggle">
                支付订单明细
                <ChevronDown />
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="purchase-order-content">
              <>
                <div className="purchase-history-toolbar">
                  <p className="field-hint">
                    统计成功支付的订单，包含已退款记录；金额为原支付金额，未扣除退款。
                  </p>
                  <Button
                    variant="secondary"
                    disabled={loading}
                    onClick={() => setRevision((current) => current + 1)}
                    aria-label="刷新购买记录"
                  >
                    <RefreshCw />
                    刷新
                  </Button>
                </div>
                {loading && (
                  <p className="empty-state" role="status">
                    正在加载购买记录…
                  </p>
                )}
                {error && (
                  <div className="operation-error" role="alert">
                    <span>{error}</span>
                    <Button
                      variant="secondary"
                      onClick={
                        identityChanged
                          ? onReloadProduct
                          : () => setRevision((current) => current + 1)
                      }
                    >
                      {identityChanged ? "返回并刷新商品列表" : "重试购买记录"}
                    </Button>
                  </div>
                )}
                {data && summary && (
                  <div className="purchase-history-content">
                    <dl className="purchase-summary" aria-label="商品购买汇总">
                      <div>
                        <dt>购买人数</dt>
                        <dd>{summary.users}</dd>
                      </div>
                      <div>
                        <dt>支付订单</dt>
                        <dd>{summary.orders}</dd>
                      </div>
                      <div>
                        <dt>退款订单</dt>
                        <dd>{summary.refunded}</dd>
                      </div>
                      <div>
                        <dt>累计支付金额（退款前）</dt>
                        <dd>{summary.amount}</dd>
                      </div>
                    </dl>
                    <Table
                      className="purchase-history-table"
                      aria-label="商品购买订单"
                    >
                      <TableHeader>
                        <TableRow>
                          <TableHead>购买用户</TableHead>
                          <TableHead>购买时间（北京时间）</TableHead>
                          <TableHead>到期时间（北京时间）</TableHead>
                          <TableHead>实付金额</TableHead>
                          <TableHead>订单状态</TableHead>
                          <TableHead>订单号</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {data.items.map((item) => {
                          const expiry = purchaseExpiry(item);
                          return (
                            <TableRow key={item.order_no}>
                              <TableCell>
                                <div className="purchase-user">
                                  <PurchaseAvatar url={item.user_avatar_url} />
                                  <div className="purchase-user-identity">
                                    <strong>
                                      {item.user_nickname || "未命名用户"}
                                    </strong>
                                    <MembershipOriginBadge
                                      origin={item.membership_origin}
                                    />
                                    <small className="technical-text">
                                      {item.user_id}
                                    </small>
                                  </div>
                                </div>
                              </TableCell>
                              <TableCell className="purchase-date-cell">
                                {purchaseDate(item.paid_at)}
                              </TableCell>
                              <TableCell className="purchase-date-cell">
                                <span>{expiry.text}</span>
                                {expiry.hint && (
                                  <span
                                    className="purchase-expiry-hint"
                                    title="历史订单按原一年规则估算，不代表账号当前会员到期时间"
                                  >
                                    {expiry.hint}
                                  </span>
                                )}
                              </TableCell>
                              <TableCell className="amount-cell">
                                {purchaseAmount(item.amount_cents)}
                              </TableCell>
                              <TableCell>
                                <Badge
                                  data-tone={
                                    item.status === "refunded"
                                      ? "warning"
                                      : "success"
                                  }
                                >
                                  {item.status === "refunded"
                                    ? "已退款"
                                    : "已支付"}
                                </Badge>
                              </TableCell>
                              <TableCell className="purchase-order-number technical-text">
                                {item.order_no}
                              </TableCell>
                            </TableRow>
                          );
                        })}
                        {!data.items.length && (
                          <TableRow>
                            <TableCell colSpan={6}>
                              <p className="empty-state">
                                该商品暂无成功支付的购买记录。
                              </p>
                            </TableCell>
                          </TableRow>
                        )}
                      </TableBody>
                    </Table>
                    <div className="product-pagination">
                      <span>
                        共 {data.total} 条 · 第 {data.page} / {pages} 页
                      </span>
                      <div className="pagination-actions">
                        <Label htmlFor="purchase-page-size">每页</Label>
                        <Select
                          value={String(pageSize)}
                          onValueChange={(value) => {
                            setPageSize(Number(value));
                            setPage(1);
                          }}
                        >
                          <SelectTrigger id="purchase-page-size">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {[10, 20, 50, 100].map((size) => (
                              <SelectItem key={size} value={String(size)}>
                                {size} 条
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                        <Button
                          variant="secondary"
                          size="icon"
                          aria-label="购买记录上一页"
                          disabled={loading || page <= 1}
                          onClick={() => setPage((current) => current - 1)}
                        >
                          <ChevronLeft />
                        </Button>
                        <Button
                          variant="secondary"
                          size="icon"
                          aria-label="购买记录下一页"
                          disabled={loading || page >= pages}
                          onClick={() => setPage((current) => current + 1)}
                        >
                          <ChevronRight />
                        </Button>
                      </div>
                    </div>
                  </div>
                )}
              </>
            </CollapsibleContent>
          </Collapsible>
        )}
        <ProductMembers
          key={origins.slice().sort().join(",")}
          api={api}
          token={token}
          product={product}
          origins={origins}
          onReloadProduct={onReloadProduct}
        />
        <DialogFooter>
          <Button variant="secondary" onClick={onClose}>
            关闭购买记录
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
