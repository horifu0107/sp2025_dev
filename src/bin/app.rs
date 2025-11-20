use adapter::{database::connect_database_with,redis::RedisClient};
use anyhow::{Error, Result};
use api::route::{
    health::build_health_check_routers,
    space::build_space_routers,
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
    space_id: Uuid,
    space_name: String,
    created_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    bootstrap().await
}

pub async fn reminder_loop() -> Result<(), Box<dyn StdError>> {
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

    let pool = PgPoolOptions::new()
        .connect("postgresql://localhost:5432/app?user=app&password=passwd")
        .await?;

    // すでに実行済みの space_id を記録
    let mut executed: HashSet<Uuid> = HashSet::new();

    loop {
        println!("Polling database at: {}", Utc::now());

        let rows =
            sqlx::query_as::<_, Space>("SELECT space_id, space_name, created_at FROM spaces")
                .fetch_all(&pool)
                .await?;

        let now = Utc::now();

        for row in rows {
            // すでに実行済みならスキップ
            if executed.contains(&row.space_id) {
                continue;
            }

            let target_time = row.created_at + ChronoDuration::minutes(5);

            if now >= target_time {
                // 実行
                send_gmail(&access_token, row.space_id, row.created_at).await;

                // 二重実行を防ぐ
                executed.insert(row.space_id);
            }
        }

        // 10秒待機
        sleep(Duration::from_secs(10)).await;
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

    let registry = AppRegistry::new(pool, kv, app_config);

    //     tokio::spawn(async move {
    //     if let Err(e) = reminder_loop().await {
    //         eprintln!("reminder_loop error: {:?}", e);
    //     }
    // });

    let app = Router::new()
        .merge(build_health_check_routers())
        .merge(build_space_routers())
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
    created_at: DateTime<Utc>,
) -> Result<(), Box<dyn StdError>> {
    println!(
        ">>> [ACTION] Triggered for space_id={} at {}",
        space_id,
        Utc::now()
    );
    let to = "horikawa0107tokyo@gmail.com";
    let subject = format!("スペースID {} の通知", space_id);
    let body_text = format!(
        "スペースID {} が {} に作成されてから10分経ちました。",
        space_id,
        created_at.format("%Y-%m-%d %H:%M:%S")
    );

    let message_str = format!(
        "To: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{}",
        to, subject, body_text
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
