///PLCから受け取ったデータのハンドラー
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use std::collections::HashMap;
use serde_json::Value;

lazy_static! {
    static ref DB_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);
}

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
pub fn init_database() -> Result<mpsc::UnboundedSender<DbWriteRequest>> {
    let db_path = get_database_path();

    // ディレクトリが存在しない場合は作成
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let conn = Connection::open(&db_path)?;

    // データベース接続をグローバルに保存
    let mut db = DB_CONNECTION.lock().unwrap();
    *db = Some(conn);

    println!("Database initialized at: {:?}", db_path);

    // DB書き込み専用スレッドを起動し、チャネルの送信側を返す
    let tx = start_db_writer_thread();

    Ok(tx)
}

/// DB書き込み専用スレッドを起動する
/// チャネルの送信側を返すので、呼び出し側で clone して使用する
fn start_db_writer_thread() -> mpsc::UnboundedSender<DbWriteRequest> {
    let (tx, mut rx) = mpsc::unbounded_channel::<DbWriteRequest>();

    // DB書き込み専用スレッドを起動
    std::thread::spawn(move || {
        println!("DB writer thread started");

        while let Some(request) = rx.blocking_recv() {
            let table_name = format!("plc_data_{}", request.plc_id);

            let db = DB_CONNECTION.lock().unwrap();
            if let Some(conn) = db.as_ref() {
                //messageのsql文への変換処理を書く
                let recv_data: HashMap<String, Value> = serde_json::from_str(&request.message).unwrap();

                for (key,value) in &recv_data{


                }



                let insert_sql = format!(
                    "INSERT INTO {} (timestamp, message) VALUES (?1, ?2)",
                    table_name
                );

                match conn.execute(&insert_sql, [&request.timestamp, &request.message]) {
                    Ok(_) => {
                        println!(
                            "Data saved to '{}': {} - {}",
                            table_name, request.timestamp, request.message
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to save data to '{}': {}",
                            table_name, e
                        );
                    }
                }
            }
        }

        println!("DB writer thread stopped");
    });

    tx
}

/// データベースのパスを取得
fn get_database_path() -> PathBuf {
    // 実行ファイルのディレクトリにデータベースを配置
    let mut path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."));
    path.pop(); // 実行ファイル名を削除
    path.push("plc_data.db");
    path
}

/// PLC IDに基づいてテーブルを作成する
/// テーブル名: plc_data_{plc_id}
pub fn create_table_for_plc(plc_id: u32) -> Result<()> {
    let table_name = format!("plc_data_{}", plc_id);
    let sql=CREATE_TABLE_SQL.replace("{TABLE_NAME",&table_name);

    let db = DB_CONNECTION.lock().unwrap();
    if let Some(conn) = db.as_ref() {
        conn.execute(&sql, [])?;
        println!("Table '{}' created or already exists", table_name);
    }

    Ok(())
}

/// PLCから受信したデータをDB書き込みスレッドに送信する
/// 各受信タスクは独自の tx クローンを持っているので、ロック不要
pub fn save_plc_data(
    tx: &mpsc::UnboundedSender<DbWriteRequest>,
    plc_id: u32,
    timestamp: &str,
    message: &str,
) -> Result<(), String> {
    let request = DbWriteRequest {
        plc_id,
        timestamp: timestamp.to_string(),
        message: message.to_string(),
    };

    tx.send(request)
        .map_err(|e| format!("Failed to send to DB writer thread: {}", e))?;
    Ok(())
}

/// データベース接続をクローズする(アプリケーション終了時)
pub fn close_database() {
    let mut db = DB_CONNECTION.lock().unwrap();
    *db = None;
    println!("Database connection closed");
}
