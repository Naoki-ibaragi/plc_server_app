use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::types::PlcConnection;
use crate::data_handler::DbWriteRequest;
use tokio::sync::mpsc;

/// グローバルな接続状態を管理する型
pub type ConnectionState = Arc<Mutex<HashMap<u32, PlcConnection>>>;

/// DB書き込みチャネルの送信側を管理する型
pub type DbChannelState = mpsc::UnboundedSender<DbWriteRequest>;

/// 接続状態を初期化
pub fn init_connection_state() -> ConnectionState {
    Arc::new(Mutex::new(HashMap::new()))
}
