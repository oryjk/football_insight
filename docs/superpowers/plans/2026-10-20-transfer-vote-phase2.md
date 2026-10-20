# 转会投票第二期（拉票海报与小程序码）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 单张转会提名卡可以生成"帮我投一票"海报（球员、表态、赞踩与名次、分享人、扫码投一票的小程序码），保存到相册转发；后端提供该提名卡的小程序码接口。

**Architecture:** 后端在 `transfer` 模块新增 `TransferCodePort` 端口和 `TransferService::code_image`，Axum 新增 `GET /api/v1/transfers/nominations/{share_code}/wxacode`；微信小程序码适配器放在现有 `share_code/adapters/wechat_code.rs`，把战术板"按分享码缓存小程序码"的逻辑抽成共用函数，新增 `WechatTransferCode`（场景值 `s=share&c=transfer&t=<码>`，页面 `pages/transfer/view`）。小程序在转会分包内新增海报绘制、分享组合函数和分享弹层，单卡页"分享拉票"改为打开弹层，扫码进入时解析场景值。

**Tech Stack:** Rust 2024 / Axum 0.8 / reqwest / async-trait / thiserror 2；uni-app (Vue 3 + TS) / Bun test / 微信 canvas。

**Spec:** `docs/superpowers/specs/2026-10-20-transfer-vote-design.md`（第 6 节 wxacode 行、第 7 节 `TransferCodePort`、第 8.2 节 `transferPoster.ts`、第 9 节海报、第 10 节第二期）。第一期计划 `docs/superpowers/plans/2026-10-20-transfer-vote-phase1.md` 已完成并上线，本计划在其基础上增量修改。

## Global Constraints

- **第一期之后这些文件被改过，动手前先重新读当前版本，不要凭第一期的记忆修改**：`football_insight_mini/src/pages/transfer/view.vue`、`board.vue`、`useTransferBoard.ts`、`transfer.css`、`src/utils/transferLinks.ts`、`src/components/FiTransferCard.vue`、`src/pages/insights/components/InsightsTransferPanel.vue`、`src/pages/insights/index.vue`（新增了非主队拦截 `transferBlockedMessage`/`transferHint`、表态文案、切换不清空与 `stale`、`anchorTabsToTop`、榜名改为"最想送走/最想留下/最想买入"）。
- 界面风格与现有一致；组件化（组件只收属性、发事件，调接口和跳转放页面或组合函数，`bun run check:boundaries` 保证）；token 化（界面样式引用 `fi-tokens.css`；**canvas 海报的绘制颜色属于位图装饰色，沿用 `lineupPoster.ts` 的写法放在文件内 `palette` 常量里并注释**）。
- 后端六边形：端口在 `transfer/ports.rs`，编排在 `transfer/application.rs`，微信适配器在 `share_code/adapters`，装配在 `transfer/bootstrap.rs`。
- 新代码只放转会分包 `pages/transfer`（主包距 2MB 约 410KB），不要放主包。
- 只做微信小程序，不处理 `football_insight_h5`，不做 uni-app H5 构建验证。
- 后端只用 `rustfmt --edition 2024 <本次改动的文件>` 格式化，**不要格式化 `src/lib.rs`**（rustfmt 会顺着 `mod` 声明格式化整个后端）。
- 不对生产库做任何写操作；不部署；不上传小程序；不改 `manifest.json` 版本号（构建后若 `manifest.json`、`src/config/generatedMiniProgramVersion.ts` 被改动，`git checkout --` 还原）；不读不改 `football_insight_admin_android`；不打印、不提交 `.env` 与密钥。
- 提交信息：英文 subject，`feat(transfer): …` / `feat(mini): …` / `docs: …`，结尾 Co-Authored-By 行。
- 小程序码 `env_version` 为 `release`：体验版扫码只能打开正式版，这是既有限制（与战术板、回流海报一致），文案里沿用"正式版扫码打开"。

## Review Focus

1. 球员头像、分享人头像、队徽任一加载失败——海报仍生成，用名字首字或空位兜底，不报错。（Task 3 测试 `poster_draws_without_any_remote_images`）
2. 小程序码接口失败——明确提示"分享图生成失败，请重试"，不导出没有小程序码的海报。（Task 3 测试 `poster_requires_a_mini_program_code`；Task 4 由 `generate()` 捕获错误）
3. 游客点"生成海报"——先去登录，登录后回到这张卡。（Task 4 `generate()` 用 `buildTransferViewUrl(code)` 作为回跳地址）
4. 扫码进入：场景值 `s=share&c=transfer&t=<码>` 能解析出分享码，其他场景（战术板、回流）不会被误认。（Task 3 测试 `scene_parsing_accepts_only_transfer_cards`）
5. 很多张卡同时请求小程序码——缓存有上限，不会无限增长；同一张卡的并发请求只请求一次微信。（Task 1 测试 `transfer_codes_use_their_own_scene_and_cap_the_cache`）

---

## File Structure

| 文件 | 职责 |
|---|---|
| `football_insight_service_backend_rs/src/share_code/adapters/wechat_code.rs` | 抽出 `per_code_image` 共用函数；新增 `WechatTransferCode` 并实现 `TransferCodePort` |
| `…/share_code/adapters/wechat_code/tests.rs` | 新增转会场景测试 |
| `…/src/transfer/domain.rs` | `TransferCodeImage` |
| `…/src/transfer/ports.rs` | `TransferCodePort` |
| `…/src/transfer/application.rs` | `TransferService::new` 增加 `codes` 参数；`code_image` |
| `…/src/transfer/adapters/dto.rs`、`web.rs` | `CodeDto` 与 `GET …/wxacode` |
| `…/src/transfer/bootstrap.rs`、`src/app.rs` | 装配（需要 `AppConfig` 的小程序 appid/secret） |
| `football_insight_mini/src/types/transfer.ts`、`src/api/transfer.ts` | `TransferCodeImage`、`getTransferCode` |
| `src/utils/transferLinks.ts` | `parseTransferCode` 支持扫码场景值；`TRANSFER_SHARE_SCENE` |
| `src/pages/transfer/transferPoster.ts` | 海报尺寸常量、`drawTransferPoster`（纯绘制，便于单测） |
| `src/pages/transfer/transferPoster.test.ts` | 绘制与场景解析测试 |
| `src/pages/transfer/useTransferShare.ts` | 生成、预览、保存海报的流程 |
| `src/pages/transfer/components/TransferShareSheet.vue` | 分享弹层（分享给好友 + 生成海报） |
| `src/pages/transfer/view.vue` | "分享拉票"打开弹层；放置 canvas |

