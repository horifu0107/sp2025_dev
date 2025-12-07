use crate::database::{
    model::reservation::{ReservationRow, ReservationStateRow, ReturnedReservationRow},
    ConnectionPool,
};
use async_trait::async_trait;

use derive_new::new;
use kernel::model::reservation::{
    event::{CreateReservation, UpdateReturned},
    Reservation,
};
use kernel::model::id::{SpaceId, ReservationId, UserId};
use kernel::repository::reservation::ReservationRepository;
use shared::error::{AppError, AppResult};

#[derive(new)]
pub struct ReservationRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl ReservationRepository for ReservationRepositoryImpl {
    // 予約操作を行う
    async fn create(&self, event: CreateReservation) -> AppResult<ReservationId> {
        let mut tx = self.db.begin().await?;

        // トランザクション分離レベルを SERIALIZABLE に設定する
        self.set_transaction_serializable(&mut tx).await?;

        // 事前のチェックとして、以下を調べる。
        // - 指定のスペース ID をもつスペースが存在するか
        // - 存在した場合、その時間帯は予約中ではないか
        //
        // 上記の両方が Yes だった場合、このブロック以降の処理に進む
        {
            //
        // ① スペースの存在確認 ＋ is_active チェック
        //
        let space_row = sqlx::query!(
            r#"
            SELECT space_id, is_active
            FROM spaces
            WHERE space_id = $1
            "#,
            event.space_id as _
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        let space = match space_row {
            None => {
                return Err(AppError::EntityNotFound(format!(
                    "スペース（{}）が見つかりませんでした。",
                    event.space_id
                )))
            }
            Some(s) => s,
        };

        if !space.is_active {
            return Err(AppError::UnprocessableEntity(format!(
                "スペース（{}）は現在利用できません（is_active = false）",
                event.space_id
            )));
        }

        //
        // ② 希望予約時間帯が既存予約と重なっていないか確認
        //    重複条件：
        //        existing.start < new.end AND new.start < existing.end
        //
        let overlap = sqlx::query!(
            r#"
            SELECT reservation_id
            FROM reservations
            WHERE space_id = $1
              AND reservation_start_time < $3
              AND $2 < reservation_end_time
            LIMIT 1;
            "#,
            event.space_id as _,
            event.reservation_start_time,
            event.reservation_end_time,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if overlap.is_some() {
            return Err(AppError::UnprocessableEntity(format!(
                "スペース（{}）は指定時間帯にすでに予約が存在します。",
                event.space_id
            )));
        }

        //
        // ここまでのチェックを通過すれば予約を作成する
        //
        }

        // 予約処理を行う、すなわち reservations テーブルにレコードを追加する
        let reservation_id = ReservationId::new();
        let res = sqlx::query!(
            r#"
                INSERT INTO reservations
                (reservation_id, space_id, user_id, reserved_at,
                reservation_start_time,reservation_end_time,
                reminder_at,reminder_is_already)
                VALUES ($1, $2, $3, $4,$5,$6,$7,$8)
                ;
            "#,
            reservation_id as _,
            event.space_id as _,
            event.reserved_by as _,
            event.reserved_at,
            event.reservation_start_time,
            event.reservation_end_time,
            event.reminder_at,
            event.reminder_is_already,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No reservation record has been created".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(reservation_id)
    }

    // 予約終了操作を行う
    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        // トランザクション分離レベルを SERIALIZABLE に設定する
        self.set_transaction_serializable(&mut tx).await?;

        // 予約終了操作時は事前のチェックとして、以下を調べる。
        // - 指定のスペース ID をもつスペースが存在するか
        // - 存在した場合、
        // - このスペースの予約データがあり
        // - かつ、借りたユーザーが指定のユーザーと同じか
        //
        // 上記の両方が Yes だった場合、このブロック以降の処理に進む
        // なお、ブロックの使用は意図的である。こうすることで、
        // res 変数がシャドーイングで上書きされるのを防ぐなどの
        // メリットがある。
        {
            //
            // ① スペースの存在確認
            //
            let space_row = sqlx::query!(
                r#"
                SELECT space_id
                FROM spaces
                WHERE space_id = $1
                "#,
                event.space_id as _
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
    
            if space_row.is_none() {
                return Err(AppError::EntityNotFound(format!(
                    "スペース（{}）が見つかりませんでした。",
                    event.space_id
                )));
            }
    
            //
            // ② reservation_id が存在し、space_id と一致するか
            //
            let existing_reservation = sqlx::query!(
                r#"
                SELECT reservation_id, user_id, reservation_end_time
                FROM reservations
                WHERE reservation_id = $1 AND space_id = $2
                "#,
                event.reservation_id as _,
                event.space_id as _
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
    
            let Some(res_row) = existing_reservation else {
                return Err(AppError::EntityNotFound(format!(
                    "予約（ID={}）がスペース（{}）に存在しません。",
                    event.reservation_id, event.space_id
                )));
            };
    
            //
            // ③ reservation_end_time が現在時刻より前か
            //
            // let now = chrono::Local::now();
            // if res_row.reservation_end_time > now {
            //     return Err(AppError::UnprocessableEntity(format!(
            //         "予約（ID={}）はまだ終了していません（reservation_end_time が現在より未来です）",
            //         event.reservation_id
            //     )));
            // }
        }

        // データベース上の予約終了操作として、
        // reservations テーブルにある該当予約 ID のレコードを、
        // returned_at を追加して returned_reservations テーブルに INSERT する
        // is_cancel は event.is_cancel をそのまま使用
        let res = sqlx::query!(
            r#"
                INSERT INTO returned_reservations
                (reservation_id, space_id, user_id, reserved_at, 
                returned_at,reservation_start_time,reservation_end_time,reminder_at,
                is_cancel,reminder_is_already)
                SELECT reservation_id, space_id, user_id, reserved_at, $2,
                reservation_start_time,reservation_end_time,reminder_at,$3,reminder_is_already
                FROM reservations
                WHERE reservation_id = $1
                ;
            "#,
            event.reservation_id as _,
            event.returned_at,
            event.is_cancel, 
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No returned_reservations record has been created".into(),
            ));
        }

        // 上記処理が成功したら reservations テーブルから該当予約 ID のレコードを削除する
        let delete_res = sqlx::query!(
            r#"
                DELETE FROM reservations WHERE reservation_id = $1;
            "#,
            event.reservation_id as _,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if delete_res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No reservation record has been deleted".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    // すべての未予約終了の予約情報を取得する
    async fn find_unreturned_all(&self) -> AppResult<Vec<Reservation>> {
        // reservations テーブルにあるレコードを全件抽出する
        // spaces テーブルと INNER JOIN し、スペースの情報も一緒に抽出する
        // 出力するレコードは、予約日の古い順に並べる
        sqlx::query_as!(
            ReservationRow,
            r#"
                SELECT
                r.reservation_id,
                r.space_id,
                r.user_id,
                u.user_name,
                u.email,
                r.reservation_start_time,
                r.reservation_end_time,
                r.reserved_at,
                r.reminder_at,
                r.reminder_is_already,
                s.space_name,
                s.is_active,
                s.capacity,
                s.equipment,
                s.address
                FROM reservations AS r
                INNER JOIN spaces AS s ON r.space_id = s.space_id
                INNER JOIN users AS u ON r.user_id  = u.user_id
                ORDER BY r.reserved_at ASC
                ;
            "#,
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Reservation::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    // reminder_is_alreadyを更新する
    async fn update_reminder_is_already(&self,reservation_id:ReservationId,reminder_is_already:bool) -> AppResult<()> {
        // reservations テーブルの該当データのreminder_is_alreadyを更新する
        let res = sqlx::query!(
            r#"
                UPDATE reservations
                SET
                    reminder_is_already = $1
                WHERE reservation_id = $2
            "#,
            reminder_is_already,
            reservation_id as _
            )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified reservation not found".into()));
        }

        Ok(())
    }



    // ユーザー ID に紐づく未予約終了の予約情報を取得する
    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Reservation>> {
        // find_unreturned_all の SQL に
        // ユーザー ID で絞り込む WHERE 句を追加したものである
        sqlx::query_as!(
            ReservationRow,
            r#"
                SELECT
                r.reservation_id,
                r.space_id,
                r.user_id,
                u.user_name,
                u.email,
                r.reservation_start_time,
                r.reservation_end_time,
                r.reserved_at,
                r.reminder_at,
                r.reminder_is_already,
                s.space_name,
                s.is_active,
                s.capacity,
                s.equipment,
                s.address
                FROM reservations AS r
                INNER JOIN spaces AS s ON r.space_id = s.space_id
                INNER JOIN users AS u ON r.user_id  = u.user_id
                WHERE r.user_id = $1
                ORDER BY r.reserved_at ASC
                ;
            "#,
            user_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Reservation::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    // スペースの予約履歴（予約終了済みも含む）を取得する
    async fn find_history_by_space_id(&self, space_id: SpaceId) -> AppResult<Vec<Reservation>> {
        // このメソッドでは、予約中・予約終了済みの両方を取得して
        // スペースに対する予約履歴の一覧として返す必要がある。
        // そのため、未予約終了の予約情報と予約終了済みの予約情報をそれぞれ取得し、
        // 未予約終了の予約情報があれば Vec に挿入して返す、という実装とする。
        // 未予約終了の予約情報を取得
        let reservation: Option<Reservation> = self.find_unreturned_by_space_id(space_id).await?;
        // 予約終了済みの予約情報を取得
        let mut reservation_histories: Vec<Reservation> = sqlx::query_as!(
            ReturnedReservationRow,
            r#"
                SELECT
                rr.reservation_id,
                rr.space_id,
                rr.user_id,
                u.user_name,
                u.email,
                rr.is_cancel,
                rr.reminder_is_already,
                rr.reserved_at,
                rr.reminder_at,
                rr.returned_at,
                rr.reservation_start_time,
                rr.reservation_end_time,
                s.space_name,
                s.is_active,
                s.capacity,
                s.equipment,
                s.address
                FROM returned_reservations AS rr
                INNER JOIN spaces AS s ON rr.space_id = s.space_id
                INNER JOIN users AS u ON rr.user_id = u.user_id
                WHERE s.space_id = $1
                ORDER BY rr.reserved_at DESC
            "#,
            space_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Reservation::from)
        .collect();

        // 予約中である場合は予約終了済みの履歴の先頭に追加する
        if let Some(co) = reservation {
            reservation_histories.insert(0, co);
        }

        Ok(reservation_histories)
    }

    // スペースの予約（予約終了済みを含まない）を取得する
    async fn find_reservations_by_space_id(&self, space_id: SpaceId) -> AppResult<Vec<Reservation>> {
        // このメソッドでは、予約中を取得して
        // スペースに対する予約の一覧として返す必要がある。
        let row : Vec<Reservation>= sqlx::query_as!(
            ReservationRow,
            r#"
                SELECT
                r.reservation_id,
                r.space_id,
                r.user_id,
                u.user_name,
                u.email,
                r.reservation_start_time,
                r.reservation_end_time,
                r.reserved_at,
                r.reminder_at,
                r.reminder_is_already,
                s.space_name,
                s.is_active,
                s.capacity,
                s.equipment,
                s.address
                FROM reservations AS r
                INNER JOIN spaces AS s ON r.space_id = s.space_id
                INNER JOIN users AS u ON r.user_id  = u.user_id
                WHERE r.space_id = $1
                ;
            "#,
            space_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Reservation::from)
        .collect();
        
    
        Ok(row.into())
    }

    async fn find_by_id(&self, reservation_id: ReservationId) -> AppResult<Reservation>{
        let row : Reservation= sqlx::query_as!(
            ReservationRow,
            r#"
                SELECT
                r.reservation_id,
                r.space_id,
                r.user_id,
                u.user_name,
                u.email,
                r.reservation_start_time,
                r.reservation_end_time,
                r.reserved_at,
                r.reminder_at,
                r.reminder_is_already,
                s.space_name,
                s.is_active,
                s.capacity,
                s.equipment,
                s.address
                FROM reservations AS r
                INNER JOIN spaces AS s ON r.space_id = s.space_id
                INNER JOIN users AS u ON r.user_id  = u.user_id
                WHERE r.reservation_id = $1
                ;
            "#,
            reservation_id as _
        )
        .fetch_one(self.db.inner_ref())
        .await
        .map(Reservation::from)
        .map_err(AppError::SpecificOperationError)?;
        
    
        Ok(row.into())
    }
}

impl ReservationRepositoryImpl {
    // create, update_returned メソッドでのトランザクションを利用するにあたり
    // トランザクション分離レベルを SERIALIZABLE にするために
    // 内部的に使うメソッド
    async fn set_transaction_serializable(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> AppResult<()> {
        sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut **tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    // find_history_by_space_id で未予約終了の予約情報を取得するために
    // 内部的に使うメソッド
    async fn find_unreturned_by_space_id(&self, space_id: SpaceId) -> AppResult<Option<Reservation>> {
        let res = sqlx::query_as!(
            ReservationRow,
            r#"
                SELECT
                r.reservation_id,
                r.space_id,
                r.user_id,
                u.user_name,
                u.email,
                r.reservation_start_time,
                r.reservation_end_time,
                r.reserved_at,
                r.reminder_at,
                r.reminder_is_already,
                s.space_name,
                s.is_active,
                s.capacity,
                s.equipment,
                s.address
                FROM reservations AS r
                INNER JOIN spaces AS s ON r.space_id = s.space_id
                INNER JOIN users AS u ON r.user_id  = u.user_id
                WHERE s.space_id = $1
            "#,
            space_id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .map(Reservation::from);

        Ok(res)
    }
}