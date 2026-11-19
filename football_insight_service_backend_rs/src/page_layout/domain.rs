use std::collections::{HashMap, HashSet};

use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub const MAX_IMAGE_URL_CHARS: usize = 2048;
pub const MAX_UPLOAD_IMAGE_BYTES: usize = 4 * 1024 * 1024;
pub const DEFAULT_PHOENIX_STADIUM_BG_URL: &str =
    "https://oryjk.cn:82/minio/football-insight/static/user/phoenix-stadium-bg.webp";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageGroup {
    PrimaryTab,
    Feature,
    Detail,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockDefinition {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct PageDefinition {
    pub page_key: &'static str,
    pub title: &'static str,
    pub route_path: &'static str,
    pub group: PageGroup,
    pub default_background_image_url: Option<&'static str>,
    pub blocks: &'static [BlockDefinition],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageLayoutBlock {
    pub key: String,
    pub visible: bool,
    #[serde(default)]
    pub background_image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPageLayout {
    pub page_key: String,
    pub background_image_url: Option<String>,
    pub blocks: Vec<PageLayoutBlock>,
    pub version: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedPageLayoutBlock {
    pub key: String,
    pub label: String,
    pub description: String,
    pub visible: bool,
    pub background_image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedPageLayout {
    pub page_key: String,
    pub title: String,
    pub route_path: String,
    pub group: PageGroup,
    pub background_image_url: Option<String>,
    pub blocks: Vec<ResolvedPageLayoutBlock>,
    pub version: i64,
    pub is_customized: bool,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicPageLayout {
    pub page_key: String,
    pub background_image_url: Option<String>,
    pub blocks: Vec<PageLayoutBlock>,
    pub version: i64,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavePageLayoutInput {
    pub background_image_url: Option<String>,
    pub blocks: Vec<PageLayoutBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPageLayoutWrite {
    pub page_key: String,
    pub background_image_url: Option<String>,
    pub blocks: Vec<PageLayoutBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLayoutImage {
    pub mime_type: &'static str,
    pub ext: &'static str,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum PageLayoutError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("page layout not found")]
    NotFound,
    #[error("image storage is not configured")]
    ImageStorageUnavailable,
    #[error("page layout operation failed")]
    Internal,
}

const HOME_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "daily_topics",
        label: "每日电报",
        description: "每日新闻快讯与语音播报卡片",
    },
    BlockDefinition {
        key: "my_team_support",
        label: "我的主队",
        description: "主队下一场比赛、开票倒计时与赛前助力",
    },
    BlockDefinition {
        key: "fan_entries",
        label: "球迷互动入口",
        description: "排阵容与转会表态快捷入口",
    },
    BlockDefinition {
        key: "story_pulse",
        label: "联赛脉冲看点",
        description: "本轮摘要、焦点比赛脉冲与看点",
    },
    BlockDefinition {
        key: "standings",
        label: "积分榜摘要",
        description: "联赛积分榜与近十场走势",
    },
    BlockDefinition {
        key: "top_scorers",
        label: "射手榜摘要",
        description: "射手榜前五与逐场进球柱状图",
    },
];

const RANKINGS_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "rankings_standings",
        label: "积分榜入口",
        description: "积分榜总览与轮次切换卡片",
    },
    BlockDefinition {
        key: "rankings_surface",
        label: "球队与球员榜",
        description: "球队榜、球员榜分类切换面板",
    },
    BlockDefinition {
        key: "matches_progress",
        label: "赛季进度",
        description: "当前赛季轮次进度总览",
    },
    BlockDefinition {
        key: "matches_upcoming",
        label: "即将到来赛程",
        description: "未开赛轮次与对阵安排",
    },
    BlockDefinition {
        key: "matches_results",
        label: "最近赛果",
        description: "按轮次分组的历史完赛结果",
    },
];

const INSIGHTS_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "ticket_watch_entry",
        label: "余票看板入口",
        description: "快速进入实时余票与回流监控",
    },
    BlockDefinition {
        key: "secondary_entries",
        label: "战术与转会入口",
        description: "排阵容、转会表态等互动入口",
    },
    BlockDefinition {
        key: "team_selector",
        label: "球队选择器",
        description: "切换当前查看洞察的球队",
    },
    BlockDefinition {
        key: "match_outlook",
        label: "赛前数据倾向",
        description: "Elo V2 胜平负概率与赛前倾向",
    },
    BlockDefinition {
        key: "team_insight",
        label: "球队洞察与贡献",
        description: "进球、助攻与失球贡献归因分析",
    },
];

const SEAT_SWAP_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "notice",
        label: "状态提示横幅",
        description: "未开放或未登录提示条",
    },
    BlockDefinition {
        key: "region_map",
        label: "看台区域面板",
        description: "球场看台分区分布与筛选",
    },
    BlockDefinition {
        key: "pool_list",
        label: "换座候选池",
        description: "可申请交换的座位列表",
    },
];

const USER_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "account_hero",
        label: "账号头图与会员卡",
        description: "用户头像、昵称与当前会员等级卡",
    },
    BlockDefinition {
        key: "account_info",
        label: "账号信息面板",
        description: "绑定状态、邀请码与权益入口",
    },
    BlockDefinition {
        key: "season_recap_entry",
        label: "赛季总结入口",
        description: "前往生成个人赛季观赛总结",
    },
    BlockDefinition {
        key: "account_actions",
        label: "账号管理与退出",
        description: "编辑资料、切换账号或退出登录",
    },
    BlockDefinition {
        key: "settings_entry",
        label: "设置与审核控制",
        description: "白名单管理员设置页入口",
    },
];

