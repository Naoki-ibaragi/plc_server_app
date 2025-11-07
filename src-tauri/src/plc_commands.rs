use tauri::{AppHandle, command,Emitter};
use tokio::net::{TcpStream, TcpListener};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use crate::types::PlcConnection;
use crate::state::ConnectionState;
use chrono::{DateTime, Utc};
use crate::data_handler::{create_table_for_plc, save_plc_data};

/// PLCに接続する
#[command]
pub async fn connect_plc(
    plc_id: u32,
    plc_ip: String,
    plc_port: u16,
    pc_ip: String,
    state: tauri::State<'_, ConnectionState>,
    app: AppHandle,
) -> Result<String, String> {
     println!("Connecting to PLC ID: {}, IP: {}:{}", plc_id, plc_ip, plc_port);

    // 既に接続されているかチェック
    {
        let connections = state.lock();
        if let Some(conn) = connections.get(&plc_id) {
            if conn.is_connected {
                return Err("Already connected".to_string());
            }
        }
    }

    // PCのポートでリッスンを開始（ポート番号0で自動割り当て）
    let listen_addr = format!("{}:0", pc_ip);
    println!("Trying to listen on: {} (auto-assign port)", listen_addr);

    let listener = TcpListener::bind(&listen_addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", listen_addr, e))?;

    let local_addr = listener.local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?;
    let pc_port = local_addr.port();

    println!("Listening on {}:{}", pc_ip, pc_port);

    // PLCに接続を試みる（接続先として）
    let plc_addr = format!("{}:{}", plc_ip, plc_port);
    println!("Trying to connect to PLC at: {}", plc_addr);

    let stream = TcpStream::connect(&plc_addr)
        .await
        .map_err(|e| format!("Failed to connect to PLC at {}: {}", plc_addr, e))?;

    println!("Connected to PLC at {}", plc_addr);

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

    // PLCごとのテーブルを作成
    if let Err(e) = create_table_for_plc(plc_id) {
        eprintln!("Failed to create table for PLC {}: {}", plc_id, e);
    }

    // 受信ループを別のタスクで実行
    let state_clone = Arc::clone(&state.inner());
    tokio::spawn(async move {
        receive_data_from_plc(plc_id, stream, state_clone,app).await;
    });

    Ok(format!("Connected to PLC {}:{}", plc_ip, plc_port))
}

/// PLCからデータを受信する
async fn receive_data_from_plc(
    plc_id: u32,
    mut stream: TcpStream,
    state: ConnectionState,
    app:AppHandle,
) {
    println!("Starting receive loop for PLC ID: {}", plc_id);
    let mut buffer = vec![0u8; 4096];

    loop {
        // 接続状態をチェック
        {
            let connections = state.lock();
            if let Some(conn) = connections.get(&plc_id) {
                if !conn.is_connected {
                    println!("PLC ID {} is disconnected, stopping receive loop", plc_id);
                    break;
                }
            } else {
                println!("PLC ID {} not found in state, stopping receive loop", plc_id);
                break;
            }
        }

        // データを受信
        match stream.read(&mut buffer).await {
            Ok(0) => {
                println!("PLC ID {} connection closed by remote", plc_id);
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
                    eprintln!("Failed to emit disconnection event: {}", e);
                }

                break;
            }
            Ok(n) => {
                println!("Received {} bytes from PLC ID {}", n, plc_id);
                // 受信したデータを処理
                let received_data = &buffer[..n];
                process_received_data(plc_id, received_data,&app);
            }
            Err(e) => {
                eprintln!("Error reading from PLC ID {}: {}", plc_id, e);
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
                    eprintln!("Failed to emit disconnection event: {}", e);
                }

                break;
            }
        }
    }

    println!("Receive loop ended for PLC ID: {}", plc_id);
}

/// 受信したデータを処理する
fn process_received_data(plc_id: u32, data: &[u8],app:&AppHandle) {
    println!("Processing data for PLC ID {}: {:?}", plc_id, data);

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

            // データベースに保存（チャネル経由で送信）
            if let Err(e) = save_plc_data(plc_id, &formatted_date, text) {
                eprintln!("Failed to send data to DB writer for PLC {}: {}", plc_id, e);
            }

        }
        Err(e) => {
            eprintln!("Failed to decode UTF-8 from PLC ID {}: {}", plc_id, e);
            // デコード失敗時は16進数で表示
            let hex_string: String = data.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<String>>()
                .join(" ");
            println!("Data in hex: {}", hex_string);
        }
    }
}

/// PLCから切断する
#[command]
pub async fn disconnect_plc(
    plc_id: u32,
    state: tauri::State<'_, ConnectionState>,
) -> Result<String, String> {
    println!("Disconnecting from PLC ID: {}", plc_id);

    let mut connections = state.lock();

    if let Some(conn) = connections.get_mut(&plc_id) {
        if !conn.is_connected {
            return Err("Not connected".to_string());
        }

        conn.is_connected = false;

        // TODO: ソケットを閉じる処理

        println!("Disconnected from PLC ID: {}", plc_id);
        Ok(format!("Disconnected from PLC {}", plc_id))
    } else {
        Err("PLC not found".to_string())
    }
}