---

### Task 1: 后端——小程序码端口、服务与接口

**Files:**
- Modify: `football_insight_service_backend_rs/src/share_code/adapters/wechat_code.rs`
- Modify: `football_insight_service_backend_rs/src/share_code/adapters/wechat_code/tests.rs`
- Modify: `football_insight_service_backend_rs/src/transfer/domain.rs`、`ports.rs`、`application.rs`、`adapters/dto.rs`、`adapters/web.rs`、`bootstrap.rs`
- Modify: `football_insight_service_backend_rs/src/app.rs`

**Interfaces:**
- Produces: `crate::transfer::domain::TransferCodeImage { image_base64: String, mime_type: String }`；`crate::transfer::ports::TransferCodePort { async fn image(&self, share_code: &str) -> Result<TransferCodeImage, TransferError> }`；`TransferService::new(repo, directory, users, codes: Arc<dyn TransferCodePort>, clock)`（**参数顺序：codes 在 clock 之前**）；`TransferService::code_image(&self, user: Uuid, share_code: &str) -> Result<TransferCodeImage, TransferError>`；`crate::share_code::adapters::wechat_code::WechatTransferCode::new(app_id, secret)`；`build_transfer_routes(pool, token_port, config: &AppConfig)`。
- HTTP：`GET /api/v1/transfers/nominations/{share_code}/wxacode` → `{ "image_base64": "...", "mime_type": "image/png" }`；必须登录（401），分享码格式错 400，卡不存在 404，微信失败 503。

- [ ] **Step 1: 写适配器的失败测试**

在 `wechat_code/tests.rs` 末尾追加（`fixture` 与 `Calls` 已存在；`lineup: true` 让假微信接口不校验回流场景）：

```rust
#[tokio::test]
async fn transfer_codes_use_their_own_scene_and_cap_the_cache() {
    use crate::transfer::ports::TransferCodePort;
    let calls = Arc::new(Calls {
        lineup: true,
        ..Calls::default()
    });
    let (template, server) = fixture(calls.clone()).await;
    let mut adapter = WechatTransferCode::new("test-app".into(), "test-secret".into());
    adapter.api_base = template.api_base;
    adapter.client = template.client;
    let (a, a_again) = tokio::join!(adapter.image("ABCD1234"), adapter.image("ABCD1234"));
    assert_eq!(a.unwrap(), a_again.unwrap());
    assert_eq!(calls.codes.load(Ordering::SeqCst), 1);
    assert_eq!(
        calls.payloads.lock().unwrap()[0],
        json!({"scene":"s=share&c=transfer&t=ABCD1234","page":"pages/transfer/view","check_path":false,"env_version":"release","width":280})
    );
    for index in 0..(MAX_PER_CODE_TARGETS + 5) {
        adapter.image(&format!("T{index:07}")).await.unwrap();
    }
    assert!(adapter.codes.lock().await.len() <= MAX_PER_CODE_TARGETS);
    assert_eq!(
        adapter.image("bad").await.unwrap_err(),
        crate::transfer::domain::TransferError::Invalid
    );
    server.abort();
}
```

- [ ] **Step 2: 运行，确认编译失败**

Run: `cd football_insight_service_backend_rs && cargo test --lib share_code::adapters::wechat_code 2>&1 | tail -5`
Expected: 编译失败，`WechatTransferCode`、`MAX_PER_CODE_TARGETS`、`TransferCodePort` 未定义。

- [ ] **Step 3: 抽出共用函数并新增 `WechatTransferCode`**

`wechat_code.rs`：把 `const MAX_LINEUP_CODES: usize = 128;` 改名为 `pub(crate) const MAX_PER_CODE_TARGETS: usize = 128;`，并在 `WechatLineupCode` 定义之前加入共用函数：

```rust
#[derive(Debug, PartialEq, Eq)]
enum PerCodeError {
    Invalid,
    Unavailable,
}

/// 按分享码缓存小程序码：每个码一个缓存槽与在途锁，最多 MAX_PER_CODE_TARGETS 个；
/// 满了淘汰一个没有在途请求的槽。战术板与转会提名共用。
async fn per_code_image(
    codes: &Mutex<HashMap<String, Arc<WechatMiniCode>>>,
    template: (&str, &str, &reqwest::Client, &str),
    code: &str,
    scene: String,
    page: &str,
) -> Result<CodeImage, PerCodeError> {
    if code.len() != 8 || !code.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(PerCodeError::Invalid);
    }
    let (app_id, secret, client, api_base) = template;
    let adapter = {
        let mut codes = codes.lock().await;
        if let Some(adapter) = codes.get(code) {
            adapter.clone()
        } else {
            if codes.len() >= MAX_PER_CODE_TARGETS {
                // An in-flight entry has an extra Arc owner; keep its coalescing lock.
                let key = codes
                    .iter()
                    .find_map(|(key, entry)| (Arc::strong_count(entry) == 1).then(|| key.clone()))
                    .ok_or(PerCodeError::Unavailable)?;
                codes.remove(&key);
            }
            let mut adapter =
                WechatMiniCode::for_target(app_id.into(), secret.into(), scene, page.into());
            adapter.client = client.clone();
            adapter.api_base = api_base.into();
            let adapter = Arc::new(adapter);
            codes.insert(code.into(), adapter.clone());
            adapter
        }
    };
    adapter.cached_image().await.map_err(|_| PerCodeError::Unavailable)
}
```

