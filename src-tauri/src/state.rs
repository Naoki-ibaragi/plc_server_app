use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::types::PlcConnection;

/// グローバルな接続状態を管理する型
pub type ConnectionState = Arc<Mutex<HashMap<u32, PlcConnection>>>;

/// 接続状態を初期化
pub fn init_connection_state() -> ConnectionState {
    Arc::new(Mutex::new(HashMap::new()))
}
