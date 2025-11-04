use std::fs;
use std::path::PathBuf;
use tauri::command;
use crate::types::{Config, PlcConfig};

/// config.jsonを読み込んでPLC設定情報をフロントエンドに渡す
#[command]
pub async fn init_socket() -> Result<Vec<PlcConfig>, String> {
    // 実行ファイルのディレクトリからconfig.jsonを読み込む
    let config_path = get_config_path()?;

    // デバッグ用: パスを出力
    println!("Trying to read config from: {:?}", config_path);

    // ファイルを読み込む
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    // JSONをパース
    let config: Config = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    Ok(config.plcs)
}

/// 設定ファイルのパスを取得
pub fn get_config_path() -> Result<PathBuf, String> {
    // 開発時とリリース時でパスを変える
    #[cfg(debug_assertions)]
    {
        // 開発時: 現在のディレクトリを確認して適切なパスを構築
        let current = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        println!("Current directory: {:?}", current);

        // src-tauriディレクトリにいる場合は、そのままconfig.jsonを探す
        let mut path = current.clone();
        path.push("config.json");
        if path.exists() {
            return Ok(path);
        }

        // プロジェクトルートにいる場合は、src-tauri/config.jsonを探す
        let mut path = current;
        path.push("src-tauri");
        path.push("config.json");
        Ok(path)
    }

    #[cfg(not(debug_assertions))]
    {
        // リリース時: 実行ファイルと同じディレクトリからconfig.jsonを読む
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;
        let mut config_path = exe_path.parent()
            .ok_or("Failed to get parent directory")?
            .to_path_buf();
        config_path.push("config.json");
        Ok(config_path)
    }
}
