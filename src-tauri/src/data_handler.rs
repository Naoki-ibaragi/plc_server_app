///PLCから受け取ったデータのハンドラー
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use parking_lot::Mutex;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use std::collections::HashMap;
use serde_json::Value;
use std::env;
use chrono::NaiveDateTime;

use crate::regist_data_to_db::*;

lazy_static! {
    static ref DB_POOL: Mutex<Option<Pool<Postgres>>> = Mutex::new(None);
}

//テーブルを作成するためのsql文を読み込み
static CREATE_TABLE_SQL:&str = include_str!("sql/create_table.sql");

/// DB書き込みリクエストの構造体
#[derive(Debug, Clone)]
pub struct DbWriteRequest {
    pub plc_id: u32,
    pub timestamp: String,
    pub message: String,
}

/// データベースを初期化し、DB書き込み専用スレッドを起動する
/// チャネルの送信側を返すので、各スレッドで clone して使用する
pub async fn init_database() -> Result<mpsc::UnboundedSender<DbWriteRequest>, sqlx::Error> {
    // 環境変数からPostgreSQL接続文字列を取得
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            log::warn!("DATABASE_URL not set, using default connection string");
            "postgresql://postgres:password@localhost:5432/chiptest".to_string()
        });

    // 接続プールを作成（最大接続数: 5）
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    log::info!("PostgreSQL connection pool initialized");

    // 接続プールをグローバルに保存
    let mut db_pool = DB_POOL.lock();
    *db_pool = Some(pool);

    // DB書き込み専用スレッドを起動し、チャネルの送信側を返す
    let tx = start_db_writer_thread();

    Ok(tx)
}

