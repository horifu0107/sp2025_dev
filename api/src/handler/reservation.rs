use crate::{
    extractor::AuthorizedUser,
    model::{reservation::{
        CreateReservationRequest, 
        UpdateReservationRequest,
        ReservationsResponse,
        ReservationResponse
    },
    space::UpdateSpaceRequest,
},
};
use uuid::Uuid;
use reqwest::Client;
use shared::error::AppError;
use shared::error::AppResult;
use base64::{engine::general_purpose, Engine as _};
use std::error::Error as StdError;
use chrono::{DateTime, Local};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use kernel::model::{
    reservation::event::{CreateReservation, UpdateReturned},
    id::{SpaceId, ReservationId},
};
use registry::AppRegistry;
use kernel::model::space::event::UpdateSpace;

pub async fn reservation_space(
    user: AuthorizedUser,
    Path(space_id): Path<SpaceId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateReservationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // reminder_at を予約開始1時間前へ
    let reminder_at = req.reservation_start_time - chrono::Duration::hours(1);

    let create_reservation = CreateReservation::new(
        space_id,
        user.id(),
        chrono::Local::now(),
        req.reservation_start_time,
        req.reservation_end_time,
        reminder_at,
        false,
    );

    // -------------------------
    // ① 予約作成（ここで is_active=false の場合は Err になる）
    // -------------------------
    let reservation_id=registry
        .reservation_repository()
        .create(create_reservation)
        .await?;

    // -------------------------
    // ② スペース情報を取得（is_active の確認）
    // -------------------------
    let space_opt = registry.space_repository().find_by_id(space_id).await?;
    let space = space_opt.ok_or_else(|| {
        AppError::ExternalServiceError("Space not found".to_string())
    })?;

    // -------------------------
    // ★ is_active = false なら Gmail を送らず終了
    // -------------------------
    if !space.is_active {
        return Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "reservation_id": reservation_id.to_string(),
                "reminder_sent": false
            }))
        )); 
    }


    
    // -------------------------
    // ③ Gmail送信条件を満たしているか確認
    // -------------------------
    let now = chrono::Local::now();
    if now >= reminder_at {
        return Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "reservation_id": reservation_id.to_string(),
                "reminder_sent": false
            }))
        )); // 時間が過ぎているなら送らない
    }

    // -------------------------
    // ★ ユーザー情報取得
    // -------------------------
    let user_opt = registry
        .user_repository()
        .find_current_user(user.id())
        .await?;
    let user_model = user_opt.ok_or_else(|| {
        AppError::ExternalServiceError("User not found".to_string())
    })?;

    // -------------------------
    // Gmail 送信
    // -------------------------
    let access_token = registry.google_access_token().await?;
    send_reminder_gmail(
        &access_token,
        space_id,
        reminder_at,
        space.space_name,
        user_model.user_name,
        user_model.email,
        req.reservation_start_time,
        req.reservation_end_time,
    )
    .await
    .map_err(|e| AppError::ExternalServiceError(format!("Gmail error: {e}")))?;



    // -------------------------
    // ③ Gmail が成功したら reminder_is_already=true に
    // -------------------------
    registry
        .reservation_repository()
        .update_reminder_is_already(reservation_id,true)
        .await?;

    // Ok(StatusCode::CREATED
    Ok((StatusCode::CREATED, axum::Json(serde_json::json!({
    "reservation_id": reservation_id.to_string()
}))))

}