const SEASON_RECAP_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "heading",
        label: "顶部标题区",
        description: "赛季总结页标题与说明",
    },
    BlockDefinition {
        key: "summary",
        label: "观赛数据总览",
        description: "主客场场次与总里程摘要",
    },
    BlockDefinition {
        key: "theme_picker",
        label: "海报主题选择",
        description: "深浅风格海报背景切换",
    },
    BlockDefinition {
        key: "matches",
        label: "观赛场次与距离",
        description: "勾选本赛季现场观赛记录",
    },
    BlockDefinition {
        key: "formula",
        label: "里程计算说明",
        description: "折叠展示里程核算口径",
    },
];

const STUDENT_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "intro",
        label: "功能简介",
        description: "学员专属服务顶部说明",
    },
    BlockDefinition {
        key: "contact_qr",
        label: "联系微信二维码",
        description: "负责人联系二维码卡片",
    },
    BlockDefinition {
        key: "services",
        label: "服务购买列表",
        description: "可选学员服务商品与下单入口",
    },
    BlockDefinition {
        key: "orders",
        label: "购买记录",
        description: "已下单的学员服务历史记录",
    },
    BlockDefinition {
        key: "email_reminder",
        label: "回流提醒邮箱",
        description: "设置余票回流邮件通知地址",
    },
];

const USER_SETTINGS_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "version_info",
        label: "当前版本信息",
        description: "显示小程序当前构建版本号",
    },
    BlockDefinition {
        key: "review_status",
        label: "审核状态",
        description: "显示当前版本的审核状态",
    },
    BlockDefinition {
        key: "review_control",
        label: "审核状态切换",
        description: "白名单管理员切换审核态开关",
    },
    BlockDefinition {
        key: "h5_test_login",
        label: "H5 测试账号",
        description: "H5 联调白名单快捷切换登录",
    },
];

const MEMBERSHIP_PURCHASE_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "availability_notice",
        label: "开放状态提示",
        description: "会员支付未开放时的提示卡片",
    },
    BlockDefinition {
        key: "tier_list",
        label: "会员档位列表",
        description: "各会员等级与价格选择卡片",
    },
    BlockDefinition {
        key: "benefits",
        label: "会员权益说明",
        description: "选中档位包含的权益详情",
    },
    BlockDefinition {
        key: "extra_links",
        label: "等级与邀请入口",
        description: "等级说明与邀请好友免费升级",
    },
];

const TEAM_BOARD_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "hero",
        label: "球队头图卡",
        description: "球队基础资料与统计快照",
    },
    BlockDefinition {
        key: "launcher",
        label: "发布帖子入口",
        description: "战术板发帖与筛选操作条",
    },
    BlockDefinition {
        key: "posts",
        label: "球迷帖子列表",
        description: "球队战术板讨论帖子流",
    },
];

const SUPPORT_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "hero",
        label: "助力说明头卡",
        description: "比赛背景与赛前助力引导",
    },
    BlockDefinition {
        key: "scoreboard",
        label: "助力记分板",
        description: "主客队票数对比进度条",
    },
    BlockDefinition {
        key: "stance_actions",
        label: "阵营表态操作",
        description: "投票支持与拉票规则说明",
    },
    BlockDefinition {
        key: "poster",
        label: "助力分享海报",
        description: "生成后的助力海报预览卡片",
    },
];

const TOPICS_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "heading",
        label: "页面标题区",
        description: "全部话题顶部标题与描述",
    },
    BlockDefinition {
        key: "list",
        label: "历史话题列表",
        description: "往期每日电报与话题归档",
    },
];

