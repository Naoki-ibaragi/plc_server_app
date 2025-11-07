///PLCから受け取ったデータのハンドラー
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::sync::mpsc;

lazy_static! {
    static ref DB_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);
    static ref DB_WRITER_CHANNEL: Mutex<Option<mpsc::UnboundedSender<DbWriteRequest>>> = Mutex::new(None);
}

/// DB書き込みリクエストの構造体
#[derive(Debug, Clone)]
pub struct DbWriteRequest {
    pub plc_id: u32,
    pub timestamp: String,
    pub message: String,
}

/// データベースを初期化し、DB書き込み専用スレッドを起動する
pub fn init_database() -> Result<()> {
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

    // DB書き込み専用スレッドを起動
    start_db_writer_thread();

    Ok(())
}

/// DB書き込み専用スレッドを起動する
fn start_db_writer_thread() {
    let (tx, mut rx) = mpsc::unbounded_channel::<DbWriteRequest>();

    // チャネルの送信側をグローバルに保存
    {
        let mut channel = DB_WRITER_CHANNEL.lock().unwrap();
        *channel = Some(tx);
    }

    // DB書き込み専用スレッドを起動
    std::thread::spawn(move || {
        println!("DB writer thread started");

        while let Some(request) = rx.blocking_recv() {
            let table_name = format!("plc_data_{}", request.plc_id);

            let db = DB_CONNECTION.lock().unwrap();
            if let Some(conn) = db.as_ref() {
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

    let db = DB_CONNECTION.lock().unwrap();
    if let Some(conn) = db.as_ref() {
        let create_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            table_name
        );

        conn.execute(&create_table_sql, [])?;
        println!("Table '{}' created or already exists", table_name);
    }

    Ok(())
}

/// PLCから受信したデータをDB書き込みスレッドに送信する
pub fn save_plc_data(plc_id: u32, timestamp: &str, message: &str) -> Result<(), String> {
    let channel = DB_WRITER_CHANNEL.lock().unwrap();

    if let Some(tx) = channel.as_ref() {
        let request = DbWriteRequest {
            plc_id,
            timestamp: timestamp.to_string(),
            message: message.to_string(),
        };

        tx.send(request).map_err(|e| format!("Failed to send to DB writer thread: {}", e))?;
        Ok(())
    } else {
        Err("DB writer channel not initialized".to_string())
    }
}

/// データベース接続をクローズする(アプリケーション終了時)
pub fn close_database() {
    let mut db = DB_CONNECTION.lock().unwrap();
    *db = None;
    println!("Database connection closed");
}