把 `impl crate::lineup::ports::LineupCodePort for WechatLineupCode` 的函数体替换为调用共用函数（行为与原来完全相同，现有战术板测试必须原样通过）：

```rust
    {
        use crate::lineup::domain::{LineupCodeImage, LineupPublicError};
        let image = per_code_image(
            &self.codes,
            (&self.app_id, &self.secret, &self.client, &self.api_base),
            code,
            format!("s=share&c=lineup&l={code}"),
            "pages/lineup/view",
        )
        .await
        .map_err(|error| match error {
            PerCodeError::Invalid => LineupPublicError::Invalid,
            PerCodeError::Unavailable => LineupPublicError::Unavailable,
        })?;
        Ok(LineupCodeImage {
            image_base64: image.image_base64,
            mime_type: image.mime_type,
        })
    }
```

在文件末尾 `#[cfg(test)] mod tests;` 之前新增：

```rust
/// 转会提名卡的小程序码：扫码直达 pages/transfer/view，场景值携带提名分享码。
pub struct WechatTransferCode {
    app_id: String,
    secret: String,
    client: reqwest::Client,
    api_base: String,
    codes: Mutex<HashMap<String, Arc<WechatMiniCode>>>,
}

impl WechatTransferCode {
    pub fn new(app_id: String, secret: String) -> Self {
        let template = WechatMiniCode::new(app_id, secret);
        Self {
            app_id: template.app_id,
            secret: template.secret,
            client: template.client,
            api_base: template.api_base,
            codes: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl crate::transfer::ports::TransferCodePort for WechatTransferCode {
    async fn image(
        &self,
        code: &str,
    ) -> Result<crate::transfer::domain::TransferCodeImage, crate::transfer::domain::TransferError>
    {
        use crate::transfer::domain::{TransferCodeImage, TransferError};
        let image = per_code_image(
            &self.codes,
            (&self.app_id, &self.secret, &self.client, &self.api_base),
            code,
            format!("s=share&c=transfer&t={code}"),
            "pages/transfer/view",
        )
        .await
        .map_err(|error| match error {
            PerCodeError::Invalid => TransferError::Invalid,
            PerCodeError::Unavailable => TransferError::Unavailable,
        })?;
        Ok(TransferCodeImage {
            image_base64: image.image_base64,
            mime_type: image.mime_type,
        })
    }
}
```

`transfer/domain.rs` 增加：

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferCodeImage {
    pub image_base64: String,
    pub mime_type: String,
}
```

`transfer/ports.rs` 增加（并在 `use super::domain::{…}` 中加入 `TransferCodeImage`）：

```rust
#[async_trait]
pub trait TransferCodePort: Send + Sync {
    /// 提名卡分享码对应的小程序码；码格式错为 Invalid，微信失败为 Unavailable。
    async fn image(&self, share_code: &str) -> Result<TransferCodeImage, TransferError>;
}
```

- [ ] **Step 4: 运行适配器测试，确认通过且战术板测试不受影响**

Run: `cargo test --lib share_code::adapters::wechat_code 2>&1 | grep -E "^test |test result"`
Expected: 原有 9 个测试与新增 1 个全部 `ok`。

- [ ] **Step 5: 写服务与路由的失败测试**

`transfer/application.rs` 测试模块中：
1. 新增假实现并让 `service_at` 传入它（`TransferService::new` 的新参数在 `clock` 之前）：

```rust
    pub(crate) struct FakeCodes;
    #[async_trait::async_trait]
    impl TransferCodePort for FakeCodes {
        async fn image(&self, code: &str) -> Result<TransferCodeImage, TransferError> {
            Ok(TransferCodeImage { image_base64: format!("img-{code}"), mime_type: "image/png".into() })
        }
    }
```

`service_at` 中改为 `TransferService::new(repo.clone(), Arc::new(FakeDirectory), users.clone(), Arc::new(FakeCodes), Arc::new(move || now))`，并在测试模块 `use` 中加入 `crate::transfer::{domain::TransferCodeImage, ports::TransferCodePort}`。

2. 新增测试：

```rust
    #[tokio::test]
    async fn code_images_require_an_active_viewer_and_an_existing_card() {
        let (service, _, _) = service_at(at(5), true);
        let image = service.code_image(FAN, "CODE0001").await.unwrap();
        assert_eq!(image.image_base64, "img-CODE0001");
        // 非主队、本期结束也可以生成海报：拉票海报只是分享，不是投票
        assert!(service.code_image(RIVAL, "CODE0001").await.is_ok());
        assert_eq!(service.code_image(FAN, "bad").await.unwrap_err(), TransferError::Invalid);
        assert_eq!(service.code_image(FAN, "MISSING0").await.unwrap_err(), TransferError::NotFound);
        assert_eq!(service.code_image(Uuid::from_u128(9), "CODE0001").await.unwrap_err(), TransferError::Unauthorized);
        let (disabled, _, _) = service_at(at(5), false);
        assert_eq!(disabled.code_image(FAN, "CODE0001").await.unwrap_err(), TransferError::Unauthorized);
        let (closed, _, _) = service_at(at(25), true);
        assert!(closed.code_image(FAN, "CODE0001").await.is_ok());
    }
```

`transfer/adapters/web.rs` 测试模块中新增：

```rust
    #[tokio::test]
    async fn code_image_requires_login_and_returns_the_image_contract() {
        let path = "/api/v1/transfers/nominations/CODE0001/wxacode";
        assert_eq!(call(5, "GET", path, None, None).await.0, StatusCode::UNAUTHORIZED);
        let (status, body) = call(5, "GET", path, Some("Bearer good"), None).await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json, serde_json::json!({"image_base64": "img-CODE0001", "mime_type": "image/png"}));
        assert_eq!(call(5, "GET", "/api/v1/transfers/nominations/MISSING0/wxacode", Some("Bearer good"), None).await.0, StatusCode::NOT_FOUND);
    }
