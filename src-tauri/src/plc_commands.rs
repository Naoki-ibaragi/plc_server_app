use tauri::command;
use tokio::net::{TcpStream, TcpListener};
use crate::types::PlcConnection;
use crate::state::ConnectionState;

/// PLCに接続する
#[command]
pub async fn connect_plc(
    plc_id: u32,
    plc_ip: String,
    plc_port: u16,
    pc_ip: String,
    pc_port: u16,
    state: tauri::State<'_, ConnectionState>,
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

    // PCのポートでリッスンを開始
    let listen_addr = format!("{}:{}", pc_ip, pc_port);
    println!("Trying to listen on: {}", listen_addr);

    let _listener = TcpListener::bind(&listen_addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", listen_addr, e))?;

    println!("Listening on {}", listen_addr);

    // PLCに接続を試みる（接続先として）
    let plc_addr = format!("{}:{}", plc_ip, plc_port);
    println!("Trying to connect to PLC at: {}", plc_addr);

    let _stream = TcpStream::connect(&plc_addr)
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
                pc_port,
                is_connected: true,
            },
        );
    }

    // TODO: 受信ループを別のタスクで実行する
    // tokio::spawn(async move {
    //     // 受信処理
    // });

    Ok(format!("Connected to PLC {}:{}", plc_ip, plc_port))
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
