use serde::{Deserialize, Serialize};

/// ソケット通信用のデータ構造（旧バージョン用）
#[derive(Serialize, Deserialize, Debug)]
pub struct SocketData {
    pub plc_ip: String,
    pub pc_ip: String,
    pub plc_port: String,
    pub pc_port: String,
}

/// PLC設定情報
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlcConfig {
    pub id: u32,
    pub name: String,
    pub table_name: String,
    pub plc_ip: String,
    pub plc_port: u16,
    pub pc_ip: String,
}

/// 設定ファイル全体の構造
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub plcs: Vec<PlcConfig>,
}

/// PLC接続情報を管理する構造体
#[derive(Debug, Clone)]
pub struct PlcConnection {
    pub plc_id: u32,
    pub table_name:String,
    pub plc_ip: String,
    pub plc_port: u16,
    pub pc_ip: String,
    pub is_connected: bool,
}