```

- [ ] **Step 6: 运行，确认编译失败**

Run: `cargo test --lib transfer:: 2>&1 | tail -5`
Expected: 编译失败，`code_image` 未定义、`TransferService::new` 参数数量不对。

- [ ] **Step 7: 实现服务、路由与装配**

`transfer/application.rs`：`TransferService` 增加字段 `codes: Arc<dyn TransferCodePort>`，`new` 签名改为 `new(repo, directory, users, codes: Arc<dyn TransferCodePort>, clock: Clock)`，并新增：

```rust
    /// 拉票海报的小程序码：只要求登录的正常账号和存在的提名卡；
    /// 不要求是主队、也不要求本期开放——海报只是分享，不是投票。
    pub async fn code_image(&self, user_id: Uuid, share_code: &str) -> Result<TransferCodeImage, TransferError> {
        if !valid_share_code(share_code) {
            return Err(TransferError::Invalid);
        }
        self.viewer_profile(Some(user_id)).await?.ok_or(TransferError::Unauthorized)?;
        self.repo.card(share_code, Some(user_id)).await?.ok_or(TransferError::NotFound)?;
        self.codes.image(share_code).await
    }
```

（`use` 中加入 `TransferCodeImage` 与 `TransferCodePort`。）

`transfer/adapters/dto.rs` 增加：

```rust
#[derive(Serialize, Debug, PartialEq)]
pub struct CodeDto { pub image_base64: String, pub mime_type: String }
impl From<crate::transfer::domain::TransferCodeImage> for CodeDto {
    fn from(i: crate::transfer::domain::TransferCodeImage) -> Self { Self { image_base64: i.image_base64, mime_type: i.mime_type } }
}
```

`transfer/adapters/web.rs`：`use super::dto::{…, CodeDto, …}`；路由加 `.route("/api/v1/transfers/nominations/{share_code}/wxacode", get(code_image))`；处理函数：

```rust
async fn code_image(State(state): Shared, headers: HeaderMap, Path(code): Path<String>) -> Result<Json<CodeDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state.service.code_image(user, &code).await.map(|i| Json(i.into())).map_err(map_error)
}
```

`transfer/bootstrap.rs`：

```rust
use crate::{config::AppConfig, share_code::adapters::wechat_code::WechatTransferCode};

pub fn build_transfer_routes(pool: PgPool, token_port: Arc<dyn TokenPort>, config: &AppConfig) -> Router {
    let store = Arc::new(PostgresTransferRepository::new(pool));
    let codes = Arc::new(WechatTransferCode::new(
        config.wechat_mini_app_id.clone(),
        config.wechat_mini_app_secret.clone(),
    ));
    let service = TransferService::new(store.clone(), store.clone(), store, codes, Arc::new(chrono::Utc::now));
    transfer_routes(Arc::new(TransferWebState { service: Arc::new(service), token_port }))
}
```

`src/app.rs`：`build_transfer_routes(pool.clone(), auth.token_port.clone(), config)`。

- [ ] **Step 8: 运行全部后端测试与 clippy**

Run: `cargo test 2>&1 | grep -E "^test result|FAILED|panicked" | tail -8 && cargo clippy --lib --tests 2>&1 | grep -B2 -A6 "src/transfer\|share_code" | grep -E "^(warning|error)" | head`
Expected: 全部 `ok`；clippy 对 `transfer`、`share_code` 无新告警。`tests/transfer_postgres.rs` 不受影响（它不构造 `TransferService`）。

- [ ] **Step 9: 格式化并提交**

```bash
rustfmt --edition 2024 src/share_code/adapters/wechat_code.rs src/share_code/adapters/wechat_code/tests.rs src/transfer/domain.rs src/transfer/ports.rs src/transfer/application.rs src/transfer/adapters/dto.rs src/transfer/adapters/web.rs src/transfer/bootstrap.rs src/app.rs
git add src/share_code/adapters/wechat_code.rs src/share_code/adapters/wechat_code/tests.rs src/transfer src/app.rs
git commit -m "feat(transfer): add mini program codes for transfer nominations"
```

---

### Task 2: 小程序——类型、接口与扫码场景解析

**Files:**
- Modify: `football_insight_mini/src/types/transfer.ts`、`src/api/transfer.ts`、`src/utils/transferLinks.ts`、`src/utils/transferLinks.test.ts`

**Interfaces:**
- Produces: `TransferCodeImage { image_base64: string; mime_type: 'image/png' | 'image/jpeg' }`；`getTransferCode(code: string): Promise<TransferCodeImage>`；`parseTransferCode(query)` 同时接受 `code=` 与扫码 `scene`（`s=share&c=transfer&t=<码>`）。

- [ ] **Step 1: 写失败测试**

`src/utils/transferLinks.test.ts` 末尾追加：

```ts
test('scene_parsing_accepts_only_transfer_cards', async () => {
  const { parseTransferCode } = await import('./transferLinks')
  expect(parseTransferCode({ scene: encodeURIComponent('s=share&c=transfer&t=ABCD1234') })).toBe('ABCD1234')
  expect(parseTransferCode({ scene: 's=share&c=transfer&t=ABCD1234' })).toBe('ABCD1234')
  expect(parseTransferCode({ scene: encodeURIComponent('s=share&c=lineup&l=ABCD1234') })).toBe('')
  expect(parseTransferCode({ scene: encodeURIComponent('s=share&c=transfer&t=bad') })).toBe('')
  expect(parseTransferCode({ scene: '%E0%A4%A' })).toBe('')
  expect(parseTransferCode({ code: 'ABCD1234', scene: 'whatever' })).toBe('ABCD1234')
})
```

- [ ] **Step 2: 运行确认失败**

Run: `cd football_insight_mini && bun test src/utils/transferLinks.test.ts 2>&1 | grep -E "^\(fail|^ *[0-9]+ (pass|fail)"`
Expected: `scene_parsing_accepts_only_transfer_cards` 失败。

- [ ] **Step 3: 实现**

`src/utils/transferLinks.ts` 中把 `parseTransferCode` 替换为：

```ts
/** 链接参数 code=，或扫小程序码进入时的 scene（s=share&c=transfer&t=<码>，可能经过 URL 编码）。 */
export function parseTransferCode(query: Record<string, unknown> | undefined): string {
  if (validTransferCode(query?.code)) return query!.code as string
  if (typeof query?.scene !== 'string') return ''
  try {
    const scene = decodeURIComponent(query.scene)
    const values = Object.fromEntries(scene.split('&').map(pair => pair.split('=')))
    return values.s === 'share' && values.c === 'transfer' && validTransferCode(values.t) ? values.t : ''
  } catch { return '' }
}
```

`src/types/transfer.ts` 末尾：

```ts
export interface TransferCodeImage { image_base64: string; mime_type: 'image/png' | 'image/jpeg' }
```

`src/api/transfer.ts`（`import type` 中加入 `TransferCodeImage`）：

```ts
export function getTransferCode(code: string): Promise<TransferCodeImage> {
  return request({ url: `/transfers/nominations/${encodeURIComponent(code)}/wxacode`, auth: true })
}
```

- [ ] **Step 4: 运行确认通过并检查**

Run: `bun test src/utils/transferLinks.test.ts 2>&1 | grep -E "^ *[0-9]+ (pass|fail)" && bun run type-check 2>&1 | tail -1`
Expected: 全部 pass；类型通过。

- [ ] **Step 5: 提交**

```bash
git add src/types/transfer.ts src/api/transfer.ts src/utils/transferLinks.ts src/utils/transferLinks.test.ts
git commit -m "feat(mini): add transfer code api and parse transfer scan scenes"
```

---

### Task 3: 小程序——海报绘制

**Files:**
- Create: `football_insight_mini/src/pages/transfer/transferPoster.ts`
- Create: `football_insight_mini/src/pages/transfer/transferPoster.test.ts`

**Interfaces:**
- Consumes: `TransferCardView`（`card`、`team`、`window`），`transferStance`、`TRANSFER_BOARD_LABELS`（`utils/transferLinks`）。
- Produces: `TRANSFER_POSTER = { canvasId: 'transfer-share-canvas', width: 750, height: 1200 }`；`interface TransferSharer { display_name: string; avatar_url: string | null }`；`transferPosterHeadline(view): string`；`drawTransferPoster(context, view, images: Map<string, string>, codePath: string, sharer: TransferSharer): void`（`codePath` 为空时抛 `Error('小程序码未就绪，请重试')`）。

- [ ] **Step 1: 写失败测试**

`src/pages/transfer/transferPoster.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import type { TransferCardView } from '../../types/transfer'
import { TRANSFER_POSTER, drawTransferPoster, transferPosterHeadline } from './transferPoster'

