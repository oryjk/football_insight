export const STUDENT_SERVICES = [
  { code: "certificate_renewal", title: "证书续期", price_cents: 9900, unit: "年" },
  { code: "tool_upgrade", title: "工具升级", price_cents: 69900, unit: "套" },
] as const;
export type StudentServiceCode = (typeof STUDENT_SERVICES)[number]["code"];
export interface StudentServiceConfig {
  code: StudentServiceCode;
  title: string;
  description: string;
  price_cents: number;
  unit: string;
  enabled: boolean;
  product_id: string;
}
export interface StudentConfig {
  contact_qr_url: string;
  services: StudentServiceConfig[];
}
export class StudentConfigError extends Error {
  constructor(public field: string, message: string) {
    super(message);
    this.name = "StudentConfigError";
  }
}
export function parseStudentConfig(value: unknown): StudentConfig {
  const invalid = () => { throw new Error("学员功能配置返回异常，请重新加载"); };
  if (!value || typeof value !== "object") return invalid();
  const data = value as Record<string, unknown>;
  if (typeof data.contact_qr_url !== "string" || !Array.isArray(data.services) || data.services.length !== 2) return invalid();
  const services = STUDENT_SERVICES.map((fixed) => {
    const matches = (data.services as unknown[]).filter((item) => item && typeof item === "object" && (item as Record<string, unknown>).code === fixed.code);
    if (matches.length !== 1) return invalid();
    const item = matches[0] as Record<string, unknown>;
    if (item.title !== fixed.title || item.price_cents !== fixed.price_cents || item.unit !== fixed.unit || typeof item.description !== "string" || typeof item.product_id !== "string" || typeof item.enabled !== "boolean") return invalid();
    return { ...fixed, description: item.description, product_id: item.product_id, enabled: item.enabled };
  });
  return { contact_qr_url: data.contact_qr_url, services };
}
export function qrUrlError(input: string): string | null {
  const url = input.trim();
  if (!url) return null;
  if ([...url].length > 2048) return "二维码链接最多 2048 个字符";
  try {
    const parsed = new URL(url);
    if (parsed.protocol === "https:" && parsed.hostname && !parsed.username && !parsed.password && !/\s/u.test(url)) return null;
  } catch { /* Report the same actionable validation error. */ }
  return "二维码须使用不含账号密码的 HTTPS 链接";
}
export function studentConfigPayload(form: StudentConfig): StudentConfig {
  const parsed = parseStudentConfig(form);
  const contact_qr_url = parsed.contact_qr_url.trim();
  const qrError = qrUrlError(contact_qr_url);
  if (qrError) throw new StudentConfigError("contact_qr_url", qrError);
  const services = parsed.services.map((item) => {
    const description = item.description.trim();
    const product_id = item.product_id.trim();
    if ([...description].length > 4000) throw new StudentConfigError(`${item.code}.description`, `${item.title}说明最多 4000 个字符`);
    if ([...product_id].length > 200) throw new StudentConfigError(`${item.code}.product_id`, `${item.title}商品 ID 最多 200 个字符`);
    if (/\s/u.test(product_id)) throw new StudentConfigError(`${item.code}.product_id`, `${item.title}商品 ID 不能包含空白字符`);
    if (item.enabled) {
      if (!description) throw new StudentConfigError(`${item.code}.description`, `启用${item.title}前请填写服务说明`);
      if (!product_id) throw new StudentConfigError(`${item.code}.product_id`, `启用${item.title}前请填写已发布微信商品 ID`);
      if (!contact_qr_url) throw new StudentConfigError("contact_qr_url", `启用${item.title}前请填写联系二维码链接`);
    }
    return { ...item, description, product_id };
  });
  return { contact_qr_url, services };
}
