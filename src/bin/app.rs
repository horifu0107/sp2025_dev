use adapter::{database::connect_database_with,redis::RedisClient};
use anyhow::{Error, Result};
use adapter::{
    database::{
        ConnectionPool,
    },
};
use api::route::{
    v1,
    auth};
use axum::Router;
use registry::AppRegistry;
use shared::config::AppConfig;
use std::error::Error as StdError;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc};
use tokio::net::TcpListener;

use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::Client;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::{collections::HashSet, time::Duration};
use tokio::time::sleep;
use uuid::Uuid;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use shared::env::{which, Environment};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use anyhow::Context;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;
#[derive(Debug, FromRow)]
struct Space {
    reservation_id: Uuid,
    is_active: bool,
    space_id: Uuid,
    user_id: Uuid,
    reminder_is_already: bool,
    reservation_start_time: DateTime<Utc>,
    reservation_end_time: DateTime<Utc>,
    reminder_at: DateTime<Utc>,
    email: String,
    user_name: String,
    space_name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    bootstrap().await
}

pub async fn reminder_loop(pool: ConnectionPool) -> Result<(), Box<dyn StdError>> {
    // --------------------------
    // Gmail OAuth2 認証
    // --------------------------
    let secret_path = "/Users/horikawafuka2/Documents/class_2025/sp/test_gmail/client_secret_483730081753-qm9ujsmkcgfpag17j2iv618fspsjpgou.apps.googleusercontent.com.json";
    let token_path = "/Users/horikawafuka2/Documents/class_2025/sp/test_gmail/token.json";

    let secret = yup_oauth2::read_application_secret(secret_path).await?;
    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::Interactive)
        .persist_tokens_to_disk(token_path)
        .build()
        .await?;

    let token = auth
        .token(&["https://www.googleapis.com/auth/gmail.send"])
        .await?;
    let access_token = token.token().unwrap().to_string();

    // let pool = PgPoolOptions::new()
    //     .connect("postgresql://localhost:5432/app?user=app&password=passwd")
    //     .await?;

    

    loop {
        println!("Polling database at: {}", Utc::now());

        let rows =
            sqlx::query_as::<_, Space>("SELECT r.reservation_id AS reservation_id,
                    r.space_id AS space_id,
                    s.is_active AS is_active,
                    r.reminder_is_already AS reminder_is_already,
                    r.reminder_at AS reminder_at,
                    r.reservation_start_time AS reservation_start_time,
                    r.reservation_end_time AS reservation_end_time,
                    u.user_id AS user_id,
                    u.email AS email,
                    u.user_name AS user_name,
                    s.space_name AS space_name
                FROM reservations AS r
                INNER JOIN users AS u 
                ON r.user_id = u.user_id
                INNER JOIN spaces AS s 
                ON r.space_id = s.space_id;
                ")
                .fetch_all(pool.inner_ref())
                .await?;

        let now = Utc::now();

        for row in rows {
            // すでに実行済みならスキップ

            if row.reminder_is_already == true || row.is_active == false{
                continue;
            }

            // let target_time = row.created_at + ChronoDuration::minutes(5);

            if now >= row.reminder_at {
                // 実行
                send_gmail(
                    &access_token, 
                    row.space_id, 
                    row.reminder_at,
                    row.space_name,
                    row.user_name,
                    row.email,
                    row.reservation_start_time,
                    row.reservation_end_time,
                ).await;

                // DB を更新（reminder_is_already = true）
            if let Err(err) = pool.mark_reminder_as_done(row.reservation_id).await {
                eprintln!("Failed to update reminder flag: {:?}", err);
            }

                
            }
        }

        // 10秒待機
        sleep(Duration::from_secs(30)).await;
    }
    // Ok(()) は到達しないが、Rust 的には必要
    #[allow(unreachable_code)]
    Ok(())
}

fn init_logger() -> Result<()> {
    let log_level = match which() {
        Environment::Development => "debug",
        Environment::Production => "info",
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());

    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        .try_init()?;

    Ok(())
}

async fn bootstrap() -> Result<()> {
    let app_config = AppConfig::new()?;
    let pool = connect_database_with(&app_config.database);
    let kv = Arc::new(RedisClient::new(&app_config.redis)?);


    // reminder_loop に渡す PgPool をクローン
    let pg_pool_for_loop = pool.clone();

    tokio::spawn(async move {
        if let Err(e) = reminder_loop(pg_pool_for_loop).await {
            eprintln!("reminder_loop error: {:?}", e);
        }
    });

    let registry = AppRegistry::new(pool, kv, app_config);



    let app = Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .with_state(registry);

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app)
        .await
        .context("Unexpected error happened in server")
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,error.message = %e, "Unexpected error"
            )
        })
}

// ----------------------------------------------
// Gmail送信処理
// ----------------------------------------------
async fn send_gmail(
    access_token: &str,
    space_id: Uuid,
    reminder_at: DateTime<Utc>,
    space_name: String,
    user_name: String,
    email: String,
    reservation_start_time: DateTime<Utc>,
    reservation_end_time: DateTime<Utc>,
) -> Result<(), Box<dyn StdError>> {
    println!(
        ">>> [ACTION] Triggered for space_id={} at {}",
        space_id,
        Utc::now()
    );
    // let to = "horikawa0107tokyo@gmail.com";
    let subject = format!(" remind mail");
    let body_text = format!(
        "{}さん {} の予約の1時間前です。予約時間：{} 〜{}",
        user_name,
        space_name,
        reservation_start_time.format("%Y-%m-%d %H:%M:%S"),
        reservation_end_time.format("%Y-%m-%d %H:%M:%S")
    );

    let message_str = format!(
        "To: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{}",
        email, subject, body_text
    );

    let encoded_message = general_purpose::URL_SAFE_NO_PAD.encode(message_str.as_bytes());

    let url = "https://gmail.googleapis.com/gmail/v1/users/me/messages/send";
    let client = Client::new();

    let res = client
        .post(url)
        .bearer_auth(access_token)
        .json(&serde_json::json!({ "raw": encoded_message }))
        .send()
        .await?;

    if res.status().is_success() {
        println!("✅ Gmail 送信成功");
    } else {
        eprintln!("❌ Gmail送信失敗: {}", res.text().await?);
    }

    Ok(())
}