const view: TransferCardView = {
  team: { team_id: 77680, team_name: '成都蓉城', avatar_url: 'https://crest' },
  window: { code: '2026-winter', name: '2026 冬窗', opens_at: '', closes_at: '', open: true },
  can_vote: true,
  card: {
    share_code: 'ABCD1234', intent: 'out', returning: false, up: 128, down: 23, my_vote: 0, rank: 2, created_at: '',
    player: { player_id: 1, name: '某某', avatar_url: 'https://player', position: 'FW', shirt_number: 9, current_team: null, departed: false },
  },
}
function recorder() {
  const texts: string[] = [], images: string[] = []
  const noop = () => {}
  const context = { setFillStyle: noop, setFontSize: noop, fillText: (value: string) => texts.push(value), fillRect: noop, setStrokeStyle: noop, setLineWidth: noop, strokeRect: noop, beginPath: noop, arc: noop, save: noop, clip: noop, restore: noop, setTextAlign: noop, drawImage: (path: string) => images.push(path) } as unknown as UniApp.CanvasContext
  return { context, texts, images }
}

describe('transfer poster', () => {
  test('size and headline follow the card stance', () => {
    expect([TRANSFER_POSTER.width, TRANSFER_POSTER.height]).toEqual([750, 1200])
    expect(transferPosterHeadline(view)).toBe('成都蓉城球迷希望他离队')
    expect(transferPosterHeadline({ ...view, card: { ...view.card, intent: 'in', returning: true } })).toBe('成都蓉城球迷希望他回归')
  })

  test('poster_requires_a_mini_program_code', () => {
    const { context } = recorder()
    expect(() => drawTransferPoster(context, view, new Map(), '', { display_name: '球迷', avatar_url: null })).toThrow('小程序码')
  })

  test('draws player, stance, tally, rank, sharer and code', () => {
    const { context, texts, images } = recorder()
    const sharer = { display_name: '转发球迷', avatar_url: 'https://sharer' }
    drawTransferPoster(context, view, new Map([['https://crest', '/crest.png'], ['https://player', '/player.png'], ['https://sharer', '/sharer.png']]), '/code.png', sharer)
    expect(images).toEqual(['/crest.png', '/player.png', '/sharer.png', '/code.png'])
    for (const expected of ['某某', '成都蓉城球迷希望他离队', '赞成 128', '反对 23', '最想送走第 2 名', '转发球迷', '邀你表态', '扫码投一票']) {
      expect(texts.some(value => value.includes(expected))).toBe(true)
    }
  })

  test('poster_draws_without_any_remote_images', () => {
    const { context, texts, images } = recorder()
    drawTransferPoster(context, { ...view, card: { ...view.card, rank: null } }, new Map(), '/code.png', { display_name: '', avatar_url: null })
    expect(images).toEqual(['/code.png'])
    // 球员头像缺失用名字首字；分享人昵称为空时显示"球迷"
    expect(texts.includes('某')).toBe(true)
    expect(texts.includes('球迷')).toBe(true)
    expect(texts.some(value => value.includes('最想送走第'))).toBe(false)
  })
})
```

- [ ] **Step 2: 运行确认失败**

Run: `bun test src/pages/transfer/transferPoster.test.ts 2>&1 | tail -3`
Expected: 失败，模块不存在。

- [ ] **Step 3: 实现**

`src/pages/transfer/transferPoster.ts`：

```ts
import type { TransferCardView } from '../../types/transfer'
import { TRANSFER_BOARD_LABELS, transferStance } from '../../utils/transferLinks'

