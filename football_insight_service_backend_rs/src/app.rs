use std::sync::Arc;

use axum::{Router, routing::get};
use chrono::Duration;
use sqlx::PgPool;

use crate::{
    activity::{
        adapters::{
            persistence::postgres_user_activity_repository::PostgresUserActivityRepository,
            web::{handlers::ActivityWebState, routes::activity_routes},
        },
        application::record_page_activity::RecordPageActivityUseCase,
    },
    ai::{
        adapters::{
            integration::rig_openai_chat_port::{DisabledAiChatPort, RigOpenAiChatPort},
            web::{handlers::AiWebState, routes::ai_routes},
        },
        application::chat_with_model::ChatWithModelUseCase,
        ports::ai_chat_port::AiChatPort,
    },
    auth::{
        adapters::{
            integration::current_standard_match_port::HttpCurrentStandardMatchPort,
            integration::wechat_crypto_port::OfficialWechatCryptoPort,
            integration::wechat_oauth_port::OfficialWechatOauthPort,
            persistence::postgres_auth_repository::PostgresAuthRepository,
            security::argon2_password_port::Argon2PasswordPort,
            security::jwt_token_port::JwtTokenPort,
            web::{handlers::AuthWebConfig, routes::auth_routes},
        },
        application::{
            bind_wechat_account::BindWechatAccountUseCase,
            bind_wechat_mini_program_account::BindWechatMiniProgramAccountUseCase,
            get_current_user::GetCurrentUserUseCase,
            handle_wechat_webhook::HandleWechatWebhookUseCase,
            login_with_mini_wechat::CompleteMiniWechatLoginUseCase,
            login_with_password::LoginWithPasswordUseCase,
            login_with_wechat::CompleteWechatLoginUseCase, logout::LogoutUseCase,
            register_with_invite::RegisterWithInviteUseCase,
            reset_password_with_invite::ResetPasswordWithInviteUseCase,
        },
    },
    config::AppConfig,
    health::adapters::web::routes::health_routes,
    health::{
        adapters::persistence::postgres_health_port::PostgresHealthPort,
        application::get_health::GetHealthUseCase,
    },
    insight::{
        adapters::{
            persistence::postgres_insight_query_repository::PostgresInsightQueryRepository,
            web::routes::insight_routes,
        },
        application::{
            get_live_matches::GetLiveMatchesUseCase, get_live_overview::GetLiveOverviewUseCase,
            get_live_rankings::GetLiveRankingsUseCase,
            get_live_team_insights::GetLiveTeamInsightsUseCase, get_overview::GetOverviewUseCase,
            get_round_matches::GetRoundMatchesUseCase, get_round_overview::GetRoundOverviewUseCase,
            get_round_rankings::GetRoundRankingsUseCase,
            list_available_rounds::ListAvailableRoundsUseCase,
        },
    },
    payment::{
        adapters::{
            integration::wechat_pay_port::HttpWechatPayPort,
            persistence::{
                postgres_order_repository::PostgresOrderRepository,
                postgres_payment_settlement_port::PostgresPaymentSettlementPort,
            },
            web::{handlers::PaymentWebState, routes::payment_routes},
        },
        application::{
            create_membership_order::CreateMembershipOrderUseCase,
            get_membership_product::GetMembershipProductUseCase,
            get_order_status::GetOrderStatusUseCase,
            handle_wechat_notify::HandleWechatNotifyUseCase,
        },
    },
    support::{
        adapters::{
            persistence::postgres_support_repository::PostgresSupportRepository,
            web::{handlers::SupportWebState, routes::support_routes},
        },
        application::{
            cast_match_support_vote::CastMatchSupportVoteUseCase,
            get_match_support_detail::GetMatchSupportDetailUseCase,
            get_support_profile::GetSupportProfileUseCase,
            list_support_teams::ListSupportTeamsUseCase, set_favorite_team::SetFavoriteTeamUseCase,
        },
    },
    system_config::{
        adapters::{
            persistence::{
                postgres_mini_program_review_config_port::PostgresMiniProgramReviewConfigPort,
                postgres_system_config_port::PostgresSystemConfigPort,
            },
            web::routes::system_config_routes,
        },
        application::{
            get_mini_program_review_config::GetMiniProgramReviewConfigUseCase,
            get_public_system_config::GetPublicSystemConfigUseCase,
        },
    },
    team_board::{
        adapters::{
            persistence::postgres_team_board_repository::PostgresTeamBoardRepository,
            web::{handlers::TeamBoardWebState, routes::team_board_routes},
        },
        application::{
            add_team_board_comment::AddTeamBoardCommentUseCase,
            create_team_board_post::CreateTeamBoardPostUseCase,
            get_team_board::GetTeamBoardUseCase,
            toggle_team_board_post_like::ToggleTeamBoardPostLikeUseCase,
        },
    },
    ticket_watch::{
        adapters::{
            integration::http_ticket_monitor_port::HttpTicketMonitorPort,
            integration::noop_tracked_interest_cache_port::NoopTrackedInterestCachePort,
            integration::redis_tracked_interest_cache_port::RedisTrackedInterestCachePort,
            web::{handlers::TicketWatchWebState, routes::ticket_watch_routes},
        },
        application::{
            current_board_cache::CurrentTicketWatchBoardCache,
            get_current_ticket_watch_board::GetCurrentTicketWatchBoardUseCase,
            get_current_ticket_watch_match::GetCurrentTicketWatchMatchUseCase,
            get_match_block_interests::GetMatchBlockInterestsUseCase,
            get_match_ticket_inventory::GetMatchTicketInventoryUseCase,
            get_match_tracked_interests::GetMatchTrackedInterestsUseCase,
            list_ticket_watch_matches::ListTicketWatchMatchesUseCase,
            list_ticket_watch_regions::ListTicketWatchRegionsUseCase,
            toggle_match_block_interest::ToggleMatchBlockInterestUseCase,
        },
    },
};