const TOPIC_DETAIL_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "header",
        label: "标题与字号栏",
        description: "电报标题、日期与字号切换",
    },
    BlockDefinition {
        key: "audio_player",
        label: "语音播报条",
        description: "云扬语音播报入口与播放器",
    },
    BlockDefinition {
        key: "roundup_content",
        label: "电报快讯正文",
        description: "每日电报分类快讯与互动区",
    },
    BlockDefinition {
        key: "article_gallery",
        label: "图集画廊",
        description: "单篇话题配图轮播（旧模式）",
    },
    BlockDefinition {
        key: "article_body",
        label: "正文与来源",
        description: "单篇话题段落与新闻来源（旧模式）",
    },
    BlockDefinition {
        key: "article_timeline",
        label: "球队话题时间线",
        description: "关联球队的历史话题时间轴",
    },
];

const TICKET_WATCH_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "header",
        label: "看板头部与切换",
        description: "当前比赛 / 历史回流 / 趋势 Tab",
    },
    BlockDefinition {
        key: "current_match",
        label: "当前比赛概览",
        description: "本场开票状态与总体余票卡片",
    },
    BlockDefinition {
        key: "focus_panel",
        label: "重点关注区域",
        description: "高频关注与意向看台摘要",
    },
    BlockDefinition {
        key: "tracked_interests",
        label: "我的盯票意向",
        description: "个人已关注的价位与看台列表",
    },
    BlockDefinition {
        key: "recent_reflux",
        label: "近期回流动态",
        description: "开票 10 分钟后的实时回流记录",
    },
    BlockDefinition {
        key: "best_windows",
        label: "最佳盯盘时段",
        description: "基于历史统计的高概率回流窗口",
    },
    BlockDefinition {
        key: "inventory_stack",
        label: "分区库存列表",
        description: "按票档与看台分组的实时余票",
    },
    BlockDefinition {
        key: "supply_insights",
        label: "供给洞察分析",
        description: "开票节奏与放票结构分析",
    },
    BlockDefinition {
        key: "history_selector",
        label: "历史比赛选择器",
        description: "历史 Tab 切换往期主场比赛",
    },
    BlockDefinition {
        key: "history_match",
        label: "历史比赛卡片",
        description: "历史 Tab 所选场次回流摘要",
    },
    BlockDefinition {
        key: "history_trend",
        label: "历史趋势统计",
        description: "多场次跨轮次回流趋势图表",
    },
    BlockDefinition {
        key: "share_poster",
        label: "分享海报面板",
        description: "余票看板长图海报预览与保存",
    },
];

const INSIGHTS_TACTICS_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "heading",
        label: "排阵容标题区",
        description: "页面标题与玩法引导",
    },
    BlockDefinition {
        key: "team_selector",
        label: "球队选择器",
        description: "选择查看或排布阵容的球队",
    },
    BlockDefinition {
        key: "tactics_panel",
        label: "共识与社区阵容",
        description: "球迷共识首发与热门方案列表",
    },
];

const INSIGHTS_TRANSFER_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "heading",
        label: "转会表态标题区",
        description: "页面标题与窗口说明",
    },
    BlockDefinition {
        key: "home_team_bar",
        label: "当前主队提示栏",
        description: "主队身份标识与切换提示",
    },
    BlockDefinition {
        key: "transfer_panel",
        label: "转会表态主面板",
        description: "留队、离队与引援投票面板",
    },
];

const LINEUP_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "team_picker",
        label: "球队选择器",
        description: "切换排阵目标球队",
    },
    BlockDefinition {
        key: "status_bar",
        label: "状态与社区入口",
        description: "草稿保存状态与社区列表链接",
    },
    BlockDefinition {
        key: "formation_bar",
        label: "阵型选择栏",
        description: "切换 4-4-2 / 4-3-3 等阵型",
    },
    BlockDefinition {
        key: "pitch",
        label: "战术板球场",
        description: "11 人首发拖拽与点选画布",
    },
    BlockDefinition {
        key: "hint",
        label: "排阵提示区",
        description: "一键补全与操作说明提示",
    },
    BlockDefinition {
        key: "action_bar",
        label: "发布与分享栏",
        description: "发布阵容与生成分享卡片操作",
    },
];

const LINEUP_VIEW_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "summary_panel",
        label: "阵容概览卡",
        description: "阵型、作者与重合度信息",
    },
    BlockDefinition {
        key: "pitch",
        label: "球场阵容展示",
        description: "只读首发阵型球场图",
    },
    BlockDefinition {
        key: "reaction_bar",
        label: "互动与分享栏",
        description: "赞踩表态与转发分享按钮",
    },
    BlockDefinition {
        key: "navigation_bar",
        label: "排阵跳转操作",
        description: "前往自己排阵或球队战术板",
    },
];

