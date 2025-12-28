use tauri::{AppHandle, command,Emitter};
use tokio::net::{TcpStream, TcpListener};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use crate::types::PlcConnection;
use crate::state::{ConnectionState, DbChannelState};
use chrono::{DateTime, Utc};
use crate::data_handler::{create_chipdata_table, ensure_current_partitions, save_plc_data};

/// PLCに接続する(フロントエンドから呼び出し)
#[command]
pub async fn connect_plc(
    plc_id: u32,
    plc_ip: String,
    plc_port: u16,
    pc_ip: String,
    state: tauri::State<'_, ConnectionState>,
    db_channel: tauri::State<'_, DbChannelState>,
    app: AppHandle,
) -> Result<String, String> {
     log::info!("Connecting to PLC ID: {}, IP: {}:{}", plc_id, plc_ip, plc_port);

    // 既に接続されているかチェック
    {
        let connections = state.lock();
        if let Some(conn) = connections.get(&plc_id) {
            if conn.is_connected {
                log::warn!("Already connected");
                return Err("Already connected".to_string());
            }
        }
    }

    // PCのポートでリッスンを開始（ポート番号0で自動割り当て）
    let listen_addr = format!("{}:0", pc_ip);
    log::info!("Trying to listen on: {} (auto-assign port)", listen_addr);

    let listener = TcpListener::bind(&listen_addr)
        .await
        .map_err(|e| {
            log::error!("Failed to bind to {}: {}", listen_addr, e);
            format!("Failed to bind to {}: {}", listen_addr, e)
        })?;

    let local_addr = listener.local_addr()
        .map_err(|e| { 
            log::error!("Failed to get local address: {}", e);
            format!("Failed to get local address: {}", e)
        })?;
    let pc_port = local_addr.port();

    log::info!("Listening on {}:{}", pc_ip, pc_port);

    // PLCに接続を試みる（接続先として）
    let plc_addr = format!("{}:{}", plc_ip, plc_port);
    log::info!("Trying to connect to PLC at: {}", plc_addr);

    let stream = TcpStream::connect(&plc_addr)
        .await
        .map_err(|e| format!("Failed to connect to PLC at {}: {}", plc_addr, e))?;

    log::info!("Connected to PLC at {}", plc_addr);

    // 接続情報を保存
    {
        let mut connections = state.lock();
        connections.insert(
            plc_id,
            PlcConnection {
                plc_id,
                plc_ip: plc_ip.clone(),
                plc_port,
                pc_ip: pc_ip.clone(),
                is_connected: true,
            },
        );
    }

    // CHIPDATAパーティションテーブルを作成（初回のみ）
    if let Err(e) = create_chipdata_table().await {
        log::error!("Failed to create CHIPDATA table: {}", e);
    }

    // 現在月と次月のパーティションを確保
    if let Err(e) = ensure_current_partitions().await {
        log::error!("Failed to ensure partitions: {}", e);
    }

    // 受信ループを別のタスクで実行
    // DB チャネルをクローンして渡す（ロックフリー）
    let state_clone = Arc::clone(&state.inner());
    let db_tx = db_channel.inner().clone();
    tokio::spawn(async move {
        receive_data_from_plc(plc_id, stream, state_clone, db_tx, app).await;
    });

    Ok(format!("Connected to PLC {}:{}", plc_ip, plc_port))
}

