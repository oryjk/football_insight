use std::{net::SocketAddr, time::Duration};

use axum::{
    Router,
    routing::get,
};
use football_insight_service_backend_rs::ticket_watch::adapters::integration::http_ticket_monitor_port::HttpTicketMonitorPort;
use football_insight_service_backend_rs::ticket_watch::ports::ticket_monitor_port::TicketMonitorPort;
use tokio::net::TcpListener;
use tokio::time::sleep;

#[tokio::test]
async fn fetch_all_matches_uses_schedule_id_for_inventory_and_sorts_latest_first() {
    let app = Router::new().route(
        "/api/matches/all",
        get(|| async {
            (
                [("content-type", "application/json")],
                r#"{
                    "success": true,
                    "message": "ok",
                    "matches": [
                        {
                            "id": 1,
                            "match_id": "288600",
                            "home_name": "成都蓉城",
                            "away_name": "上海申花",
                            "match_time": "2026-04-10T19:35:00",
                            "begin_date": "2026-04-10T19:35:00",
                            "round": 6,
                            "is_current": false
                        },
                        {
                            "id": 2,
                            "match_id": "bad-id",
                            "home_name": "无效场次",
                            "away_name": "跳过",
                            "match_time": "2026-04-09T19:35:00",
                            "begin_date": "2026-04-09T19:35:00",
                            "round": 5,
                            "is_current": false
                        },
                        {
                            "id": 3,
                            "match_id": "288601",
                            "home_name": "北京国安",
                            "away_name": "山东泰山",
                            "match_time": "2026-04-12T19:35:00",
                            "begin_date": "2026-04-12T19:35:00",
                            "round": 7,
                            "is_current": true
                        }
                    ]
                }"#,
            )
        }),
    );
    let base_url = spawn_test_server(app).await;
    let port = HttpTicketMonitorPort::new(Some(base_url));

    let matches = port.fetch_all_matches().await.expect("fetch matches");

    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].match_id, 3);
    assert_eq!(matches[0].external_match_id, "288601");
    assert_eq!(matches[0].home_team_name, "北京国安");
    assert_eq!(matches[1].match_id, 1);
    assert_eq!(matches[1].external_match_id, "288600");
    assert_eq!(matches[2].match_id, 2);
    assert_eq!(matches[2].external_match_id, "bad-id");
}

#[tokio::test]
async fn fetch_regions_flattens_nested_template_items() {
    let app = Router::new().route(
        "/api/match/regions-template",
        get(|| async {
            (
                [("content-type", "application/json")],
                r#"{
                    "success": true,
                    "data": {
                        "code": 0,
                        "msg": "ok",
                        "btn_status": 0,
                        "btn_text": "",
                        "max_num": 0,
                        "type_code": 0,
                        "data": [
                            {
                                "name": "看台 A",
                                "list": [
                                    { "id": 1, "name": "A1", "price": "380", "usable_count": 0, "estate": 0 },
                                    { "id": 2, "name": "A2", "price": "380", "usable_count": 0, "estate": 0 }
                                ]
                            },
                            {
                                "name": "看台 B",
                                "list": [
                                    { "id": 3, "name": "B1", "price": "580", "usable_count": 0, "estate": 0 }
                                ]
                            }
                        ]
                    }
                }"#,
            )
        }),
    );
    let base_url = spawn_test_server(app).await;
    let port = HttpTicketMonitorPort::new(Some(base_url));

    let regions = port.fetch_regions().await.expect("fetch regions");

    assert_eq!(regions.len(), 3);
    assert_eq!(regions[0].block_name, "A1");
    assert_eq!(regions[0].price, "380");
    assert_eq!(regions[2].block_name, "B1");
}

#[tokio::test]
async fn fetch_regions_allows_four_second_ticket_monitor_response() {
    let app = Router::new().route(
        "/api/match/regions-template",
        get(|| async {
            sleep(Duration::from_secs(4)).await;

            (
                [("content-type", "application/json")],
                r#"{
                    "success": true,
                    "data": {
                        "code": 0,
                        "msg": "ok",
                        "btn_status": 0,
                        "btn_text": "",
                        "max_num": 0,
                        "type_code": 0,
                        "data": [
                            {
                                "name": "看台 A",
                                "list": [
                                    { "id": 1, "name": "A1", "price": "380", "usable_count": 0, "estate": 0 }
                                ]
                            }
                        ]
                    }
                }"#,
            )
        }),
    );
    let base_url = spawn_test_server(app).await;
    let port = HttpTicketMonitorPort::new(Some(base_url));

    let regions = port.fetch_regions().await.expect("fetch delayed regions");

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].block_name, "A1");
}

async fn spawn_test_server(app: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let address: SocketAddr = listener.local_addr().expect("resolve addr");

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("serve test app");
    });

    format!("http://{}", address)
}