const LINEUP_LIST_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "title",
        label: "页面标题",
        description: "球队战术板社区列表标题",
    },
    BlockDefinition {
        key: "team_picker",
        label: "球队选择器",
        description: "按球队筛选社区阵容方案",
    },
    BlockDefinition {
        key: "sort_bar",
        label: "排序切换栏",
        description: "按最新或点赞数排序",
    },
    BlockDefinition {
        key: "list",
        label: "社区阵容列表",
        description: "球迷已发布的阵容卡片流",
    },
    BlockDefinition {
        key: "cta",
        label: "新建阵容按钮",
        description: "底部「我也排一套」行动按钮",
    },
];

const TRANSFER_BOARD_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "title",
        label: "转会榜标题区",
        description: "转会榜名称与转会窗状态",
    },
    BlockDefinition {
        key: "tabs",
        label: "意图分类标签",
        description: "留队 / 离队 / 引援分类切换",
    },
    BlockDefinition {
        key: "list",
        label: "转会提名列表",
        description: "按票数展示的球员提名卡片",
    },
];

const TRANSFER_VIEW_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "header",
        label: "提名标题区",
        description: "转会提名标题与窗口说明",
    },
    BlockDefinition {
        key: "card",
        label: "提名详情卡",
        description: "球员转会意图与投票操作卡",
    },
    BlockDefinition {
        key: "actions",
        label: "分享与跳转操作",
        description: "转发、看榜单与我也来提名",
    },
];

const TELEGRAPH_ENTRY_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "timeline",
        label: "电报正文与评论",
        description: "单条电报详情、赞踩与评论流",
    },
];

const TELEGRAPH_COMMENT_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "thread",
        label: "评论主楼与回复",
        description: "评论上下文及子回复讨论流",
    },
];

const DIRECTOR_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "header",
        label: "标题与窗口说明",
        description: "我来当总监标题、转会窗标签与建队进度说明",
    },
    BlockDefinition {
        key: "team_selector",
        label: "球队选择器",
        description: "切换建队模拟的目标球队",
    },
    BlockDefinition {
        key: "foreign_section",
        label: "5 外援建队区",
        description: "现役外援留队/离队决策与新外援引援名额",
    },
    BlockDefinition {
        key: "domestic_section",
        label: "本土引援与宣言",
        description: "本土球员出入调整与建队宣言填写",
    },
];

const DIRECTOR_VIEW_BLOCKS: &[BlockDefinition] = &[
    BlockDefinition {
        key: "header",
        label: "方案标题区",
        description: "总监建队方案标题与所属转会窗",
    },
    BlockDefinition {
        key: "comparison",
        label: "好友 1v1 分歧对比",
        description: "与邀请好友建队方案的差异对比卡",
    },
    BlockDefinition {
        key: "view_tabs",
        label: "视图切换栏",
        description: "方案详情与好友圈共识切换控件",
    },
    BlockDefinition {
        key: "plan_overview",
        label: "建队方案总览",
        description: "5 外援与本土阵容建队结果展示",
    },
    BlockDefinition {
        key: "comments",
        label: "点赞与评论区",
        description: "方案赞踩互动与球迷评论列表",
    },
    BlockDefinition {
        key: "circle_panel",
        label: "好友圈共识面板",
        description: "好友圈成员建队方案与共识统计",
    },
];