/// DB書き込み専用タスクを起動する（非同期）
/// チャネルの送信側を返すので、呼び出し側で clone して使用する
fn start_db_writer_thread() -> mpsc::UnboundedSender<DbWriteRequest> {
    let (tx, mut rx) = mpsc::unbounded_channel::<DbWriteRequest>();

    // DB書き込み専用の非同期タスクを起動
    tokio::spawn(async move {
        log::info!("DB writer async task started");

        //各装置ID毎に現在のロット番号の各シリアル番号に対してld_pickup_dateを管理する
        //各装置のロット番号が切り替わったタイミングでそのキーを破棄して、新規で作成する
        //[装置id][lot番号][serial]=ld_pickup_date
        let mut manage_ld_pickup_date:HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>=HashMap::new();

        while let Some(request) = rx.recv().await {
            // 受信データをログ出力
            log::info!(
                "Received PLC data - ID: {}, Size: {} bytes",
                request.plc_id,
                request.message.len()
            );
            log::info!("PLC data content: {}", request.message);

            // 接続プールから接続を取得
            let pool = {
                let db_pool = DB_POOL.lock();
                match db_pool.as_ref() {
                    Some(p) => p.clone(),
                    None => {
                        log::error!("DB pool not available for PLC ID: {}", request.plc_id);
                        continue;
                    }
                }
            };

            // PLCから受信したjson形式データをhashmapに変換する
            let recv_data: HashMap<String, Value> = match serde_json::from_str(&request.message) {
                Ok(data) => data,
                Err(e) => {
                    log::error!("Failed to parse JSON data: {}", e);
                    continue;
                }
            };
                
            // ロット番号の取り出し
            let lot_name = recv_data
                .get("LOT")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // 機種名の取り出し
            let type_name = recv_data
                .get("TYPE")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // 装置名の取り出し
            let machine_id = recv_data
                .get("MACHINE")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            // トランザクション開始
            let mut tx = match pool.begin().await {
                Ok(transaction) => transaction,
                Err(e) => {
                    log::error!("Failed to begin transaction: {}", e);
                    continue;
                }
            };

            // U1_TRを含むキーを優先的に処理（ld_pickup_dateの登録のため）
            for (key, value) in &recv_data {
                if key.contains("U1_TR") {
                    let result = regist_u1_tr_info(&mut tx, machine_id, lot_name, type_name, value,&mut manage_ld_pickup_date).await;
                    if let Err(e) = result {
                        log::error!("Failed to register data for key '{}': {}", key, e);
                    }
                }
            }

            // 各ユニット情報の取り出しと登録
            for (key, value) in &recv_data {
                let result = if key.contains("U1_TR") {
                    // すでに処理済みなのでスキップ
                    continue;
                } else if key.contains("_A1_") && !key.contains("U1") && !key.contains("U7") && !key.contains("U2"){
                    // 上流アームコレットの使用回数データを登録
                    let unit_name = match key.split('_').next() {
                        Some(v) => v,
                        None => continue,
                    };
                    regist_arm1_info(&mut tx, machine_id, lot_name, type_name, unit_name, value, &manage_ld_pickup_date).await
                } else if key.contains("_A1_") && key.contains("U1"){
                    //LDのアーム1にはwano,wax,way情報が入っているので分ける
                    regist_ld_arm1_info(&mut tx, machine_id, lot_name, type_name, value, &manage_ld_pickup_date).await
                }else if key.contains("_A1_") && key.contains("U7"){
                    //ULDのアーム1でpx,py,ax,ay,at,pax,pay全て一気に登録する
                    regist_uld_arm1_info(&mut tx, machine_id, lot_name, type_name, value, &manage_ld_pickup_date).await
                }else if key.contains("_A1_") && key.contains("U2"){
                    //DC1のアーム1で予熱テーブルの補正量も登録するように変更
                    regist_dc1_arm1_info(&mut tx, machine_id, lot_name, type_name, value, &manage_ld_pickup_date).await
                }else if key.contains("_A2_") {
                    // 下流アームコレットの使用回数データを登録
                    let unit_name = match key.split('_').next() {
                        Some(v) => v,
                        None => continue,
                    };
                    regist_arm2_info(&mut tx, machine_id, lot_name, type_name, unit_name, value, &manage_ld_pickup_date).await
                } else if key.contains("_PH_") {
                    // DC1,ULD予熱テーブルのデータを登録
                    let unit_name = match key.split('_').next() {
                        Some(v) => v,
                        None => continue,
                    };
                    regist_ph_info(&mut tx, machine_id, lot_name, type_name, unit_name, value, &manage_ld_pickup_date).await
                } else if key.contains("_TS_") && !key.contains("U6") {
                    // DC1~DC2検査テーブルのデータを登録
                    let unit_name = match key.split('_').next() {
                        Some(v) => v,
                        None => continue,
                    };
                    regist_ts_info(&mut tx, machine_id, lot_name, type_name, unit_name, value, &manage_ld_pickup_date).await
                } else if key.contains("_TS_") && key.contains("U6") {
                    // IP検査テーブルのデータを登録
                    regist_ip_ts_info(&mut tx, machine_id, lot_name, type_name, value, &manage_ld_pickup_date).await
                } else if key.contains("U6_T1_") {
                    // IP表面検査のBINデータを登録
                    regist_ip_surf_info(&mut tx, machine_id, lot_name, type_name, value, &manage_ld_pickup_date).await
                } else if key.contains("U6_T2_") {
                    // IP裏面検査のBINデータを登録
                    regist_ip_back_info(&mut tx, machine_id, lot_name, type_name, value, &manage_ld_pickup_date).await
                } else if key.contains("U7_PI_") {
                    // ULDポケット認識時のデータを登録
                    regist_uld_pocket_info(&mut tx, machine_id, lot_name, type_name, value, &manage_ld_pickup_date).await
                } else if key.contains("U7_CI_") {
                    // ULDポケット挿入時のデータを登録
                    regist_uld_chip_info(&mut tx, machine_id, lot_name, type_name, value, &manage_ld_pickup_date).await
                } else if key.contains("_AL_") {
                    // アラーム情報の登録
                    let unit_name = match key.split('_').next() {
                        Some(v) => v,
                        None => continue,
                    };
                    regist_alarm_info(&mut tx, machine_id, lot_name, type_name, unit_name, value, &manage_ld_pickup_date).await
                } else {
                    continue;
                };

                if let Err(e) = result {
                    log::error!("Failed to register data for key '{}': {}", key, e);
                }
            }

            // トランザクションコミット
            match tx.commit().await {
                Ok(_) => {
                    log::info!("DB write completed for PLC ID: {}", request.plc_id);
                }
                Err(e) => {
                    log::error!("Failed to commit transaction: {}", e);
                }
            }
        }

        log::warn!("DB writer async task stopped");
    });

    tx
}