pub fn build_router(pool: PgPool, config: &AppConfig) -> Router {
    let insight_repository = Arc::new(PostgresInsightQueryRepository::new(pool.clone()));
    let overview_use_case = Arc::new(GetOverviewUseCase::new(insight_repository.clone()));
    let live_overview_use_case = Arc::new(GetLiveOverviewUseCase::new(insight_repository.clone()));
    let live_rankings_use_case = Arc::new(GetLiveRankingsUseCase::new(insight_repository.clone()));
    let live_team_insights_use_case =
        Arc::new(GetLiveTeamInsightsUseCase::new(insight_repository.clone()));
    let live_matches_use_case = Arc::new(GetLiveMatchesUseCase::new(insight_repository.clone()));
    let round_overview_use_case =
        Arc::new(GetRoundOverviewUseCase::new(insight_repository.clone()));
    let round_rankings_use_case =
        Arc::new(GetRoundRankingsUseCase::new(insight_repository.clone()));
    let round_matches_use_case = Arc::new(GetRoundMatchesUseCase::new(insight_repository.clone()));
    let available_rounds_use_case =
        Arc::new(ListAvailableRoundsUseCase::new(insight_repository.clone()));
    let health_use_case = Arc::new(GetHealthUseCase::new(Arc::new(PostgresHealthPort::new(
        pool.clone(),
    ))));
    let system_config_port = Arc::new(PostgresSystemConfigPort::new(pool.clone()));
    let system_config_use_case = Arc::new(GetPublicSystemConfigUseCase::new(
        system_config_port.clone(),
        insight_repository.clone(),
    ));
    let mini_program_review_config_use_case = Arc::new(GetMiniProgramReviewConfigUseCase::new(
        Arc::new(PostgresMiniProgramReviewConfigPort::new(pool.clone())),
    ));

    let auth_repository = Arc::new(PostgresAuthRepository::new(pool.clone()));
    let user_activity_repository = Arc::new(PostgresUserActivityRepository::new(pool.clone()));
    let support_repository = Arc::new(PostgresSupportRepository::new(pool.clone()));
    let team_board_repository = Arc::new(PostgresTeamBoardRepository::new(pool.clone()));
    let password_port = Arc::new(Argon2PasswordPort);
    let token_port = Arc::new(JwtTokenPort::new(config.jwt_secret.clone()));
    let wechat_oauth_port = Arc::new(OfficialWechatOauthPort::new(
        config.wechat_app_id.clone(),
        config.wechat_app_secret.clone(),
    ));
    let mini_wechat_oauth_port = Arc::new(OfficialWechatOauthPort::new(
        config.wechat_mini_app_id.clone(),
        config.wechat_mini_app_secret.clone(),
    ));
    let wechat_crypto_port = Arc::new(
        OfficialWechatCryptoPort::new(
            config.wechat_webhook_token.clone(),
            config.wechat_encoding_aes_key.clone(),
        )
        .expect("invalid wechat webhook config"),
    );
    let current_standard_match_port = Arc::new(HttpCurrentStandardMatchPort::new(
        config.ticket_monitor_base_url.clone(),
    ));
    let ticket_watch_port = Arc::new(HttpTicketMonitorPort::new(
        config.ticket_monitor_base_url.clone(),
    ));
    let tracked_interest_cache_port: Arc<
        dyn crate::ticket_watch::ports::tracked_interest_cache_port::TrackedInterestCachePort,
    > = match RedisTrackedInterestCachePort::new(&config.redis_url, 60) {
        Ok(port) => Arc::new(port),
        Err(error) => {
            tracing::warn!(error = %error, "failed to initialize tracked interest redis cache, fallback to noop");
            Arc::new(NoopTrackedInterestCachePort)
        }
    };
    let register_with_invite_use_case = Arc::new(RegisterWithInviteUseCase::new(
        auth_repository.clone(),
        password_port.clone(),
        token_port.clone(),
        Duration::days(30),
    ));
    let login_with_password_use_case = Arc::new(LoginWithPasswordUseCase::new(
        auth_repository.clone(),
        password_port,
        token_port.clone(),
        Duration::days(30),
    ));
    let reset_password_with_invite_use_case = Arc::new(ResetPasswordWithInviteUseCase::new(
        auth_repository.clone(),
        Arc::new(Argon2PasswordPort),
    ));
    let complete_wechat_login_use_case = Arc::new(CompleteWechatLoginUseCase::new(
        auth_repository.clone(),
        wechat_oauth_port,
        token_port.clone(),
        Duration::days(30),
        Duration::minutes(10),
    ));
    let complete_mini_wechat_login_use_case = Arc::new(CompleteMiniWechatLoginUseCase::new(
        auth_repository.clone(),
        mini_wechat_oauth_port,
        token_port.clone(),
        Duration::days(30),
        Duration::minutes(10),
    ));
    let bind_wechat_account_use_case = Arc::new(BindWechatAccountUseCase::new(
        auth_repository.clone(),
        Arc::new(Argon2PasswordPort),
        token_port.clone(),
        Duration::days(30),
    ));
    let bind_mini_wechat_account_use_case = Arc::new(BindWechatMiniProgramAccountUseCase::new(
        auth_repository.clone(),
        token_port.clone(),
        Duration::days(30),
    ));
    let get_current_user_use_case = Arc::new(GetCurrentUserUseCase::new(
        auth_repository.clone(),
        token_port.clone(),
    ));
    let record_page_activity_use_case = Arc::new(RecordPageActivityUseCase::new(
        user_activity_repository.clone(),
    ));
    let list_support_teams_use_case =
        Arc::new(ListSupportTeamsUseCase::new(support_repository.clone()));
    let get_support_profile_use_case =
        Arc::new(GetSupportProfileUseCase::new(support_repository.clone()));
    let set_favorite_team_use_case =
        Arc::new(SetFavoriteTeamUseCase::new(support_repository.clone()));
    let get_match_support_detail_use_case = Arc::new(GetMatchSupportDetailUseCase::new(
        support_repository.clone(),
    ));
    let cast_match_support_vote_use_case =
        Arc::new(CastMatchSupportVoteUseCase::new(support_repository));
    let ai_chat_port: Arc<dyn AiChatPort> = match config.openai_api_key.clone() {
        Some(openai_api_key) => Arc::new(RigOpenAiChatPort::new(
            openai_api_key,
            config.ai_chat_model.clone(),
            config.openai_base_url.clone(),
            system_config_port.clone(),
            insight_repository.clone(),
        )),
        None => Arc::new(DisabledAiChatPort::new()),
    };
    let chat_with_model_use_case = Arc::new(ChatWithModelUseCase::new(ai_chat_port));
    let logout_use_case = Arc::new(LogoutUseCase::new());
    let wechat_webhook_use_case = Arc::new(HandleWechatWebhookUseCase::new(
        auth_repository.clone(),
        wechat_crypto_port,
        current_standard_match_port,
        system_config_port.clone(),
    ));
    let auth_web_config = Arc::new(AuthWebConfig {
        wechat_app_id: config.wechat_app_id.clone(),
    });
    let get_team_board_use_case = Arc::new(GetTeamBoardUseCase::new(
        team_board_repository.clone(),
        insight_repository.clone(),
    ));
    let create_team_board_post_use_case = Arc::new(CreateTeamBoardPostUseCase::new(
        team_board_repository.clone(),
        insight_repository.clone(),
    ));
    let add_team_board_comment_use_case = Arc::new(AddTeamBoardCommentUseCase::new(
        team_board_repository.clone(),
    ));
    let toggle_team_board_post_like_use_case =
        Arc::new(ToggleTeamBoardPostLikeUseCase::new(team_board_repository));
    let team_board_web_state = Arc::new(TeamBoardWebState {
        get_team_board_use_case,
        create_team_board_post_use_case,
        add_team_board_comment_use_case,
        toggle_team_board_post_like_use_case,
        token_port,
    });
    let support_web_state = Arc::new(SupportWebState {
        list_support_teams_use_case,
        get_support_profile_use_case,
        set_favorite_team_use_case,
        get_match_support_detail_use_case,
        cast_match_support_vote_use_case,
        token_port: team_board_web_state.token_port.clone(),
    });
    let ticket_watch_web_state = Arc::new(TicketWatchWebState {
        get_current_ticket_watch_board_use_case: Arc::new(GetCurrentTicketWatchBoardUseCase::new(
            Arc::new(CurrentTicketWatchBoardCache::new(
                std::time::Duration::from_secs(2),
            )),
            Arc::new(GetCurrentTicketWatchMatchUseCase::new(
                ticket_watch_port.clone(),
            )),
            Arc::new(GetMatchTicketInventoryUseCase::new(
                ticket_watch_port.clone(),
            )),
            Arc::new(GetMatchBlockInterestsUseCase::new(
                ticket_watch_port.clone(),
            )),
            Arc::new(GetMatchTrackedInterestsUseCase::new(
                ticket_watch_port.clone(),
                tracked_interest_cache_port.clone(),
            )),
        )),
        get_current_ticket_watch_match_use_case: Arc::new(GetCurrentTicketWatchMatchUseCase::new(
            ticket_watch_port.clone(),
        )),
        list_ticket_watch_matches_use_case: Arc::new(ListTicketWatchMatchesUseCase::new(
            ticket_watch_port.clone(),
        )),
        list_ticket_watch_regions_use_case: Arc::new(ListTicketWatchRegionsUseCase::new(
            ticket_watch_port.clone(),
        )),
        get_match_ticket_inventory_use_case: Arc::new(GetMatchTicketInventoryUseCase::new(
            ticket_watch_port.clone(),
        )),
        get_match_block_interests_use_case: Arc::new(GetMatchBlockInterestsUseCase::new(
            ticket_watch_port.clone(),
        )),
        get_match_tracked_interests_use_case: Arc::new(GetMatchTrackedInterestsUseCase::new(
            ticket_watch_port.clone(),
            tracked_interest_cache_port.clone(),
        )),
        toggle_match_block_interest_use_case: Arc::new(ToggleMatchBlockInterestUseCase::new(
            ticket_watch_port,
            tracked_interest_cache_port,
        )),
        token_port: support_web_state.token_port.clone(),
    });
    let ai_web_state = Arc::new(AiWebState {
        chat_with_model_use_case,
        get_current_user_use_case: get_current_user_use_case.clone(),
    });
    let activity_web_state = Arc::new(ActivityWebState {
        record_page_activity_use_case,
        token_port: team_board_web_state.token_port.clone(),
    });
    let order_repository = Arc::new(PostgresOrderRepository::new(pool.clone()));
    let payment_settlement_port = Arc::new(PostgresPaymentSettlementPort::new(pool.clone()));
    let wechat_pay_port = Arc::new(HttpWechatPayPort::new(
        config.wechat_mini_app_id.clone(),
        config.wechat_pay_mch_id.clone(),
        config.wechat_pay_api_key.clone(),
        config.public_base_url.clone(),
    ));
    let user_membership_port: Arc<
        dyn crate::auth::ports::user_membership_port::UserMembershipPort,
    > = auth_repository.clone();
    let create_membership_order_use_case = Arc::new(CreateMembershipOrderUseCase::new(
        order_repository.clone(),
        user_membership_port.clone(),
        wechat_pay_port,
    ));
    let get_membership_product_use_case =
        Arc::new(GetMembershipProductUseCase::new(system_config_port.clone()));
    let get_order_status_use_case = Arc::new(GetOrderStatusUseCase::new(order_repository.clone()));
    let handle_wechat_notify_use_case = Arc::new(HandleWechatNotifyUseCase::new(
        order_repository.clone(),
        payment_settlement_port,
    ));
    let payment_web_state = Arc::new(PaymentWebState {
        create_membership_order_use_case,
        get_membership_product_use_case,
        get_order_status_use_case,
        handle_wechat_notify_use_case,
        token_port: team_board_web_state.token_port.clone(),
        user_membership_port,
        wechat_pay_api_key: config.wechat_pay_api_key.clone(),
    });

    Router::new()
        .route("/", get(|| async { "football insight service" }))
        .merge(health_routes(health_use_case))
        .merge(system_config_routes(
            system_config_use_case,
            mini_program_review_config_use_case,
        ))
        .merge(auth_routes(
            register_with_invite_use_case,
            login_with_password_use_case,
            reset_password_with_invite_use_case,
            complete_wechat_login_use_case,
            bind_wechat_account_use_case,
            complete_mini_wechat_login_use_case,
            bind_mini_wechat_account_use_case,
            get_current_user_use_case,
            logout_use_case,
            wechat_webhook_use_case,
            auth_web_config,
        ))
        .merge(activity_routes(activity_web_state))
        .merge(ai_routes(ai_web_state))
        .merge(support_routes(support_web_state))
        .merge(ticket_watch_routes(ticket_watch_web_state))
        .merge(insight_routes(
            overview_use_case,
            live_overview_use_case,
            live_rankings_use_case,
            live_team_insights_use_case,
            live_matches_use_case,
            round_overview_use_case,
            round_rankings_use_case,
            round_matches_use_case,
            available_rounds_use_case,
        ))
        .merge(team_board_routes(team_board_web_state))
        .merge(payment_routes(payment_web_state))
}
