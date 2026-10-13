import type { AdminApi } from "../../api";
import { PurchaseAccountChooser } from "./PurchaseAccountChooser";
import { setProductTier, setPurchaseScope } from "./purchaseScopeModel";
import { Input } from "../../components/ui/input";
import { Textarea } from "../../components/ui/textarea";
import { Label } from "../../components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { Switch } from "../../components/ui/switch";
import { membershipTiers, type ProductForm } from "./productsModel";
export function ProductFields({
  value,
  api,
  token,
  onChange,
  disabled,
  fieldError,
}: {
  value: ProductForm;
  api: AdminApi;
  token: string;
  onChange: (value: ProductForm) => void;
  disabled: boolean;
  fieldError: { field: string; message: string } | null;
}) {
  const change = (key: keyof ProductForm, next: string | boolean) =>
    onChange({ ...value, [key]: next });
  const error = (field: string) =>
    fieldError?.field === field ? (
      <p id={`${field}-error`} role="alert" className="field-error">
        {fieldError.message}
      </p>
    ) : null;
  const attrs = (field: string) => ({
    "aria-invalid": fieldError?.field === field,
    "aria-describedby":
      fieldError?.field === field ? `${field}-error` : undefined,
  });
  return (
    <div className="product-form-grid">
      <div className="form-field">
        <Label htmlFor="product-name">商品名称</Label>
        <Input
          id="product-name"
          value={value.name}
          onChange={(e) => change("name", e.target.value)}
          disabled={disabled}
          {...attrs("name")}
        />
        {error("name")}
      </div>
      <div className="form-field">
        <Label htmlFor="product-tier">会员等级</Label>
        <Select
          value={value.membership_tier}
          onValueChange={(v) => onChange(setProductTier(value, v))}
          disabled={disabled}
        >
          <SelectTrigger id="product-tier" {...attrs("membership_tier")}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {membershipTiers.map((t) => (
              <SelectItem key={t} value={t}>
                {t}
                {t === "V3" ? " · 测试商品" : ""}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {error("membership_tier")}
      </div>
      <div className="form-field form-wide">
        <Label htmlFor="product-id">微信道具 ID</Label>
        <Input
          id="product-id"
          value={value.product_id}
          onChange={(e) => change("product_id", e.target.value)}
          disabled={disabled}
          {...attrs("product_id")}
        />
        {error("product_id")}
      </div>
      <div className="form-field">
        <Label htmlFor="product-amount">道具金额（元）</Label>
        <Input
          id="product-amount"
          inputMode="decimal"
          placeholder="例如 39.00"
          value={value.amount}
          onChange={(e) => change("amount", e.target.value)}
          disabled={disabled}
          {...attrs("amount")}
        />
        <p className="field-hint">
          与微信已发布道具金额一致；不修改公开会员标价。
        </p>
        {error("amount")}
      </div>
      <div className="form-field">
        <Label htmlFor="product-duration">会员有效期（月）</Label>
        <Input
          id="product-duration"
          inputMode="numeric"
          value={value.durationMonths}
          onChange={(e) => change("durationMonths", e.target.value)}
          disabled={disabled}
          {...attrs("durationMonths")}
        />
        <p className="field-hint">填写 1–1200 的整数，新建默认为 12 月。</p>
        {error("durationMonths")}
      </div>
      <div className="form-field">
        <Label htmlFor="product-enabled">启用状态</Label>
        <div className="switch-field">
          <Switch
            id="product-enabled"
            checked={value.enabled}
            onCheckedChange={(v) => change("enabled", v)}
            disabled={disabled}
          />
          <span>{value.enabled ? "已启用" : "已停用"}</span>
        </div>
      </div>
      <div className="form-field form-wide">
        <Label htmlFor="purchase-scope">购买范围</Label>
        <Select
          value={value.purchaseScope}
          onValueChange={(scope) =>
            onChange(setPurchaseScope(value, scope as "public" | "specified"))
          }
          disabled={disabled}
        >
          <SelectTrigger id="purchase-scope" {...attrs("purchaseScope")}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              value="public"
              disabled={value.membership_tier === "V3"}
            >
              公开购买
            </SelectItem>
            <SelectItem value="specified">指定账号</SelectItem>
          </SelectContent>
        </Select>
        <p className="field-hint">
          {value.membership_tier === "V3"
            ? "V3 测试商品仅允许指定账号购买。"
            : value.purchaseScope === "public"
              ? "公开商品允许符合会员购买条件的账号购买。"
              : "仅下方指定账号可购买，至少选择一个账号。"}
        </p>
        {error("purchaseScope")}
        {value.purchaseScope === "specified" && (
          <PurchaseAccountChooser
            api={api}
            token={token}
            value={value}
            onChange={onChange}
            disabled={disabled}
            validationError={
              fieldError?.field === "allowedUsers"
                ? fieldError.message
                : undefined
            }
          />
        )}
      </div>
      <div className="form-field form-wide">
        <Label htmlFor="product-remark">备注</Label>
        <Textarea
          id="product-remark"
          rows={3}
          value={value.remark}
          onChange={(e) => change("remark", e.target.value)}
          disabled={disabled}
          {...attrs("remark")}
        />
        {error("remark")}
      </div>
    </div>
  );
}
