import type {
  AdminPageLayout,
  AdminPageLayoutBlock,
  PageLayoutGroup,
  SavePageLayoutPayload,
} from "../../api";

export const PAGE_GROUP_ORDER: readonly PageLayoutGroup[] = [
  "primary_tab",
  "feature",
  "detail",
];

export const PAGE_GROUP_LABELS: Record<PageLayoutGroup, string> = {
  primary_tab: "主导航页",
  feature: "功能页",
  detail: "详情页",
};

export const MAX_LAYOUT_IMAGE_BYTES = 5 * 1024 * 1024;
export const MAX_LAYOUT_IMAGE_URL_LEN = 1024;
export const PREVIEW_BASE_URL_STORAGE_KEY =
  "fi_admin_layout_preview_base_url";

export const ALLOWED_IMAGE_MIME_TYPES = new Set([
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/gif",
]);

export const WECHAT_ONLY_PREVIEW_PAGES = new Set([
  "lineup",
  "lineup_view",
  "lineup_list",
]);

export function normalizeImageUrlInput(
  value: string | null | undefined,
): string | null {
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

export function validateImageUrlInput(
  value: string | null | undefined,
): string | null {
  const normalized = normalizeImageUrlInput(value);
  if (!normalized) return null;
  if (normalized.length > MAX_LAYOUT_IMAGE_URL_LEN) {
    return `图片地址不能超过 ${MAX_LAYOUT_IMAGE_URL_LEN} 个字符`;
  }
  if (!/^https?:\/\/[^\s]+$/i.test(normalized)) {
    return "图片地址须以 http:// 或 https:// 开头，且不能包含空格";
  }
  return null;
}

export function validateUploadImageFile(file: {
  type: string;
  size: number;
}): string | null {
  if (!ALLOWED_IMAGE_MIME_TYPES.has(file.type)) {
    return "仅支持 PNG、JPG、WebP 或 GIF 图片";
  }
  if (file.size <= 0) {
    return "图片内容为空，请重新选择";
  }
  if (file.size > MAX_LAYOUT_IMAGE_BYTES) {
    return "图片大小不能超过 5MB";
  }
  return null;
}

export function clonePageLayout(layout: AdminPageLayout): AdminPageLayout {
  return {
    ...layout,
    blocks: layout.blocks.map((block) => ({ ...block })),
  };
}

export function moveBlock(
  blocks: readonly AdminPageLayoutBlock[],
  index: number,
  delta: -1 | 1,
): AdminPageLayoutBlock[] {
  const target = index + delta;
  if (index < 0 || index >= blocks.length || target < 0 || target >= blocks.length) {
    return blocks.map((b) => ({ ...b }));
  }
  return reorderBlocks(blocks, index, target);
}

export function reorderBlocks(
  blocks: readonly AdminPageLayoutBlock[],
  fromIndex: number,
  toIndex: number,
): AdminPageLayoutBlock[] {
  if (
    fromIndex < 0 ||
    fromIndex >= blocks.length ||
    toIndex < 0 ||
    toIndex >= blocks.length ||
    fromIndex === toIndex
  ) {
    return blocks.map((b) => ({ ...b }));
  }
  const next = blocks.map((b) => ({ ...b }));
  const [moved] = next.splice(fromIndex, 1);
  if (!moved) return next;
  next.splice(toIndex, 0, moved);
  return next;
}

export function toggleBlockVisibility(
  blocks: readonly AdminPageLayoutBlock[],
  key: string,
  visible: boolean,
): AdminPageLayoutBlock[] {
  return blocks.map((block) =>
    block.key === key ? { ...block, visible } : { ...block },
  );
}

export function setAllBlocksVisibility(
  blocks: readonly AdminPageLayoutBlock[],
  visible: boolean,
): AdminPageLayoutBlock[] {
  return blocks.map((block) => ({ ...block, visible }));
}

export function setBlockBackground(
  blocks: readonly AdminPageLayoutBlock[],
  key: string,
  backgroundImageUrl: string | null,
): AdminPageLayoutBlock[] {
  return blocks.map((block) =>
    block.key === key
      ? { ...block, background_image_url: backgroundImageUrl }
      : { ...block },
  );
}

export function buildSavePayload(
  draft: AdminPageLayout,
): SavePageLayoutPayload {
  return {
    background_image_url: normalizeImageUrlInput(draft.background_image_url),
    blocks: draft.blocks.map((block) => ({
      key: block.key,
      visible: block.visible,
      background_image_url: normalizeImageUrlInput(block.background_image_url),
    })),
  };
}

export function validateDraft(draft: AdminPageLayout): string | null {
  const pageBgErr = validateImageUrlInput(draft.background_image_url);
  if (pageBgErr) {
    return `页面背景图无效：${pageBgErr}`;
  }
  for (const block of draft.blocks) {
    const blockBgErr = validateImageUrlInput(block.background_image_url);
    if (blockBgErr) {
      return `卡片「${block.label}」背景图无效：${blockBgErr}`;
    }
  }
  return null;
}

export function isLayoutDirty(
  saved: AdminPageLayout | null,
  draft: AdminPageLayout | null,
): boolean {
  if (!saved || !draft) return false;
  if (saved.page_key !== draft.page_key) return false;
  if (
    normalizeImageUrlInput(saved.background_image_url) !==
    normalizeImageUrlInput(draft.background_image_url)
  ) {
    return true;
  }
  if (saved.blocks.length !== draft.blocks.length) return true;
  for (let i = 0; i < saved.blocks.length; i += 1) {
    const a = saved.blocks[i];
    const b = draft.blocks[i];
    if (!a || !b) return true;
    if (
      a.key !== b.key ||
      a.visible !== b.visible ||
      normalizeImageUrlInput(a.background_image_url) !==
        normalizeImageUrlInput(b.background_image_url)
    ) {
      return true;
    }
  }
  return false;
}

export function filterAndGroupPages(
  pages: readonly AdminPageLayout[],
  keyword: string,
): Array<{ group: PageLayoutGroup; label: string; items: AdminPageLayout[] }> {
  const normalized = keyword.trim().toLowerCase();
  const matched = normalized
    ? pages.filter(
        (item) =>
          item.title.toLowerCase().includes(normalized) ||
          item.page_key.toLowerCase().includes(normalized) ||
          item.route_path.toLowerCase().includes(normalized) ||
          item.description.toLowerCase().includes(normalized),
      )
    : [...pages];

  return PAGE_GROUP_ORDER.map((group) => ({
    group,
    label: PAGE_GROUP_LABELS[group],
    items: matched.filter((item) => item.group === group),
  })).filter((section) => section.items.length > 0);
}

export function defaultPreviewBaseUrl(
  configuredUrl?: string,
  currentOrigin?: string,
): string {
  const explicit = configuredUrl?.trim();
  if (explicit) return explicit.replace(/\/+$/, "");
  if (currentOrigin && /match\.oryjk\.cn/i.test(currentOrigin)) {
    return `${currentOrigin.replace(/\/+$/, "")}/football`;
  }
  return "https://match.oryjk.cn/football";
}

export function buildPreviewIframeUrl(
  baseUrl: string,
  routePath: string,
): string {
  const cleanedBase = baseUrl.trim().replace(/\/+$/, "");
  if (!cleanedBase) return "";
  const normalizedRoute = routePath.startsWith("/")
    ? routePath
    : `/${routePath}`;
  return `${cleanedBase}/#${normalizedRoute}?layout_preview=1`;
}

export interface PreviewApplyMessage {
  type: "fi-layout-preview:apply";
  pageKey: string;
  layout: {
    page_key: string;
    background_image_url: string | null;
    blocks: Array<{
      key: string;
      visible: boolean;
      background_image_url: string | null;
    }>;
    version: number;
    updated_at: string | null;
  };
}

export interface PreviewNavigateMessage {
  type: "fi-layout-preview:navigate";
  routePath: string;
}

export function buildPreviewApplyMessage(
  draft: AdminPageLayout,
): PreviewApplyMessage {
  return {
    type: "fi-layout-preview:apply",
    pageKey: draft.page_key,
    layout: {
      page_key: draft.page_key,
      background_image_url:
        validateImageUrlInput(draft.background_image_url) === null
          ? normalizeImageUrlInput(draft.background_image_url)
          : null,
      blocks: draft.blocks.map((block) => ({
        key: block.key,
        visible: block.visible,
        background_image_url:
          validateImageUrlInput(block.background_image_url) === null
            ? normalizeImageUrlInput(block.background_image_url)
            : null,
      })),
      version: draft.version,
      updated_at: draft.updated_at,
    },
  };
}

export function buildPreviewNavigateMessage(
  routePath: string,
): PreviewNavigateMessage {
  return {
    type: "fi-layout-preview:navigate",
    routePath,
  };
}
