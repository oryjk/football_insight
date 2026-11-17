import type { AdminTelegraphCommentStatus } from "../../api";
import { firstReadTime } from "./topicReadersModel";

export function commentCreatedLabel(value: string): string {
  return firstReadTime(value);
}

export function commentStatusLabel(status: AdminTelegraphCommentStatus): string {
  switch (status) {
    case "visible":
      return "展示中";
    case "hidden":
      return "已隐藏";
    case "deleted":
      return "已删除";
  }
}

export function commentEntryLabel(title: string, entryId: string): string {
  const normalized = title.trim();
  if (normalized) return normalized;
  return `条目 ${entryId.slice(0, 8)}`;
}
