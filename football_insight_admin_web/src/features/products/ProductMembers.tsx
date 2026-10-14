import { useEffect, useState } from "react";
import { ChevronLeft, ChevronRight, RefreshCw } from "lucide-react";
import type {
  AdminApi,
  Product,
  ProductMemberPage,
  MembershipOrigin,
} from "../../api";
import { errorMessage } from "../../apiClient";
import { Button } from "../../components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/table";
import { Badge } from "../../components/ui/badge";
import { MembershipOriginBadge } from "./MembershipOriginBadge";
import { PurchaseAvatar } from "./ProductUserAvatar";
import {
  assertPurchaseIdentity,
  PurchaseIdentityError,
  purchaseDate,
} from "./purchasesModel";

export function ProductMembers({
  api,
  token,
  product,
  onReloadProduct,
  origins,
}: {
  api: AdminApi;
  token: string;
  product: Pick<Product, "product_key" | "name" | "catalog_product_id">;
  onReloadProduct: () => void;
  origins: MembershipOrigin[];
}) {
  const [page, setPage] = useState(1);
  const [revision, setRevision] = useState(0);
  const [data, setData] = useState<ProductMemberPage | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [identityChanged, setIdentityChanged] = useState(false);
  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setData(null);
    setError("");
    setIdentityChanged(false);
    if (!origins.length) {
      setLoading(false);
      return () => controller.abort();
    }
    api
      .productMembers(
        token,
        product.product_key,
        page,
        controller.signal,
        origins,
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
    revision,
    origins,
  ]);
  const pages = data ? Math.max(1, Math.ceil(data.total / data.page_size)) : 1;
  return (
    <div className="purchase-history-content">
      <h3>会员列表</h3>
      <div className="purchase-history-toolbar">
        <p className="field-hint">
          显示该等级的会员账号（含已到期），不代表购买了当前道具。来源依据当前会员授予记录；无法确认的显示未知来源。
        </p>
        <Button
          variant="secondary"
          disabled={loading}
          aria-label="刷新同等级会员"
          onClick={() => setRevision((value) => value + 1)}
        >
          <RefreshCw />
          刷新
        </Button>
      </div>
      {!origins.length && (
        <p className="empty-state">请至少选择一种会员来源。</p>
      )}
      {loading && (
        <p role="status" className="empty-state">
          正在加载会员…
        </p>
      )}
      {error && (
        <div role="alert" className="operation-error">
          <span>{error}</span>
          <Button
            variant="secondary"
            onClick={
              identityChanged
                ? onReloadProduct
                : () => setRevision((value) => value + 1)
            }
          >
            {identityChanged ? "返回并刷新商品列表" : "重试会员记录"}
          </Button>
        </div>
      )}
      {data && (
        <>
          <p className="field-hint">
            当前筛选的 {data.membership_tier} 会员共 {data.total}{" "}
            人；推荐升级及未知来源账号不计入购买人数和订单金额。
          </p>
          <Table className="purchase-members-table" aria-label="同等级会员">
            <TableHeader>
              <TableRow>
                <TableHead>会员用户 / 来源</TableHead>
                <TableHead>当前到期时间（北京时间）</TableHead>
                <TableHead>会员状态</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.items.map((item) => (
                <TableRow key={item.user_id}>
                  <TableCell>
                    <div className="purchase-user">
                      <PurchaseAvatar url={item.user_avatar_url} />
                      <div className="purchase-user-identity">
                        <strong>{item.user_nickname || "未命名用户"}</strong>
                        <MembershipOriginBadge
                          origin={item.membership_origin}
                        />
                        <small className="technical-text">{item.user_id}</small>
                      </div>
                    </div>
                  </TableCell>
                  <TableCell className="purchase-date-cell">
                    {item.expires_at === null
                      ? "长期有效"
                      : purchaseDate(item.expires_at)}
                  </TableCell>
                  <TableCell>
                    <Badge data-tone={item.expired ? "warning" : "success"}>
                      {item.expired ? "已到期" : "有效"}
                    </Badge>
                  </TableCell>
                </TableRow>
              ))}
              {!data.items.length && (
                <TableRow>
                  <TableCell colSpan={3}>
                    <p className="empty-state">当前来源筛选下暂无会员。</p>
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
          <div className="product-pagination">
            <span>
              共 {data.total} 人 · 第 {data.page} / {pages} 页
            </span>
            <div className="pagination-actions">
              <Button
                variant="secondary"
                size="icon"
                disabled={loading || page <= 1}
                aria-label="同等级会员上一页"
                onClick={() => setPage((value) => value - 1)}
              >
                <ChevronLeft />
              </Button>
              <Button
                variant="secondary"
                size="icon"
                disabled={loading || page >= pages}
                aria-label="同等级会员下一页"
                onClick={() => setPage((value) => value + 1)}
              >
                <ChevronRight />
              </Button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