/// CHIPDATAパーティションテーブルを作成する（初回のみ実行）
/// パーティションテーブル名: chipdata（固定）
pub async fn create_chipdata_table() -> Result<(), sqlx::Error> {
    let pool = {
        let db_pool = DB_POOL.lock();
        db_pool.as_ref().map(|p| p.clone())
    };

    if let Some(pool) = pool {
        // SQL文を分割して実行（PostgreSQLは複数コマンドを同時実行できないため）
        let mut statements = Vec::new();
        let mut current_statement = String::new();

        for line in CREATE_TABLE_SQL.lines() {
            let trimmed = line.trim();

            // 空行やコメント行のみはスキップ
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            current_statement.push_str(line);
            current_statement.push('\n');

            // セミコロンで終わる行があれば、それを1つの文として保存
            if trimmed.ends_with(';') {
                let stmt = current_statement.trim().trim_end_matches(';').trim();
                if !stmt.is_empty() {
                    statements.push(stmt.to_string());
                }
                current_statement.clear();
            }
        }

        // 各SQL文を順次実行
        for (i, statement) in statements.iter().enumerate() {
            log::debug!("Executing SQL statement {}: {}", i + 1, &statement[..statement.len().min(100)]);
            sqlx::query(statement)
                .execute(&pool)
                .await?;
        }

        log::info!("CHIPDATA partition table and indexes created or already exist");
    }
    Ok(())
}

/// 指定された年月のパーティションを作成する
/// 例: 2025年1月 → chipdata_2025_01
pub async fn create_partition_for_month(year: i32, month: u32) -> Result<(), sqlx::Error> {
    let pool = {
        let db_pool = DB_POOL.lock();
        db_pool.as_ref().map(|p| p.clone())
    };

    if let Some(pool) = pool {
        let partition_name = format!("chipdata_{}_{:02}", year, month);

        // 次の月の開始日を計算
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} PARTITION OF chipdata FOR VALUES FROM ('{}-{:02}-01') TO ('{}-{:02}-01')",
            partition_name, year, month, next_year, next_month
        );

        sqlx::query(&sql)
            .execute(&pool)
            .await?;

        log::info!("Partition '{}' created for {}/{}", partition_name, year, month);
    }
    Ok(())
}

/// 現在の年月と次月のパーティションを自動作成
pub async fn ensure_current_partitions() -> Result<(), sqlx::Error> {
    use chrono::{Datelike, Local};

    let now = Local::now();
    let current_year = now.year();
    let current_month = now.month();

    // 現在の月のパーティション作成
    create_partition_for_month(current_year, current_month).await?;

    // 次月のパーティション作成（月末の書き込みエラーを防ぐ）
    let next_month = if current_month == 12 { 1 } else { current_month + 1 };
    let next_year = if current_month == 12 { current_year + 1 } else { current_year };
    create_partition_for_month(next_year, next_month).await?;

    Ok(())
}

/// PLCから受信したデータをDB書き込みスレッドに送信する
pub fn save_plc_data(
    tx: &mpsc::UnboundedSender<DbWriteRequest>,
    plc_id: u32,
    timestamp: &str,
    message: &str,
) -> Result<(), String> {
    let request = DbWriteRequest {
        plc_id:plc_id,
        timestamp: timestamp.to_string(),
        message: message.to_string(),
    };

    tx.send(request)
        .map_err(|e| format!("Failed to send to DB writer thread: {}", e))?;
    Ok(())
}

/// データベース接続プールをクローズする(アプリケーション終了時)
pub async fn close_database() {
    let mut db_pool = DB_POOL.lock();
    if let Some(pool) = (*db_pool).take() {
        pool.close().await;
        log::info!("Database connection pool closed");
    }
}