export const TRANSFER_POSTER = { canvasId: 'transfer-share-canvas', width: 750, height: 1200 }
export interface TransferSharer { display_name: string; avatar_url: string | null }

// 位图装饰色，与界面主题 token 无关（沿用 lineupPoster 的约定）。
const palette = { paper: '#ffffff', soft: '#f7f8fa', ink: '#15161b', muted: '#6d7280', red: '#dc2626', redSoft: '#fff1f0', line: '#ececf1' }

function clipped(value: string, limit: number): string {
  const chars = Array.from(value)
  return chars.length > limit ? chars.slice(0, limit - 1).join('') + '…' : value
}

export function transferPosterHeadline(view: TransferCardView): string {
  return `${view.team.team_name}球迷希望他${transferStance(view.card)}`
}

export function drawTransferPoster(context: UniApp.CanvasContext, view: TransferCardView, images: Map<string, string>, codePath: string, sharer: TransferSharer): void {
  if (!codePath) throw new Error('小程序码未就绪，请重试')
  const { card, team, window } = view
  const text = (value: string, x: number, y: number, size: number, color = palette.ink) => {
    context.setFillStyle(color); context.setFontSize(size); context.fillText(value, x, y)
  }
  const circleImage = (path: string | undefined, cx: number, cy: number, r: number, fallback: string, fallbackSize: number) => {
    context.save(); context.beginPath(); context.arc(cx, cy, r, 0, Math.PI * 2); context.clip()
    context.setFillStyle(palette.soft); context.fillRect(cx - r, cy - r, r * 2, r * 2)
    if (path) context.drawImage(path, cx - r, cy - r, r * 2, r * 2)
    else { context.setTextAlign('center'); text(fallback, cx, cy + fallbackSize * 0.35, fallbackSize, palette.muted) }
    context.restore(); context.setTextAlign('left')
  }

  context.setFillStyle(palette.paper); context.fillRect(0, 0, 750, 1200)
  // 顶部：队徽 + 球队名 + 期名
  const crest = team.avatar_url ? images.get(team.avatar_url) : undefined
  if (crest) context.drawImage(crest, 40, 48, 64, 64)
  text(clipped(team.team_name, 12), crest ? 120 : 40, 92, 34)
  text(`${window.name} · 转会表态`, 40, 152, 26, palette.red)

  // 中部：球员大头像、名字、表态
  const avatar = card.player.avatar_url ? images.get(card.player.avatar_url) : undefined
  circleImage(avatar, 375, 360, 130, Array.from(card.player.name)[0] ?? '球', 96)
  context.setTextAlign('center')
  text(clipped(card.player.name, 10), 375, 560, 48)
  text(transferPosterHeadline(view), 375, 628, 36, palette.red)
  context.setTextAlign('left')

  // 数据行：赞成 / 反对 / 名次
  context.setFillStyle(palette.redSoft); context.fillRect(40, 680, 670, 120)
  context.setTextAlign('center')
  text(`赞成 ${card.up}`, 190, 755, 34, palette.red)
  text(`反对 ${card.down}`, 375, 755, 34, palette.ink)
  if (card.rank !== null) text(`${TRANSFER_BOARD_LABELS[card.intent]}第 ${card.rank} 名`, 570, 755, 28, palette.muted)
  context.setTextAlign('left')

  // 底部：分享人 + 小程序码
  context.setStrokeStyle(palette.line); context.setLineWidth(2); context.strokeRect(40, 850, 670, 0)
  const name = sharer.display_name.trim() || '球迷'
  const sharerAvatar = sharer.avatar_url ? images.get(sharer.avatar_url) : undefined
  circleImage(sharerAvatar, 92, 960, 52, Array.from(name)[0] ?? '球', 40)
  text(clipped(name, 12), 164, 945, 30)
  text('邀你表态：你同意吗？', 164, 990, 24, palette.muted)
  text('扫码投一票 · 足球洞察', 40, 1110, 24, palette.muted)
  text('正式版扫码打开', 40, 1150, 20, palette.muted)
  context.drawImage(codePath, 520, 900, 190, 190)
}
```

- [ ] **Step 4: 运行确认通过**

Run: `bun test src/pages/transfer/transferPoster.test.ts 2>&1 | grep -E "^ *[0-9]+ (pass|fail)"`
Expected: `4 pass`

- [ ] **Step 5: 提交**

```bash
git add src/pages/transfer/transferPoster.ts src/pages/transfer/transferPoster.test.ts
git commit -m "feat(mini): draw the transfer vote poster"
```

---

### Task 4: 小程序——分享流程、分享弹层与单卡页接入

**Files:**
- Create: `football_insight_mini/src/pages/transfer/useTransferShare.ts`
- Create: `football_insight_mini/src/pages/transfer/components/TransferShareSheet.vue`
- Modify: `football_insight_mini/src/pages/transfer/view.vue`

**Interfaces:**
- Consumes: Task 2 `getTransferCode`；Task 3 `drawTransferPoster`、`TRANSFER_POSTER`；`utils/canvasPoster`（`exportCanvasToTempFile`、`flushCanvas`、`materializePosterCode`、`savePosterWithPermission`）；`transferVoteDeps`（登录与会话）；`useAppSession`。
- Produces: `useTransferShare(contextKey: () => string, returnUrl: () => string)` → `{ visible, busy, error, imagePath, open(view), generate(), preview(), save(), reset() }`；`TransferShareSheet` 属性 `{ visible, busy, imagePath, error }`，事件 `update:visible`、`generate`、`preview`、`save`。

- [ ] **Step 1: 实现分享流程**

`src/pages/transfer/useTransferShare.ts`（结构对照 `pages/lineup/useLineupShare.ts`，去掉战术板的卡片图）：

```ts
import { getCurrentInstance, nextTick, onScopeDispose, ref, watch } from 'vue'
import { getTransferCode } from '../../api/transfer'
import { transferVoteDeps } from '../../composables/useTransferVotes'
import { useAppSession } from '../../stores/appSession'
import type { TransferCardView } from '../../types/transfer'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { exportCanvasToTempFile, flushCanvas, materializePosterCode, savePosterWithPermission } from '../../utils/canvasPoster'
import { drawTransferPoster, TRANSFER_POSTER, type TransferSharer } from './transferPoster'