pub const PAGE_LAYOUT_CATALOG: &[PageDefinition] = &[
    PageDefinition {
        page_key: "home",
        title: "首页",
        route_path: "/pages/home/index",
        group: PageGroup::PrimaryTab,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: HOME_BLOCKS,
    },
    PageDefinition {
        page_key: "rankings",
        title: "榜单赛程",
        route_path: "/pages/rankings/index",
        group: PageGroup::PrimaryTab,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: RANKINGS_BLOCKS,
    },
    PageDefinition {
        page_key: "insights",
        title: "洞察",
        route_path: "/pages/insights/index",
        group: PageGroup::PrimaryTab,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: INSIGHTS_BLOCKS,
    },
    PageDefinition {
        page_key: "seat_swap",
        title: "换座",
        route_path: "/pages/seat-swap/index",
        group: PageGroup::PrimaryTab,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: SEAT_SWAP_BLOCKS,
    },
    PageDefinition {
        page_key: "user",
        title: "我的",
        route_path: "/pages/user/index",
        group: PageGroup::PrimaryTab,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: USER_BLOCKS,
    },
    PageDefinition {
        page_key: "ticket_watch",
        title: "余票看板",
        route_path: "/pages/ticket-watch/index",
        group: PageGroup::Feature,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: TICKET_WATCH_BLOCKS,
    },
    PageDefinition {
        page_key: "season_recap",
        title: "赛季总结",
        route_path: "/pages/season-recap/index",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: SEASON_RECAP_BLOCKS,
    },
    PageDefinition {
        page_key: "membership_purchase",
        title: "开通会员",
        route_path: "/pages/membership-purchase/index",
        group: PageGroup::Feature,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: MEMBERSHIP_PURCHASE_BLOCKS,
    },
    PageDefinition {
        page_key: "support",
        title: "赛前助力",
        route_path: "/pages/support/index",
        group: PageGroup::Feature,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: SUPPORT_BLOCKS,
    },
    PageDefinition {
        page_key: "topics",
        title: "全部话题",
        route_path: "/pages/topics/index",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: TOPICS_BLOCKS,
    },
    PageDefinition {
        page_key: "student",
        title: "学员功能",
        route_path: "/pages/student/index",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: STUDENT_BLOCKS,
    },
    PageDefinition {
        page_key: "user_settings",
        title: "设置",
        route_path: "/pages/user/settings/index",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: USER_SETTINGS_BLOCKS,
    },
    PageDefinition {
        page_key: "team_board",
        title: "球队战术板",
        route_path: "/pages/team-board/index",
        group: PageGroup::Feature,
        default_background_image_url: Some(DEFAULT_PHOENIX_STADIUM_BG_URL),
        blocks: TEAM_BOARD_BLOCKS,
    },
    PageDefinition {
        page_key: "insights_tactics",
        title: "排阵容",
        route_path: "/pages/insights-tactics/index",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: INSIGHTS_TACTICS_BLOCKS,
    },
    PageDefinition {
        page_key: "insights_transfer",
        title: "转会表态",
        route_path: "/pages/insights-transfer/index",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: INSIGHTS_TRANSFER_BLOCKS,
    },
    PageDefinition {
        page_key: "lineup",
        title: "战术板编辑",
        route_path: "/pages/lineup/index",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: LINEUP_BLOCKS,
    },
    PageDefinition {
        page_key: "lineup_list",
        title: "球队战术板列表",
        route_path: "/pages/lineup/list",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: LINEUP_LIST_BLOCKS,
    },
    PageDefinition {
        page_key: "transfer_board",
        title: "转会榜",
        route_path: "/pages/transfer/board",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: TRANSFER_BOARD_BLOCKS,
    },
    PageDefinition {
        page_key: "director",
        title: "我来当总监",
        route_path: "/pages/director/index",
        group: PageGroup::Feature,
        default_background_image_url: None,
        blocks: DIRECTOR_BLOCKS,
    },
    PageDefinition {
        page_key: "topic_detail",
        title: "每日电报详情",
        route_path: "/pages/topic-detail/index",
        group: PageGroup::Detail,
        default_background_image_url: None,
        blocks: TOPIC_DETAIL_BLOCKS,
    },
    PageDefinition {
        page_key: "lineup_view",
        title: "球迷阵容详情",
        route_path: "/pages/lineup/view",
        group: PageGroup::Detail,
        default_background_image_url: None,
        blocks: LINEUP_VIEW_BLOCKS,
    },
    PageDefinition {
        page_key: "transfer_view",
        title: "转会提名详情",
        route_path: "/pages/transfer/view",
        group: PageGroup::Detail,
        default_background_image_url: None,
        blocks: TRANSFER_VIEW_BLOCKS,
    },
    PageDefinition {
        page_key: "telegraph_entry",
        title: "电报条目详情",
        route_path: "/pages/telegraph-entry/index",
        group: PageGroup::Detail,
        default_background_image_url: None,
        blocks: TELEGRAPH_ENTRY_BLOCKS,
    },
    PageDefinition {
        page_key: "telegraph_comment",
        title: "评论详情",
        route_path: "/pages/telegraph-entry/comment",
        group: PageGroup::Detail,
        default_background_image_url: None,
        blocks: TELEGRAPH_COMMENT_BLOCKS,
    },
    PageDefinition {
        page_key: "director_view",
        title: "总监建队方案",
        route_path: "/pages/director/view",
        group: PageGroup::Detail,
        default_background_image_url: None,
        blocks: DIRECTOR_VIEW_BLOCKS,
    },
];

pub fn find_page_definition(page_key: &str) -> Option<&'static PageDefinition> {
    PAGE_LAYOUT_CATALOG
        .iter()
        .find(|item| item.page_key == page_key)
}

