use super::{
    adapters::{
        persistence::PostgresStudentRepository,
        web::{self, AdminStudentWebState, StudentWebState},
    },
    application::{StudentConfigService, StudentService},
};
use crate::{
    admin::application::admin_auth_service::AdminAuthService,
    auth::ports::{token_port::TokenPort, user_membership_port::UserMembershipPort},
    payment::ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
pub fn build_student_routes(
    pool: PgPool,
    orders: Arc<dyn OrderRepository>,
    users: Arc<dyn UserMembershipPort>,
    pay: Arc<dyn WechatPayPort>,
    tokens: Arc<dyn TokenPort>,
) -> Router {
    web::routes(StudentWebState {
        service: Arc::new(StudentService::new(
            Arc::new(PostgresStudentRepository::new(pool)),
            orders,
            users,
            pay,
        )),
        tokens,
    })
}
pub fn build_admin_student_routes(pool: PgPool, auth: Arc<AdminAuthService>) -> Router {
    web::admin_routes(AdminStudentWebState {
        service: Arc::new(StudentConfigService::new(Arc::new(
            PostgresStudentRepository::new(pool),
        ))),
        auth,
    })
}
