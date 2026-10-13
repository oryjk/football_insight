# local233 今日话题手动运行

此链路使用 Kimi Code `k3-256k` 整理真实新闻全文，用本机 Qwen Eric 四川话音色生成每篇独立音频，复用头像/球队 logo 的 `MinioAssetTarget` 上传，再通过 Python 原子 upsert 发布批次。Rust 只读数据库，小程序直接播放 HTTPS MinIO URL。

## 已安装组件

- CPU：Threadripper 3970X，32 核 / 64 线程；测量时约 233 GiB 可用内存。
- 模型：`data/models/Qwen3-TTS-12Hz-0.6B-CustomVoice`。
- 固定快照：`85e237c12c027371202489a0ec509ded67b5e4b5`，13 个文件共 2,498,388,392 字节，两份 safetensors 已按远端 LFS SHA256 校验。模型目录 `.football-snapshot.json` 保存安装校验信息。
- Runner：`data/native-qwen/bin/football-qwen-tts`，从仓库 `tools/qwen-native` 构建，CPU / f32，Eric + `sichuan_dialect`。安装器校验 any-tts 0.2.0 归档及 0.6B 补丁，不修改共享 Cargo 缓存。
- AAC：已安装 `ffmpeg` / `ffprobe`。先生成 24kHz 单声道 PCM16，检查非静音和削波，再转换 AAC/m4a。

官方模型与 Eric 音色说明：[Qwen 固定模型快照](https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/blob/85e237c12c027371202489a0ec509ded67b5e4b5/README.md)。这不是拿 Mac Metal 的速度推算 CPU。

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

没有添加定时任务。每次手动运行后可读：

```text
http://172.16.60.233:18092/api/v1/daily-topics?team_id=77680&limit=5
```

本次不部署、不重启生产后端；保留 local233 微信真实凭据与 ticket-monitor 配置。