pub fn normalize_optional_image_url(
    raw: Option<&str>,
    field_name: &str,
) -> Result<Option<String>, PageLayoutError> {
    let Some(value) = raw else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_IMAGE_URL_CHARS {
        return Err(PageLayoutError::InvalidInput(format!(
            "{field_name} exceeds {MAX_IMAGE_URL_CHARS} characters"
        )));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(PageLayoutError::InvalidInput(format!(
            "{field_name} must not contain whitespace"
        )));
    }
    let parsed = Url::parse(trimmed).map_err(|_| {
        PageLayoutError::InvalidInput(format!("{field_name} must be a valid http(s) URL"))
    })?;
    if !matches!(parsed.scheme(), "https" | "http") || parsed.host_str().is_none() {
        return Err(PageLayoutError::InvalidInput(format!(
            "{field_name} must use http or https scheme with a valid host"
        )));
    }
    Ok(Some(trimmed.to_string()))
}

pub fn validate_save_input(
    page_key: &str,
    input: SavePageLayoutInput,
) -> Result<ValidatedPageLayoutWrite, PageLayoutError> {
    let def = find_page_definition(page_key).ok_or(PageLayoutError::NotFound)?;
    let background_image_url =
        normalize_optional_image_url(input.background_image_url.as_deref(), "background_image_url")?;

    let valid_keys: HashSet<&'static str> = def.blocks.iter().map(|b| b.key).collect();
    let mut seen: HashSet<String> = HashSet::with_capacity(def.blocks.len());
    let mut normalized_blocks = Vec::with_capacity(def.blocks.len());

    for block in input.blocks {
        let key = block.key.trim().to_string();
        if !valid_keys.contains(key.as_str()) {
            return Err(PageLayoutError::InvalidInput(format!(
                "unknown block key '{key}' for page '{page_key}'"
            )));
        }
        if !seen.insert(key.clone()) {
            return Err(PageLayoutError::InvalidInput(format!(
                "duplicate block key '{key}' for page '{page_key}'"
            )));
        }
        let block_bg = normalize_optional_image_url(
            block.background_image_url.as_deref(),
            &format!("blocks.{key}.background_image_url"),
        )?;
        normalized_blocks.push(PageLayoutBlock {
            key,
            visible: block.visible,
            background_image_url: block_bg,
        });
    }

    // Forward-compatibility: append any catalog blocks omitted by older clients as visible defaults.
    for default_block in def.blocks {
        if !seen.contains(default_block.key) {
            normalized_blocks.push(PageLayoutBlock {
                key: default_block.key.to_string(),
                visible: true,
                background_image_url: None,
            });
        }
    }

    Ok(ValidatedPageLayoutWrite {
        page_key: def.page_key.to_string(),
        background_image_url,
        blocks: normalized_blocks,
    })
}

pub fn resolve_page_layout(
    def: &'static PageDefinition,
    stored: Option<&StoredPageLayout>,
) -> ResolvedPageLayout {
    let def_by_key: HashMap<&'static str, &'static BlockDefinition> =
        def.blocks.iter().map(|b| (b.key, b)).collect();

    let mut resolved_blocks = Vec::with_capacity(def.blocks.len());
    let mut seen: HashSet<&'static str> = HashSet::with_capacity(def.blocks.len());

    if let Some(record) = stored {
        for block in &record.blocks {
            if let Some(block_def) = def_by_key.get(block.key.as_str()).copied()
                && seen.insert(block_def.key)
            {
                let safe_bg = normalize_optional_image_url(
                    block.background_image_url.as_deref(),
                    "background_image_url",
                )
                .ok()
                .flatten();
                resolved_blocks.push(ResolvedPageLayoutBlock {
                    key: block_def.key.to_string(),
                    label: block_def.label.to_string(),
                    description: block_def.description.to_string(),
                    visible: block.visible,
                    background_image_url: safe_bg,
                });
            }
        }
    }

    for block_def in def.blocks {
        if seen.insert(block_def.key) {
            resolved_blocks.push(ResolvedPageLayoutBlock {
                key: block_def.key.to_string(),
                label: block_def.label.to_string(),
                description: block_def.description.to_string(),
                visible: true,
                background_image_url: None,
            });
        }
    }

    let safe_page_bg = match stored {
        Some(record) => record
            .background_image_url
            .as_deref()
            .and_then(|url| normalize_optional_image_url(Some(url), "background_image_url").ok())
            .flatten(),
        None => def.default_background_image_url.map(str::to_string),
    };

    ResolvedPageLayout {
        page_key: def.page_key.to_string(),
        title: def.title.to_string(),
        route_path: def.route_path.to_string(),
        group: def.group,
        background_image_url: safe_page_bg,
        blocks: resolved_blocks,
        version: stored.map(|s| s.version).unwrap_or(0),
        is_customized: stored.is_some(),
        updated_at: stored.map(|s| s.updated_at),
    }
}

