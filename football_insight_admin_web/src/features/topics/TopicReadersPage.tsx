import { useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";
import type { AdminApi, TopicReader, TopicReaderTopic } from "../../api";
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
import { PurchaseAvatar } from "../products/ProductUserAvatar";
import {
  firstReadTime,
  readerName,
  topicOptionLabel,
} from "./topicReadersModel";

export function TopicReadersPage({
  api,
  token,
}: {
  api: AdminApi;
  token: string;
}) {
  const [topics, setTopics] = useState<TopicReaderTopic[]>([]);
  const [selected, setSelected] = useState("");
  const [readers, setReaders] = useState<TopicReader[] | null>(null);
  const [revision, setRevision] = useState(0);
  const [loadingTopics, setLoadingTopics] = useState(true);
  const [loadingReaders, setLoadingReaders] = useState(false);
  const [error, setError] = useState("");
  const topic = topics.find((item) => `${item.team_id}/${item.topic_id}` === selected);

  useEffect(() => {
    const controller = new AbortController();
    setLoadingTopics(true);
    setError("");
    api
      .topicReaderTopics(token, controller.signal)
      .then((value) => {
        if (controller.signal.aborted) return;
        setTopics(value.items);
        setSelected((current) =>
          value.items.some((item) => `${item.team_id}/${item.topic_id}` === current)
            ? current
            : value.items[0]
              ? `${value.items[0].team_id}/${value.items[0].topic_id}`
              : "",
        );
      })
      .catch((reason) => {
        if (!controller.signal.aborted) setError(errorMessage(reason));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoadingTopics(false);
      });
    return () => controller.abort();
  }, [api, token, revision]);

  useEffect(() => {
    setReaders(null);
    if (!topic) return;
    const controller = new AbortController();
    setLoadingReaders(true);
    setError("");
    api
      .topicReaders(token, topic.team_id, topic.topic_id, controller.signal)
      .then((value) => {
        if (!controller.signal.aborted) setReaders(value.items);
      })
      .catch((reason) => {
        if (!controller.signal.aborted) setError(errorMessage(reason));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoadingReaders(false);
      });
    return () => controller.abort();
  }, [api, token, topic?.team_id, topic?.topic_id, revision]);

  return (
    <div className="topic-readers">
      <div className="topic-readers-toolbar">
        <select
          className="topic-readers-select"
          aria-label="选择话题"
          value={selected}
          disabled={loadingTopics || !topics.length}
          onChange={(event) => setSelected(event.target.value)}
        >
          {topics.map((item) => (
            <option key={`${item.team_id}/${item.topic_id}`} value={`${item.team_id}/${item.topic_id}`}>
              {topicOptionLabel(item)}
            </option>
          ))}
        </select>
        <Button
          variant="secondary"
          disabled={loadingTopics || loadingReaders}
          aria-label="刷新"
          onClick={() => setRevision((value) => value + 1)}
        >
          <RefreshCw />
          刷新
        </Button>
      </div>
      <p className="field-hint">
        只统计已登录用户打开话题详情的记录，同一用户只出现一次，时间为首次阅读（北京时间）。未登录的访客和此功能上线前的阅读不在其中。
      </p>
      {error && (
        <p role="alert" className="operation-error">
          {error}
        </p>
      )}
      {(loadingTopics || loadingReaders) && (
        <p role="status" className="empty-state">
          正在加载…
        </p>
      )}
      {!loadingTopics && !error && !topics.length && (
        <p className="empty-state">暂无话题。</p>
      )}
      {readers && !readers.length && !loadingReaders && (
        <p className="empty-state">这个话题还没有登录用户阅读记录。</p>
      )}
      {readers && readers.length > 0 && (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>用户</TableHead>
              <TableHead>首次阅读</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {readers.map((reader) => (
              <TableRow key={reader.user_id}>
                <TableCell>
                  <span className="topic-reader-user">
                    <PurchaseAvatar url={reader.avatar_url} />
                    {readerName(reader)}
                  </span>
                </TableCell>
                <TableCell>{firstReadTime(reader.first_read_at)}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </div>
  );
}
