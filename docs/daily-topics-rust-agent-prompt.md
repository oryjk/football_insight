# 给 Rust agent 的提示词

请在 `football_insight_service_backend_rs` 实现「今日话题」读取接口与数据库迁移。先阅读根目录和后端 AGENTS.md、README、DEPLOYMENT.md，以及 `docs/daily-topics-contract.md`。不要修改小程序或 Python，它们已按该合同实现。

Python 的 `sina_csl_scraper/src/sina_csl_scraper/topics/` 已负责采集原文、LLM 整理、Qwen Eric 四川话 TTS、MinIO 上传、单事务写入。`TopicRepository.save()` 写入 `f_i_daily_topic_editions`，不自动建表。Rust 只读结果，不在请求里抓新闻、调用 LLM/TTS 或代理音频字节。

请完成：

1. 新增 SQLx 迁移，创建合同中的表与索引。与 Python 对齐：id TEXT、team_id BIGINT、edition_date DATE、published_at TIMESTAMPTZ、payload JSONB、source_fingerprint TEXT、generator_version TEXT。不要复用 f_i_team_insights 或旧 public-config 的 home_daily_insight_audio。
2. 实现公共 `GET /api/v1/daily-topics?team_id=77680&limit=5`，默认77680与5，limit限制1..5，非法参数400。按 team_id 精确筛选、published_at DESC 读取最新批次，禁止跨球队混用。直接返回合同JSON，不新增 data 包装。items截断到limit，保留文章稳定id、正文、来源、时间和音频。无结果200返回team_id、edition_date:null、updated_at:null、items:[]。
3. 遵循既有分层，拆开DTO/领域数据、查询仓库、应用查询、HTTP handler/router；不要放进一个大文件，不读取Python本地latest.json。
4. audio.status=pending/failed时url必须null；ready时传HTTPS URL、dialect:sichuan、duration_seconds。小程序请求Rust内容JSON，点击播放时直接请求MinIO音频URL。密钥不返回客户端。
5. 测试参数边界、空结果、球队隔离、最新批次、5篇上限、来源及音频字段。运行cargo test与必要集成测试。
6. 验证前重启实际承接测试请求的后端。小程序开发接口是local233 `http://172.16.60.233:18092/api/v1`。先确认测试数据库隔离，不把迁移默默应用到生产共享库。若测试库实际为生产库，明确告知并遵守当前用户授权范围。
7. 默认不部署生产。回报明确区分测试后端已重启、生产后端未部署。线上后端在peiqian，jd是Nginx入口；若另获授权部署，再走现有Docker流程。

联调顺序：隔离库迁移 -> Python generate --write-db -> 重启local233服务 -> GET接口比对Python latest.json -> 小程序验证卡片/全文/来源/语音。请回报实际完成和未完成项，不用占位文章冒充真实新闻。
