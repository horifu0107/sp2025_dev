use std::net::{Ipv4Addr, SocketAddr};
use adapter::database::connect_database_with;
use anyhow::{Error, Result};
use api::route::health::build_health_check_routers;
use api::route::space::build_space_routers;
use axum::Router;
use registry::AppRegistry;
use shared::config::AppConfig;
use tokio::net::TcpListener;
use std::error::Error as StdError;

use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use std::{collections::HashSet,  time::Duration};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sqlx::{postgres::PgPoolOptions,PgPool,FromRow};
use uuid::Uuid;
use tokio::time::sleep;


#[derive(Debug, FromRow)]
struct Space {
    space_id: Uuid,
    space_name: String,
    created_at:DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    bootstrap().await
}

pub async fn reminder_loop()  ->  Result<(), Box<dyn StdError>>{
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

    let token = auth.token(&["https://www.googleapis.com/auth/gmail.send"]).await?;
    let access_token = token.token().unwrap().to_string();



    let pool = PgPoolOptions::new()
        .connect("postgresql://localhost:5432/app?user=app&password=passwd")
        .await?;

    // すでに実行済みの space_id を記録
    let mut executed: HashSet<Uuid> = HashSet::new();

    loop {
        println!("Polling database at: {}", Utc::now());

        let rows = sqlx::query_as::<_, Space>(
            "SELECT space_id, space_name, created_at FROM spaces",
        )
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
                send_gmail(&access_token,row.space_id,row.created_at).await;

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

async fn bootstrap() -> Result<()> {
    let app_config = AppConfig::new()?;
    let pool = connect_database_with(&app_config.database);

    let registry = AppRegistry::new(pool.clone());
    
    // tokio::spawn(async move {
    //     reminder_loop(pool).await;
    // });
    tokio::spawn(async move {
    if let Err(e) = reminder_loop().await {
        eprintln!("reminder_loop error: {:?}", e);
    }
});

    let app = Router::new()
        .merge(build_health_check_routers())
        .merge(build_space_routers())
        .with_state(registry);

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);
    axum::serve(listener, app).await.map_err(Error::from)
}

// ----------------------------------------------
// Gmail送信処理
// ----------------------------------------------
async fn send_gmail(
    access_token: &str,
    space_id: Uuid,
    created_at: DateTime<Utc>
) -> Result<(), Box<dyn StdError>> {
    println!(
        ">>> [ACTION] Triggered for space_id={} at {}",
        space_id, Utc::now()
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