pub async fn return_space(
    user: AuthorizedUser,
    Path((space_id, reservation_id)): Path<(SpaceId, ReservationId)>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {

    // ① 予約情報を DB から取得
    let reservation = registry
        .reservation_repository()
        .find_by_id(reservation_id)
        .await?;   // Reservation を返す想定
    
    let update_returned = UpdateReturned::new(
        reservation_id, 
        space_id, 
        user.id(), 
        true,
        chrono::Local::now(),
        reservation.reservation_start_time,
        reservation.reservation_end_time,
        reservation.reminder_at,
    );

    registry
        .reservation_repository()
        .update_returned(update_returned)
        .await?;
    

    let access_token = registry.google_access_token().await?;

    // ④ Gmail を送信
    send_cancel_gmail(
        &access_token,
        space_id,
        reservation.reminder_at,
        reservation.space.space_name,
        reservation.user_name,
        reservation.email,
        reservation.reservation_start_time,
        reservation.reservation_end_time,
    )
    .await
    .map_err(|e| AppError::ExternalServiceError(format!("Gmail error: {e}")))?;

    Ok(StatusCode::OK)
    
}

pub async fn cancel_space(
    user: AuthorizedUser,
    Path(space_id): Path<SpaceId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let update_event = UpdateSpace {
    space_id,
    requested_user: user.id(),  // ログインユーザID
    space_name: None,
    is_active: Some(false),   // ★ ここを false に変更
    description: None,
    capacity: None,
    equipment: None,
    address: None,
};

    // spacesテーブルの該当データ（space_idから取得）のis_activeをfalseに更新
    registry.space_repository().update_is_active(update_event).await?;   


    // ① 予約情報をスペースIDをもとに DB から取得
    let reservations = registry
        .reservation_repository()
        .find_reservations_by_space_id(space_id)
        .await?;   // Reservation を返す想定
    
    // 予約が無い場合は OK を返す（何も送信しない）
    if reservations.is_empty() {
        return Ok(StatusCode::OK);
    }

    let access_token = registry.google_access_token().await?;

    for reservation in reservations {
        let reservation_id = reservation.reservation_id;

        let update_canceled = UpdateReturned::new(
            reservation_id, 
            space_id, 
            user.id(), 
            true,
            chrono::Local::now(),
            reservation.reservation_start_time,
            reservation.reservation_end_time,
            reservation.reminder_at,
        );
    
        registry
            .reservation_repository()
            .update_returned(update_canceled)
            .await?;
        
    
    
        // ④ Gmail を送信
        send_cancel_gmail(
            &access_token,
            space_id,
            reservation.reminder_at,
            reservation.space.space_name,
            reservation.user_name,
            reservation.email,
            reservation.reservation_start_time,
            reservation.reservation_end_time,
        )
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Gmail error: {e}")))?;
    }
Ok(StatusCode::OK)
}

pub async fn stop_reservation_service(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
//すべてのスペース情報を取得する
    let all_space_info=registry
        .space_repository()
        .find_all_space_for_all_cancel()
        .await?;

    // スペースが無い場合は OK を返す（何も送信しない）
    if all_space_info.is_empty() {
        return Ok(StatusCode::OK);
    }
    

    for space in all_space_info{
        let space_id = space.space_id;
    
        let update_event = UpdateSpace {
        space_id,
        requested_user: user.id(),  // ログインユーザID
        space_name: None,
        is_active: Some(false),   // ★ ここを false に変更
        description: None,
        capacity: None,
        equipment: None,
        address: None,
    };
    
        // spacesテーブルの該当データ（space_idから取得）のis_activeをfalseに更新
        registry.space_repository().update_is_active(update_event).await?;           
    }
Ok(StatusCode::OK)
}


pub async fn cancel_all_reservation(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
//すべてのスペース情報を取得する
    let all_space_info=registry
        .space_repository()
        .find_all_space_for_all_cancel()
        .await?;

    // スペースが無い場合は OK を返す（何も送信しない）
    if all_space_info.is_empty() {
        return Ok(StatusCode::OK);
    }
    

    for space in all_space_info{
        let space_id = space.space_id;
    
        // ① 予約情報をスペースIDをもとに DB から取得
        let reservations = registry
            .reservation_repository()
            .find_reservations_by_space_id(space_id)
            .await?;   // Reservation を返す想定
        
        // 予約がないなら Gmail 送信はスキップ
        if reservations.is_empty() {
            continue;   // ★ return は絶対ダメ。ループ継続
        }
    
        let access_token = registry.google_access_token().await?;
    
        for reservation in reservations {
            let reservation_id = reservation.reservation_id;
    
            let update_canceled = UpdateReturned::new(
                reservation_id, 
                space_id, 
                user.id(), 
                true,
                chrono::Local::now(),
                reservation.reservation_start_time,
                reservation.reservation_end_time,
                reservation.reminder_at,
            );
        
            registry
                .reservation_repository()
                .update_returned(update_canceled)
                .await?;
    
        
            // ④ Gmail を送信
            send_cancel_gmail(
                &access_token,
                space_id,
                reservation.reminder_at,
                reservation.space.space_name,
                reservation.user_name,
                reservation.email,
                reservation.reservation_start_time,
                reservation.reservation_end_time,
            )
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Gmail error: {e}")))?;
        }
    }
Ok(StatusCode::OK)
}


pub async fn show_reserved_list(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<ReservationsResponse>> {
    registry
        .reservation_repository()
        .find_unreturned_all()
        .await
        .map(ReservationsResponse::from)
        .map(Json)
}

pub async fn return_reservation_by_id(
    _user: AuthorizedUser,
    Path(reservation_id): Path<ReservationId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<ReservationResponse>> {
    registry
        .reservation_repository()
        .find_by_id(reservation_id)
        .await
        .map(ReservationResponse::from)
        .map(Json)
}

pub async fn reservation_history(
    _user: AuthorizedUser,
    Path(space_id): Path<SpaceId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<ReservationsResponse>> {
    registry
        .reservation_repository()
        .find_history_by_space_id(space_id)
        .await
        .map(ReservationsResponse::from)
        .map(Json)
}


// ----------------------------------------------
// リマインダーGmail送信処理
// ----------------------------------------------
async fn send_reminder_gmail(
    access_token: &str,
    space_id: SpaceId,
    reminder_at: DateTime<Local>,
    space_name: String,
    user_name: String,
    email: String,
    reservation_start_time: DateTime<Local>,
    reservation_end_time: DateTime<Local>,
) -> Result<(), Box<dyn StdError>> {
    println!(
        ">>> [ACTION] Triggered for space_id={} at {}",
        space_id,
        Local::now()
    );

    // --------------------------------------------
    // 🔸 現在時刻との差分を計算
    // --------------------------------------------
    let now = Local::now();

    let diff = reservation_start_time.signed_duration_since(now);

    // すでに開始時刻を過ぎている
    if diff.num_seconds() < 0 {
        println!("すでに予約開始時刻を過ぎているため、リマインダーを送信しません。");
        return Ok(());
    }

    // chrono::Duration → std::time::Duration
    let std_dur = diff
        .to_std()
        .unwrap_or(std::time::Duration::from_secs(0));

    // 秒 → 分へ変換
    let minutes = std_dur.as_secs() / 60;

    // 「◯時間前」「◯分前」などの表記を作成
    let before_text = if minutes >= 60 {
        format!("{}時間{}分前", minutes / 60, minutes % 60)
    } else {
        format!("{}分前", minutes)
    };

    // --------------------------------------------
    // 🔸 Gmail文面
    // --------------------------------------------
    let subject = format!("remind mail");
    let body_text = format!(
        "{}さん\n{}の予約の{}です。\nリマインダー時刻：{}\n予約時間：{} 〜 {}",
        user_name,
        space_name,
        before_text, // ← ここに動的な残り時間が入る！
        reminder_at.format("%Y-%m-%d %H:%M:%S"),
        reservation_start_time.format("%Y-%m-%d %H:%M:%S"),
        reservation_end_time.format("%Y-%m-%d %H:%M:%S")
    );

    let message_str = format!(
        "To: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{}",
        email, subject, body_text
    );

    let encoded_message =
        general_purpose::URL_SAFE_NO_PAD.encode(message_str.as_bytes());

    let url = "https://gmail.googleapis.com/gmail/v1/users/me/messages/send";
    let client = Client::new();

    let res = client
        .post(url)
        .bearer_auth(access_token)
        .json(&serde_json::json!({ "raw": encoded_message }))
        .send()
        .await?;

    if res.status().is_success() {
        println!("✅ リマインダーGmail 送信成功");
    } else {
        eprintln!("❌ リマインダーGmail送信失敗: {}", res.text().await?);
    }

    Ok(())
}

// ----------------------------------------------
// キャンセルGmail送信処理
// ----------------------------------------------
async fn send_cancel_gmail(
    access_token: &str,
    space_id: SpaceId,
    reminder_at: DateTime<Local>,
    space_name: String,
    user_name: String,
    email: String,
    reservation_start_time: DateTime<Local>,
    reservation_end_time: DateTime<Local>,
) -> Result<(), Box<dyn StdError>> {
    println!(
        ">>> [ACTION] Triggered for space_id={} at {}",
        space_id,
        Local::now()
    );
    // let to = "horikawa0107tokyo@gmail.com";
    let subject = format!("cancel mail");
    let body_text = format!(
        "{}さん \n{}が停電により使えなくなりました。\n
        ご予約はキャンセルになります。\nリマインダー時刻：{}\n予約時間：{} 〜{}",
        user_name,
        space_name,
        reminder_at,
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
        println!("✅ キャンセルGmail 送信成功");
    } else {
        eprintln!("❌ キャンセルGmail送信失敗: {}", res.text().await?);
    }

    Ok(())
}