/** 单卡页"分享拉票"：好友卡片走页面 onShareAppMessage；这里负责生成、预览、保存海报。 */
export function useTransferShare(contextKey: () => string, returnUrl: () => string) {
  const instance = getCurrentInstance(), deps = transferVoteDeps(), session = useAppSession()
  const snapshot = ref<TransferCardView | null>(null), visible = ref(false), busy = ref(false), error = ref(''), imagePath = ref('')
  let generation = 0, disposed = false
  // 画布同一时间只给一次绘制使用，避免新的绘制覆盖仍在导出的旧图。
  let rendering: Promise<unknown> = Promise.resolve()
  function reset(): void { generation++; snapshot.value = null; visible.value = false; busy.value = false; imagePath.value = ''; error.value = '' }
  watch(contextKey, reset, { flush: 'sync' })

  function imageInfo(src: string): Promise<string> {
    return new Promise((resolve, reject) => uni.getImageInfo({ src, success: result => resolve(result.path), fail: reject }))
  }
  async function render(view: TransferCardView, codePath: string, sharer: TransferSharer, current: () => boolean): Promise<string> {
    const previous = rendering
    let release: () => void = () => {}
    rendering = new Promise<void>(resolve => { release = resolve })
    await previous.catch(() => {})
    try {
      if (!current()) return ''
      const urls = [...new Set([view.team.avatar_url, view.card.player.avatar_url, sharer.avatar_url].filter((url): url is string => Boolean(url)))]
      const images = new Map<string, string>()
      // 头像或队徽加载失败不阻断海报，绘制时用文字占位。
      await Promise.all(urls.map(async url => { try { images.set(url, await imageInfo(url)) } catch { /* fallback */ } }))
      await nextTick()
      if (!current()) return ''
      const canvas = uni.createCanvasContext(TRANSFER_POSTER.canvasId, instance)
      drawTransferPoster(canvas, view, images, codePath, sharer)
      await flushCanvas(canvas)
      if (!current()) return ''
      const path = await exportCanvasToTempFile({ ...TRANSFER_POSTER, fileType: 'png', instance })
      return current() ? path : ''
    } finally { release() }
  }

  function open(view: TransferCardView): void {
    if (disposed) return
    reset(); snapshot.value = view; visible.value = true
  }

  async function generate(): Promise<void> {
    const view = snapshot.value
    if (!view || busy.value || disposed) return
    if (!deps.loggedIn()) { deps.login(returnUrl()); return }
    const owner = generation, key = contextKey(), revision = deps.revision()
    const current = () => !disposed && owner === generation && key === contextKey() && revision === deps.revision()
    busy.value = true; error.value = ''; imagePath.value = ''
    try {
      const user = await session.ensureSessionReady()
      if (!current()) return
      if (!user) { deps.login(returnUrl()); return }
      const code = await getTransferCode(view.card.share_code)
      if (!current()) return
      const materialized = await materializePosterCode(code)
      try {
        const codePath = await imageInfo(materialized.path)
        if (!current()) return
        const path = await render(view, codePath, { display_name: user.display_name, avatar_url: user.avatar_url }, current)
        if (current()) imagePath.value = path
      } finally { materialized.cleanup() }
    } catch (cause) {
      if (!current()) return
      if (isUnauthorizedError(cause)) { deps.invalidate(revision); deps.login(returnUrl()); return }
      error.value = extractApiErrorMessage(cause, '分享图生成失败，请重试')
    } finally { if (current()) busy.value = false }
  }
  function preview(): void { if (imagePath.value) uni.previewImage({ urls: [imagePath.value] }) }
  function save(): void {
    const path = imagePath.value, owner = generation
    if (path) void savePosterWithPermission(path, uni, () => !disposed && generation === owner && imagePath.value === path)
  }
  onScopeDispose(() => { disposed = true; reset() })
  return { visible, busy, error, imagePath, open, generate, preview, save, reset }
}
```

- [ ] **Step 2: 分享弹层**

`src/pages/transfer/components/TransferShareSheet.vue`：

```vue
<template>
  <FiBottomSheet :visible="visible" title="分享拉票" height="72vh" :z-index="100" @update:visible="emit('update:visible', $event)">
    <view class="share-content">
      <text class="share-copy">发给球友，看看他们同不同意。</text>
      <!-- #ifdef MP-WEIXIN -->
      <button class="share-action share-action--primary" open-type="share">分享给好友</button>
      <!-- #endif -->
      <button class="share-action" :disabled="busy" @tap="emit('generate')">{{ busy ? '正在准备…' : imagePath ? '重新生成海报' : '生成拉票海报' }}</button>
      <text v-if="error" class="share-error">{{ error }}</text>
      <template v-if="imagePath">
        <image :src="imagePath" class="share-preview" mode="widthFix" @tap="emit('preview')" />
        <view class="share-row">
          <button class="share-action" @tap="emit('preview')">预览大图</button>
          <button class="share-action share-action--primary" @tap="emit('save')">保存到相册</button>
        </view>
      </template>
      <text class="share-copy">海报上的小程序码打开正式版；体验版可先用好友卡片验证。</text>
    </view>
  </FiBottomSheet>
