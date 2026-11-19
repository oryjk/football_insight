import { describe, expect, test } from "bun:test";
import type { AdminPageLayout } from "../../api";
import {
  buildPreviewApplyMessage,
  buildPreviewIframeUrl,
  buildSavePayload,
  clonePageLayout,
  defaultPreviewBaseUrl,
  filterAndGroupPages,
  isLayoutDirty,
  moveBlock,
  reorderBlocks,
  setAllBlocksVisibility,
  setBlockBackground,
  toggleBlockVisibility,
  validateDraft,
  validateImageUrlInput,
  validateUploadImageFile,
} from "./pageLayoutsModel";

function sampleHomeLayout(): AdminPageLayout {
  return {
    page_key: "home",
    title: "首页",
    route_path: "/pages/home/index",
    group: "primary_tab",
    description: "今日话题、主队助力、排行榜",
    background_image_url: null,
    blocks: [
      {
        key: "daily_topics",
        label: "今日话题",
        description: "每日话题精选",
        visible: true,
        background_image_url: null,
      },
      {
        key: "my_team_support",
        label: "我的主队",
        description: "主队赛程与助力",
        visible: true,
        background_image_url: null,
      },
      {
        key: "standings",
        label: "积分榜",
        description: "联赛积分榜",
        visible: true,
        background_image_url: null,
      },
    ],
    customized: false,
    version: 0,
    updated_at: null,
    updated_by: null,
  };
}

describe("pageLayoutsModel", () => {
  test("reorderBlocks and moveBlock update block ordering immutably", () => {
    const layout = sampleHomeLayout();
    const reordered = reorderBlocks(layout.blocks, 1, 0);
    expect(reordered.map((b) => b.key)).toEqual([
      "my_team_support",
      "daily_topics",
      "standings",
    ]);
    expect(layout.blocks.map((b) => b.key)).toEqual([
      "daily_topics",
      "my_team_support",
      "standings",
    ]);

    const movedDown = moveBlock(reordered, 0, 1);
    expect(movedDown.map((b) => b.key)).toEqual([
      "daily_topics",
      "my_team_support",
      "standings",
    ]);
  });

  test("toggleBlockVisibility, setAllBlocksVisibility, and setBlockBackground work cleanly", () => {
    const layout = sampleHomeLayout();
    const hiddenAll = setAllBlocksVisibility(layout.blocks, false);
    expect(hiddenAll.every((b) => !b.visible)).toBe(true);

    const toggled = toggleBlockVisibility(hiddenAll, "my_team_support", true);
    expect(toggled.find((b) => b.key === "my_team_support")?.visible).toBe(true);

    const withBg = setBlockBackground(
      toggled,
      "my_team_support",
      "https://cdn.example.com/card.png",
    );
    expect(
      withBg.find((b) => b.key === "my_team_support")?.background_image_url,
    ).toBe("https://cdn.example.com/card.png");
  });

  test("validates image URLs, upload files, and full drafts", () => {
    expect(validateImageUrlInput("")).toBeNull();
    expect(validateImageUrlInput("https://cdn.example.com/bg.webp")).toBeNull();
    expect(validateImageUrlInput("javascript:alert(1)")).not.toBeNull();
    expect(validateImageUrlInput("https://cdn.example.com/a b.png")).not.toBeNull();

    expect(
      validateUploadImageFile({ type: "image/png", size: 1024 }),
    ).toBeNull();
    expect(
      validateUploadImageFile({ type: "image/svg+xml", size: 1024 }),
    ).not.toBeNull();
    expect(
      validateUploadImageFile({ type: "image/png", size: 6 * 1024 * 1024 }),
    ).not.toBeNull();

    const draft = sampleHomeLayout();
    draft.background_image_url = "ftp://invalid.example.com/a.png";
    expect(validateDraft(draft)).toContain("页面背景图无效");
  });

  test("tracks dirty state and builds save and H5 preview messages", () => {
    const saved = sampleHomeLayout();
    const draft = clonePageLayout(saved);
    expect(isLayoutDirty(saved, draft)).toBe(false);

    draft.background_image_url = "  https://cdn.example.com/home.jpg  ";
    draft.blocks = reorderBlocks(draft.blocks, 1, 0);
    draft.blocks = toggleBlockVisibility(draft.blocks, "standings", false);
    draft.blocks = setBlockBackground(
      draft.blocks,
      "my_team_support",
      "https://cdn.example.com/team-bg.png",
    );

    expect(isLayoutDirty(saved, draft)).toBe(true);

    const payload = buildSavePayload(draft);
    expect(payload).toEqual({
      background_image_url: "https://cdn.example.com/home.jpg",
      blocks: [
        {
          key: "my_team_support",
          visible: true,
          background_image_url: "https://cdn.example.com/team-bg.png",
        },
        { key: "daily_topics", visible: true, background_image_url: null },
        { key: "standings", visible: false, background_image_url: null },
      ],
    });

    const msg = buildPreviewApplyMessage(draft);
    expect(msg.type).toBe("fi-layout-preview:apply");
    expect(msg.pageKey).toBe("home");
    expect(msg.layout.background_image_url).toBe(
      "https://cdn.example.com/home.jpg",
    );
    expect(msg.layout.blocks[0]?.key).toBe("my_team_support");
  });

  test("filters, groups pages, and builds H5 preview iframe URLs", () => {
    const pages: AdminPageLayout[] = [
      sampleHomeLayout(),
      {
        ...sampleHomeLayout(),
        page_key: "topic_detail",
        title: "话题详情",
        route_path: "/pages/topic-detail/index",
        group: "detail",
        description: "每日话题正文与音频",
      },
    ];
    expect(filterAndGroupPages(pages, "")).toHaveLength(2);
    expect(filterAndGroupPages(pages, "topic")).toHaveLength(1);
    expect(defaultPreviewBaseUrl("", "https://match.oryjk.cn")).toBe(
      "https://match.oryjk.cn/football",
    );
    expect(
      defaultPreviewBaseUrl("", "http://172.16.60.233", "/football-admin/"),
    ).toBe("http://172.16.60.233/football-admin/preview");
    expect(
      buildPreviewIframeUrl(
        "https://match.oryjk.cn/football/",
        "/pages/home/index",
      ),
    ).toBe("https://match.oryjk.cn/football/#/pages/home/index?layout_preview=1");
  });
});
