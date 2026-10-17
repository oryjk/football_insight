import type { TopicReader } from "../../api";

export function readerName(reader: Pick<TopicReader, "display_name">): string {
  const name = reader.display_name?.trim();
  return name ? name : "未设置昵称";
}

const beijingClock = new Intl.DateTimeFormat("en-CA", {
  timeZone: "Asia/Shanghai",
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  hourCycle: "h23",
});

// Beijing time to the minute; `first_read_at` is the first open, not the latest.
export function firstReadTime(value: string): string {
  const date = new Date(value);
  if (!value || Number.isNaN(date.getTime())) return "时间异常";
  const parts: Record<string, string> = {};
  for (const part of beijingClock.formatToParts(date))
    parts[part.type] = part.value;
  return `${parts.year}-${parts.month}-${parts.day} ${parts.hour}:${parts.minute}`;
}

export function topicOptionLabel(topic: {
  title: string;
  edition_date: string;
  reader_count: number;
}): string {
  const title = topic.title.trim() || "（无标题）";
  return `${topic.edition_date} · ${title}（${topic.reader_count} 人）`;
}