impl From<ResolvedPageLayout> for PublicPageLayout {
    fn from(value: ResolvedPageLayout) -> Self {
        Self {
            page_key: value.page_key,
            background_image_url: value.background_image_url,
            blocks: value
                .blocks
                .into_iter()
                .map(|b| PageLayoutBlock {
                    key: b.key,
                    visible: b.visible,
                    background_image_url: b.background_image_url,
                })
                .collect(),
            version: value.version,
            updated_at: value.updated_at,
        }
    }
}

pub fn parse_layout_image_data_url(data_url: &str) -> Result<ParsedLayoutImage, PageLayoutError> {
    let trimmed = data_url.trim();
    let Some(rest) = trimmed.strip_prefix("data:") else {
        return Err(PageLayoutError::InvalidInput(
            "image must be a valid base64 data URL".into(),
        ));
    };
    let Some((header, encoded)) = rest.split_once(";base64,") else {
        return Err(PageLayoutError::InvalidInput(
            "image must use base64 encoding".into(),
        ));
    };
    let declared_mime = header.trim().to_ascii_lowercase();
    if !matches!(
        declared_mime.as_str(),
        "image/png" | "image/jpeg" | "image/jpg" | "image/webp" | "image/gif"
    ) {
        return Err(PageLayoutError::InvalidInput(
            "unsupported image format; only PNG, JPEG, WebP, and GIF are allowed".into(),
        ));
    }
    let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return Err(PageLayoutError::InvalidInput("image data is empty".into()));
    }
    let bytes = STANDARD
        .decode(cleaned)
        .map_err(|_| PageLayoutError::InvalidInput("invalid base64 image payload".into()))?;
    if bytes.is_empty() {
        return Err(PageLayoutError::InvalidInput("image data is empty".into()));
    }
    if bytes.len() > MAX_UPLOAD_IMAGE_BYTES {
        return Err(PageLayoutError::InvalidInput(format!(
            "image exceeds maximum size of {} MB",
            MAX_UPLOAD_IMAGE_BYTES / (1024 * 1024)
        )));
    }
    let (mime_type, ext) = detect_image_magic(&bytes).ok_or_else(|| {
        PageLayoutError::InvalidInput(
            "image bytes do not match a valid PNG, JPEG, WebP, or GIF file".into(),
        )
    })?;
    Ok(ParsedLayoutImage {
        mime_type,
        ext,
        bytes,
    })
}

