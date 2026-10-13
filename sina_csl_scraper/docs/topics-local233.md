# local233 今日话题每日更新与手动运行

此链路使用 Kimi Code `k3-256k` 整理真实新闻全文，用本机 Qwen Eric 四川话音色生成每篇独立音频，复用头像/球队 logo 的 `MinioAssetTarget` 上传，再通过 Python 原子 upsert 发布批次。Rust 只读数据库，小程序直接播放 HTTPS MinIO URL。

## 已安装组件

- CPU：Threadripper 3970X，32 核 / 64 线程；测量时约 233 GiB 可用内存。
- 模型：`data/models/Qwen3-TTS-12Hz-0.6B-CustomVoice`。
- 固定快照：`85e237c12c027371202489a0ec509ded67b5e4b5`，13 个文件共 2,498,388,392 字节，两份 safetensors 已按远端 LFS SHA256 校验。模型目录 `.football-snapshot.json` 保存安装校验信息。
- Runner：`data/native-qwen/bin/football-qwen-tts`，从仓库 `tools/qwen-native` 构建，CPU / f32，Eric + `sichuan_dialect`。安装器校验 any-tts 0.2.0 归档及 0.6B 补丁，不修改共享 Cargo 缓存。
- AAC：已安装 `ffmpeg` / `ffprobe`。先生成 24kHz 单声道 PCM16，检查非静音和削波，再转换 AAC/m4a。

官方模型与 Eric 音色说明：[Qwen 固定模型快照](https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/blob/85e237c12c027371202489a0ec509ded67b5e4b5/README.md)。这不是拿 Mac Metal 的速度推算 CPU。

## 每日自动更新

local233 已配置 betalpha 用户 crontab：每天北京时间 **07:00–22:30，每半小时** 执行 `./update_daily_topics.sh --scheduled`。系统时区已确认是 Asia/Shanghai，cron 服务已运行；既有比赛和头像任务保留。模板为 `deploy/topics-daily.cron.example`。07:00 为首次触发，22:30 为最后一次触发；23:00 截止自动执行。正文、CPU 语音和上传全部成功后才发布。