</template>
<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
defineProps<{ visible: boolean; busy: boolean; imagePath: string; error: string }>()
const emit = defineEmits<{ (e: 'update:visible', value: boolean): void; (e: 'generate' | 'preview' | 'save'): void }>()
</script>
<style scoped>
.share-content { display: flex; flex-direction: column; gap: var(--fi-space-20); }
.share-copy { color: var(--fi-color-text-muted); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.share-error { color: var(--fi-color-primary-deep); font-size: var(--fi-font-24); }
.share-row { display: flex; gap: var(--fi-space-16); }
.share-action { margin: 0; padding: var(--fi-space-20); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); color: var(--fi-color-text-primary); font-size: var(--fi-font-28); line-height: var(--fi-leading-normal); }
.share-row > .share-action { flex: 1; }
.share-action::after { border: 0; }
.share-action--primary { background: var(--fi-color-primary); color: var(--fi-color-surface); }
.share-preview { width: 100%; border-radius: var(--fi-radius-md); }
</style>
```

（禁用态沿用战术板分享弹层的 `[disabled]` 写法会触发小程序组件属性选择器告警——这里不写 `[disabled]` 样式，按钮的原生禁用外观即可。）

- [ ] **Step 3: 单卡页接入**

先 **重新读取当前的 `src/pages/transfer/view.vue`**（第一期之后被修改过）。然后：

1. 模板中把 `<button class="transfer-action transfer-action--active" open-type="share">分享拉票</button>` 改为 `<button class="transfer-action transfer-action--active" @tap="openShare">分享拉票</button>`；在 `</view>`（`transfer-page` 的结束标签）之前加入：

```vue
    <TransferShareSheet v-model:visible="shareVisible" :busy="shareBusy" :image-path="shareImage" :error="shareError" @generate="share.generate" @preview="share.preview" @save="share.save" />
    <canvas canvas-id="transfer-share-canvas" class="transfer-canvas" :style="{ width: '750px', height: '1200px' }" />
```

2. `page-meta` 改为 `<page-meta :page-style="navigationPageStyle + (shareVisible ? 'overflow: hidden;' : '')" />`（弹层打开时锁定页面滚动，与战术板一致）。

3. script 增加：

```ts
import TransferShareSheet from './components/TransferShareSheet.vue'
import { useTransferShare } from './useTransferShare'
const share = useTransferShare(() => `${code.value}:${session.revision.value}`, () => buildTransferViewUrl(code.value))
const { visible: shareVisible, busy: shareBusy, error: shareError, imagePath: shareImage } = share
function openShare(): void { if (view.value) share.open(view.value) }
```

（`share` 的声明放在 `code`、`session` 声明之后。）

4. `src/pages/transfer/transfer.css` 末尾追加（离屏画布，位置不影响布局）：

```css
.transfer-canvas { position: fixed; left: -3000rpx; top: 0; pointer-events: none; }
```

- [ ] **Step 4: 全量检查与构建**

Run:

```bash
bun test > /tmp/transfer2-mini.log 2>&1; grep -E "^\(fail|^ *[0-9]+ (pass|fail)" /tmp/transfer2-mini.log
bun run type-check 2>&1 | tail -1
bun run check:boundaries | tail -1
bun run build:mp-weixin > /tmp/transfer2-build.log 2>&1; echo build=$?
bun run verify:mp-components | tail -1
grep -c "transfer-share-canvas" dist/build/mp-weixin/pages/transfer/view.wxml
```

Expected: 0 fail；类型、边界通过；`build=0`；组件已注册；画布出现在单卡页产物中。构建后若 `manifest.json` / `generatedMiniProgramVersion.ts` 被改动，`git checkout --` 还原。

- [ ] **Step 5: 提交**

```bash
git add src/pages/transfer/useTransferShare.ts src/pages/transfer/components/TransferShareSheet.vue src/pages/transfer/view.vue src/pages/transfer/transfer.css
git commit -m "feat(mini): share transfer nominations with a vote poster"
```

---

### Task 5: 文档

**Files:**
- Modify: `CHANGELOG.md`、`FEATURES.md`、`DEPLOYMENT.md`

- [ ] **Step 1: 写文档**

- `CHANGELOG.md` 顶部新增"2026-10-20 · 转会投票第二期：拉票海报与小程序码（代码完成，待部署与上传）"：变更、原因（spec 第 10 节第二期）、验证（写实际测试数字）、发布状态（无迁移；生产后端未部署；小程序未上传；海报上的码打开正式版，体验版扫码进不去是既有限制）。
- `FEATURES.md` "转会投票第一期"小节下补一行第二期概述。
- `DEPLOYMENT.md` 转会小节补：第二期无数据库迁移，部署后端（需要已有的小程序 appid/secret 环境变量，与战术板小程序码共用）→ 上传小程序；核对 `GET /api/v1/transfers/nominations/{码}/wxacode` 未登录 401。

- [ ] **Step 2: 提交并推送**

```bash
git add CHANGELOG.md FEATURES.md DEPLOYMENT.md
git commit -m "docs: record transfer vote phase 2"
git pull --rebase && git push
```

---

## Self-Review 记录

- **Spec 覆盖**：第 6 节 wxacode 接口 → Task 1；第 7 节 `TransferCodePort` 与适配器放 `share_code` → Task 1；第 8.2 `transferPoster.ts` 复用 `canvasPoster` → Task 3、4；第 9 节海报内容（队徽 + 期名、球员头像与名字、"{球队}球迷希望他{离队/留下/加盟/回归}"、赞踩数与"{榜名}第 N"、分享人"邀你表态"、"扫码投一票"、码失败不导出、头像失败占位、保存相册授权引导）→ Task 3、4；缓存 24 小时 / 失败 30 秒 → 复用 `WechatMiniCode::cached_image`（Task 1）。
- **已裁定**：生成海报不要求是主队、也不要求本期开放（海报只是分享）；转会不做战术板那样的"好友卡片专用图"，好友卡片沿用页面截图，海报是唯一新增的图。
- **类型一致性**：`TransferService::new` 新参数顺序为 `(repo, directory, users, codes, clock)`，Task 1 的测试、实现与 bootstrap 一致；前端 `TransferCodeImage` 与后端 `CodeDto` 字段一致。