fn detect_image_magic(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.len() >= 8 && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(("image/png", "png"));
    }
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some(("image/jpeg", "jpg"));
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(("image/webp", "webp"));
    }
    if bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a") {
        return Some(("image/gif", "gif"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_unique_page_keys_and_block_keys() {
        assert_eq!(PAGE_LAYOUT_CATALOG.len(), 25);
        let mut page_keys = HashSet::new();
        for page in PAGE_LAYOUT_CATALOG {
            assert!(
                page_keys.insert(page.page_key),
                "duplicate page_key {}",
                page.page_key
            );
            assert!(!page.blocks.is_empty(), "page {} has no blocks", page.page_key);
            let mut block_keys = HashSet::new();
            for block in page.blocks {
                assert!(
                    block_keys.insert(block.key),
                    "duplicate block {} in {}",
                    block.key,
                    page.page_key
                );
            }
        }
    }

    #[test]
    fn validate_save_input_preserves_order_and_appends_missing_catalog_blocks() {
        let write = validate_save_input(
            "home",
            SavePageLayoutInput {
                background_image_url: Some(" https://oryjk.cn:82/bg.webp ".into()),
                blocks: vec![
                    PageLayoutBlock {
                        key: "my_team_support".into(),
                        visible: true,
                        background_image_url: Some("https://oryjk.cn:82/card.png".into()),
                    },
                    PageLayoutBlock {
                        key: "daily_topics".into(),
                        visible: false,
                        background_image_url: Some("   ".into()),
                    },
                ],
            },
        )
        .expect("valid input");

        assert_eq!(
            write.background_image_url.as_deref(),
            Some("https://oryjk.cn:82/bg.webp")
        );
        assert_eq!(write.blocks.len(), HOME_BLOCKS.len());
        assert_eq!(write.blocks[0].key, "my_team_support");
        assert!(write.blocks[0].visible);
        assert_eq!(
            write.blocks[0].background_image_url.as_deref(),
            Some("https://oryjk.cn:82/card.png")
        );
        assert_eq!(write.blocks[1].key, "daily_topics");
        assert!(!write.blocks[1].visible);
        assert_eq!(write.blocks[1].background_image_url, None);
        // Omitted blocks default to visible=true at the end.
        assert_eq!(write.blocks[2].key, "fan_entries");
        assert!(write.blocks[2].visible);
    }

    #[test]
    fn validate_save_input_allows_hiding_all_blocks() {
        let write = validate_save_input(
            "seat_swap",
            SavePageLayoutInput {
                background_image_url: None,
                blocks: SEAT_SWAP_BLOCKS
                    .iter()
                    .map(|b| PageLayoutBlock {
                        key: b.key.into(),
                        visible: false,
                        background_image_url: None,
                    })
                    .collect(),
            },
        )
        .expect("all hidden is allowed");
        assert!(write.blocks.iter().all(|b| !b.visible));
    }

    #[test]
    fn validate_save_input_rejects_unknown_duplicate_and_bad_urls() {
        let unknown = validate_save_input(
            "home",
            SavePageLayoutInput {
                background_image_url: None,
                blocks: vec![PageLayoutBlock {
                    key: "nonexistent".into(),
                    visible: true,
                    background_image_url: None,
                }],
            },
        );
        assert!(matches!(unknown, Err(PageLayoutError::InvalidInput(_))));

        let dup = validate_save_input(
            "home",
            SavePageLayoutInput {
                background_image_url: None,
                blocks: vec![
                    PageLayoutBlock {
                        key: "daily_topics".into(),
                        visible: true,
                        background_image_url: None,
                    },
                    PageLayoutBlock {
                        key: "daily_topics".into(),
                        visible: false,
                        background_image_url: None,
                    },
                ],
            },
        );
        assert!(matches!(dup, Err(PageLayoutError::InvalidInput(_))));

        let bad_url = validate_save_input(
            "home",
            SavePageLayoutInput {
                background_image_url: Some("javascript:alert(1)".into()),
                blocks: vec![],
            },
        );
        assert!(matches!(bad_url, Err(PageLayoutError::InvalidInput(_))));

        let missing_page = validate_save_input(
            "no_such_page",
            SavePageLayoutInput {
                background_image_url: None,
                blocks: vec![],
            },
        );
        assert!(matches!(missing_page, Err(PageLayoutError::NotFound)));
    }

    #[test]
    fn resolve_page_layout_ignores_removed_keys_and_appends_new_catalog_keys() {
        let def = find_page_definition("seat_swap").unwrap();
        let stored = StoredPageLayout {
            page_key: "seat_swap".into(),
            background_image_url: Some("https://example.com/p.jpg".into()),
            blocks: vec![
                PageLayoutBlock {
                    key: "pool_list".into(),
                    visible: true,
                    background_image_url: None,
                },
                PageLayoutBlock {
                    key: "legacy_removed_block".into(),
                    visible: true,
                    background_image_url: None,
                },
            ],
            version: 3,
            updated_at: Utc::now(),
        };
        let resolved = resolve_page_layout(def, Some(&stored));
        assert_eq!(resolved.version, 3);
        assert!(resolved.is_customized);
        assert_eq!(
            resolved.blocks.iter().map(|b| b.key.as_str()).collect::<Vec<_>>(),
            vec!["pool_list", "notice", "region_map"]
        );
    }

    #[test]
    fn parse_layout_image_validates_magic_bytes_and_rejects_svg() {
        let png_bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        let data_url = format!("data:image/png;base64,{}", STANDARD.encode(png_bytes));
        let parsed = parse_layout_image_data_url(&data_url).expect("valid png");
        assert_eq!(parsed.mime_type, "image/png");
        assert_eq!(parsed.ext, "png");

        let svg = format!(
            "data:image/svg+xml;base64,{}",
            STANDARD.encode(b"<svg></svg>")
        );
        assert!(matches!(
            parse_layout_image_data_url(&svg),
            Err(PageLayoutError::InvalidInput(_))
        ));

        let spoofed = format!("data:image/png;base64,{}", STANDARD.encode(b"not a png"));
        assert!(matches!(
            parse_layout_image_data_url(&spoofed),
            Err(PageLayoutError::InvalidInput(_))
        ));
    }

    #[test]
    fn resolve_page_layout_defaults_to_builtin_bg_and_respects_explicit_clear() {
        let home_def = find_page_definition("home").expect("home");
        let default_home = resolve_page_layout(home_def, None);
        assert_eq!(
            default_home.background_image_url.as_deref(),
            Some(DEFAULT_PHOENIX_STADIUM_BG_URL)
        );
        assert!(!default_home.is_customized);

        let cleared_stored = StoredPageLayout {
            page_key: "home".into(),
            background_image_url: None,
            blocks: vec![],
            version: 1,
            updated_at: Utc::now(),
        };
        let cleared_home = resolve_page_layout(home_def, Some(&cleared_stored));
        assert_eq!(cleared_home.background_image_url, None);
        assert!(cleared_home.is_customized);

        let topics_def = find_page_definition("topics").expect("topics");
        let default_topics = resolve_page_layout(topics_def, None);
        assert_eq!(default_topics.background_image_url, None);
    }
}