/// PLCからデータを受信する
async fn receive_data_from_plc(
    plc_id: u32,
    mut stream: TcpStream,
    state: ConnectionState,
    db_tx: DbChannelState,
    app: AppHandle,
) {
    log::info!("Starting receive loop for PLC ID: {}", plc_id);
    let mut buffer = vec![0u8; 4096];
    let mut incomplete_data = String::new(); // 未完了データを蓄積するバッファ

    loop {
        // 接続状態をチェック
        {
            let connections = state.lock();
            if let Some(conn) = connections.get(&plc_id) {
                if !conn.is_connected {
                    log::warn!("PLC ID {} is disconnected, stopping receive loop", plc_id);
                    break;
                }
            } else {
                log::error!("PLC ID {} not found in state, stopping receive loop", plc_id);
                break;
            }
        }

        // データを受信
        match stream.read(&mut buffer).await {
            Ok(0) => {
                log::info!("PLC ID {} connection closed by remote", plc_id);
                // 接続が閉じられた場合
                {
                    let mut connections = state.lock();
                    if let Some(conn) = connections.get_mut(&plc_id) {
                        conn.is_connected = false;
                    }
                }

                // フロントエンドに切断イベントを送信
                let payload = serde_json::json!({
                    "plc_id": plc_id,
                    "reason": "Connection closed by remote",
                });

                if let Err(e) = app.emit("plc-disconnected", payload) {
                    log::error!("Failed to emit disconnection event: {}", e);
                }

                break;
            }
            Ok(n) => {
                log::info!("Received {} bytes from PLC ID {}", n, plc_id);

                // 受信したデータをUTF-8としてデコード
                match std::str::from_utf8(&buffer[..n]) {
                    Ok(text) => {
                        log::info!("raw_data from plc {} :{}",plc_id,text);
                        // 前回の未完了データと結合
                        incomplete_data.push_str(text);

                        // 改行で分割してメッセージを処理
                        // 最後の要素が改行で終わっていない場合は未完了データとして保持
                        let ends_with_newline = incomplete_data.ends_with('\n');
                        let mut lines: Vec<&str> = incomplete_data.split('\n').collect();

                        // 最後の要素を取り出して新しい未完了データとして保存
                        let remaining = if !ends_with_newline {
                            lines.pop().unwrap_or("").to_string()
                        } else {
                            String::new()
                        };

                        // 完全なメッセージを処理
                        for line in lines {
                            if !line.trim().is_empty() {
                                process_received_data(plc_id, line.as_bytes(), &db_tx, &app);
                            }
                        }

                        // 未完了データを更新
                        incomplete_data = remaining;
                    }
                    Err(e) => {
                        log::error!("Failed to decode UTF-8 from PLC ID {}: {}", plc_id, e);
                    }
                }
            }
            Err(e) => {
                log::error!("Error reading from PLC ID {}: {}", plc_id, e);
                // エラーが発生した場合
                {
                    let mut connections = state.lock();
                    if let Some(conn) = connections.get_mut(&plc_id) {
                        conn.is_connected = false;
                    }
                }

                // フロントエンドに切断イベントを送信
                let payload = serde_json::json!({
                    "plc_id": plc_id,
                    "reason": format!("Error: {}", e),
                });
                if let Err(e) = app.emit("plc-disconnected", payload) {
                    log::error!("Failed to emit disconnection event: {}", e);
                }

                break;
            }
        }
    }

    println!("Receive loop ended for PLC ID: {}", plc_id);
}

/// 受信したデータを処理する
fn process_received_data(plc_id: u32, data: &[u8], db_tx: &DbChannelState, app: &AppHandle) {

    // UTF-8としてデコード
    match std::str::from_utf8(data) {
        Ok(text) => {
            println!("Received text from PLC ID {}: {}", plc_id, text);
            //フロントエンドに送信する
            // JST（ローカル時刻）に変換
            let utc_now: DateTime<Utc> = Utc::now();
            let jst_now = utc_now.with_timezone(&chrono::FixedOffset::east_opt(9 * 3600).unwrap());
            let formatted_date = jst_now.format("%Y-%m-%d %H:%M:%S").to_string();

            let payload = serde_json::json!({
                "plc_id": plc_id,
                "message": text,
                "timestamp": formatted_date,
            });

            if let Err(e) = app.emit("plc-message", payload) {
                eprintln!("Failed to emit event: {}", e);
            }

            /*----受信データをデータベースに保存（チャネル経由で送信）---- */
            // 各タスクが独自のクローンを持っているので、ロック不要で高速
            if let Err(e) = save_plc_data(db_tx, plc_id, &formatted_date, text) {
                log::error!("Failed to send data to DB writer for PLC {}: {}", plc_id, e);
            }

        }
        Err(e) => {
            log::error!("Failed to decode UTF-8 from PLC ID {}: {}", plc_id, e);
        }
    }
}

/// PLCから切断する
#[command]
pub async fn disconnect_plc(
    plc_id: u32,
    state: tauri::State<'_, ConnectionState>,
    app: AppHandle,
) -> Result<String, String> {
    log::info!("Disconnecting from PLC ID: {}", plc_id);

    let mut connections = state.lock();

    if let Some(conn) = connections.get_mut(&plc_id) {
        if !conn.is_connected {
            return Err("Not connected".to_string());
        }

        conn.is_connected = false;

        // TODO: ソケットを閉じる処理

        log::info!("Disconnected from PLC ID: {}", plc_id);

        // フロントエンドに切断イベントを送信
        let payload = serde_json::json!({
            "plc_id": plc_id,
            "reason": "Manually disconnected",
        });

        if let Err(e) = app.emit("plc-disconnected", payload) {
            log::error!("Failed to emit disconnection event: {}", e);
        }

        Ok(format!("Disconnected from PLC {}", plc_id))
    } else {
        Err("PLC not found".to_string())
    }
}