自动来源配置为 `config/rongcheng-news-auto.json`：从[体坛周报中国足球频道](https://www.ttplus.cn/innerfb.html)和首页发现蓉城文章，并读取[中新网体育 RSS](https://www.chinanews.com.cn/rss/sports.xml)。列表只用于找原文地址，发布材料必须再读取完整原文与真实发布时间；不把列表摘要当正文。默认采集最近 72 小时，最多生成 5 篇，不足不补造。

每日入口 `topics daily-update` 读取数据库已发布来源历史与最新来源时间。相同 URL、早于或等于最新已发布来源时间的旧新闻不重新生成，也不改发布日期；扩充采集范围不会让较旧报道覆盖当前内容。

- 有新材料：Kimi 整理正文，Qwen CPU 逐篇合成 AAC，MinIO 上传成功，所有文章音频都为 ready 后才原子发布。
- 无新材料或没有可发布话题：正常结束，数据库内容、文章 ID、发布时间和音频 URL 保留。
- 采集源/正文全部不可用、生成、语音、上传或数据库写入失败：记录失败，保留原已发布内容。部分来源失败可继续使用其它已验证原文。
- 未完成批次记入 `data/topics/daily-state.json`；重试复用正文和已完成音频，即使跨天也不重新请求已生成正文。原材料超过 72 小时则放弃该待处理批次，等待新新闻。

运行路径仍为 `/home/betalpha/projects/football_insight/sina_csl_scraper`。锁防止并发执行，整体任务最长 6 小时，自动任务同时以当天 23:00 为截止时间。截止中断时保留待处理材料与已完成缓存，下一工作时段重试。凭据自动加载既有 `.env.topics` / `.env.sync`，数据库连接复用当前本地后端的生产库隧道；不自动建表或执行迁移。

手动执行同一个每日任务：

```bash
./update_daily_topics.sh
cat data/topics/daily-status.json
tail -n 60 data/topics/daily.log
```

日志、状态、缓存和原文均为 Git 忽略的运行产物。CLI 只在持有生成锁时更新共享状态；锁冲突不会覆盖正在执行任务的状态。若数据库已成功提交但本地状态保存失败，会如实记录已发布，不声称旧数据库内容仍保留。

2026-10-13 实测自动来源取得 6 篇完整原文，均不比已发布来源更新，因此结果为 `no_new`；线上两篇正文、ID、更新时间及 ready 音频保持一致。生成/上传/写库失败、部分音频失败、跨天重试、过期待处理材料、原文不可用与并发状态均有回归验证。

## 配置与手动入口

在 `/home/betalpha/projects/football_insight/sina_csl_scraper` 执行。私有 `.env.topics` 包含 Kimi Code 凭据、模型路径和 CPU 参数；`.env.sync` 包含服务器已有 MinIO 凭据。两者均为 0600、Git 忽略，不输出、不提交。

`scripts/run_topics_local.py` 自动用 uv 加载这两个文件，直接使用本机 MinIO 上传器，清除 SSH 上传设置。只有显式 `--write-db` 时才安全读取 `football-insight-local-test-backend` 的 DATABASE_URL 到进程环境。该容器的 `host.docker.internal:55434` 在宿主机映射到实际监听的 `172.17.0.1:55434`；不是 `127.0.0.1:55434`。数据库仍为 peiqian 的共享 `football_data`，不自动 DDL、不再迁移。

```bash
uv run --no-sync python scripts/run_topics_local.py collect \
  --config config/rongcheng-news.json --output data/topics/materials.json

uv run --no-sync python scripts/run_topics_local.py generate \
  --config config/rongcheng-news.json --upload-audio --write-db
```

上述第二条会在材料/日期/模型改变时调用 Kimi，最多生成 5 篇，不强凑数。空结果不替换有效批次。`rongcheng-news.json` 只是一批已核验的原文清单，后续需更新原文地址或配置有效 RSS；默认 72 小时，不是持续发现新文章的订阅。仍须区分一线队中超和希拉谷中乙。

复用已生成的真实两篇文章，仅刷新本地 CPU 语音，不再调用 LLM：

```bash
uv run --no-sync python scripts/run_topics_local.py refresh-audio \
  --edition-file data/topics/imports/2026-10-13/2b36e149bf1ed7567bbefae8c619ea523be539689892a3eb79f996e04db9e7db.json \
  --output-dir data/topics/cpu-validation --write-db
```

该命令复制原批次，只改变音频信息。每篇语音全部生成、校验并上传成功后，才做一次原批次 upsert；任何部分失败都保留原数据库有效音频。正文、来源、发布时间和稳定 id 不变。相同文本/设备/精度/模型路径的音频复用 SHA 校验后的本地缓存。

上传对象名固定为 `summary/audio/topics/{team_id}/{date}/{article_id}/{audio_sha}.m4a`，Content-Type 为 `audio/mp4`。ready 只提供 HTTPS URL。模型、原文、音频、基准和批次 JSON 都在 Git 忽略的 `data/`。

独立短语音与安装器：

```bash
uv run --no-sync python scripts/setup_qwen_native.py --device cpu
uv run --no-sync python scripts/run_topics_local.py tts \
  --text-file /absolute/path/to/narration.txt
```

安装器显式指定 Cargo target-dir，避免本机全局 `/data7t/cargo-target` 配置使安装位置失配；通过同目录原子 rename 安装，可在旧推理进程尚在运行时更新程序。

## CPU 实测与运行参数

短句为「各位球迷，成都蓉城，雄起！」，每次独立加载 CPU / f32 模型，无缓存命中。生成带采样，每次音频时长略有不同，因此同时比较耗时比。

| Rayon 线程 | 墙钟秒 | 音频秒 | 耗时比 | 峰值 RSS GiB |
| --- | ---: | ---: | ---: | ---: |
| 8 | 23.91 | 3.28 | 7.29 | 6.16 |
| 16 | 36.86 | 4.96 | 7.43 | 6.16 |
| 32 | 31.68 | 4.24 | 7.47 | 6.16 |

采用 `RAYON_NUM_THREADS=8`、每次一篇、`FI_QWEN_TIMEOUT_SECONDS=3600`。64 逻辑线程不等于应把每个推理实例的线程开到 64；这里以已测资源和稳定性为依据。短句不能代表长文耗时，完整文章实测另记录在 `data/topics/benchmarks/`。

基准样音：`data/topics/benchmarks/cpu-threads8/sample.m4a`。Kimi 最小真实检查为 HTTP 200，耗时 2.495 秒，223 tokens（不生成新闻）。不使用 Moonshot 开放平台地址或 GLM 参数。

2026-10-13 两篇已有真实正文的 CPU 全文验证：

| 文章 | 正文播报字符 | 音频秒 | 完整合成约耗时 | 观察峰值 RSS |
| --- | ---: | ---: | ---: | ---: |
| 蓉城一线队主场比赛 | 478（4 段） | 117.31 | 750.0 秒 | 8.245 GiB |
| 希拉谷中乙赛程预告 | 94（1 段） | 24.56 | 161.2 秒 | 6.163 GiB |

完整合成耗时按进程启动和 m4a 落盘时间估算，包含验证/转码，观察峰值来自 Linux VmHWM；短句基准则来自 `/usr/bin/time`。五次独立 native 调用（3 个短基准、2 篇全文）均成功。两篇 AAC 均为 24kHz 单声道，解码 PCM16 的 RMS 分别为 1780.00 / 2008.98，削波比例为 0。MinIO 完整读取 200、Range 206，返回字节与本地 SHA256 一致，Content-Type 为 audio/mp4。

两篇在原批次 `2b36e149bf1ed7567bbefae8c619ea523be539689892a3eb79f996e04db9e7db` 中原子更新，未改变原文、来源、日期、生成器版本、材料指纹或文章 id。数据库与实际 18092 GET JSON 一致，limit=1 验证成功。再次执行刷新命令仅耗时 4.593 秒，两份音频缓存文件均未修改。详细运行 JSON 在 `data/topics/benchmarks/full-chain-verification.json`，不提交 Git。

前述 CPU 基准阶段未添加定时任务；后续按用户授权安装了上述每日 07:00–22:30 半小时更新任务。运行后可读生产接口或本地接口：

```text
http://172.16.60.233:18092/api/v1/daily-topics?team_id=77680&limit=5
https://match.oryjk.cn/api/v1/daily-topics?team_id=77680&limit=5
```

自动更新只修改 Python 和本机定时任务，不重启后端；生产已运行此前部署的读取接口，直接读取新数据库批次。
