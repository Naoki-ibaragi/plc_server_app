use std::fs;
use std::fs::File;
use std::io::Write;
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

/// PLCの追加を実施
#[command]
pub fn add_plc(
    name:String,
    table_name:String,
    plc_ip: String,
    plc_port: u16,
    pc_ip: String,
)->Result<Vec<PlcConfig>,String>{
    //config.jsonをパースしたvecに新規のconfigを追加する
    let config_path = get_config_path()?;

    // デバッグ用: パスを出力
    println!("Trying to read config from: {:?}", config_path);

    // ファイルを読み込む
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    // JSONをパース
    let mut config: Config = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    //config vecに新規のPLC情報を追加
    let id = config.plcs.len()+1;
    config.plcs.push(
        PlcConfig{
            id:id as u32,
            table_name:table_name,
            name:name,
            plc_ip:plc_ip,
            plc_port:plc_port,
            pc_ip:pc_ip,
        });

    //jsonに書き込み
    let mut file = File::create(&config_path)
        .map_err(|e| format!("Failed to open file for writing: {}", e))?;
    let json_string = serde_json::to_string_pretty(&config)
        .expect("構造体をJSONに変換できませんでした");

    // ファイルに書き込み
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("jsonファイルへの書き込み異常: {}", e))?;

    Ok(config.plcs)

}

/// PLCの編集を実施
#[command]
pub fn edit_plc(
    id:u32,
    name:String,
    table_name:String,
    plc_ip: String,
    plc_port: u16,
    pc_ip: String,
)->Result<Vec<PlcConfig>,String>{
    //config.jsonをパースしたvecに新規のconfigを追加する
    let config_path = get_config_path()?;

    // デバッグ用: パスを出力
    println!("Trying to read config from: {:?}", config_path);

    // ファイルを読み込む
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    // JSONをパース
    let mut config: Config = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    //config vecのに受け取ったIDのPLC情報を編集
    println!("Editing PLC with ID: {}", id);
    let mut found = false;
    for plc_info in config.plcs.iter_mut(){
        println!("Checking PLC ID: {}", plc_info.id);
        if id==plc_info.id{
            println!("Found matching PLC, updating...");
            plc_info.name=name.clone();
            plc_info.table_name=table_name.clone();
            plc_info.plc_ip=plc_ip.clone();
            plc_info.plc_port=plc_port;
            plc_info.pc_ip=pc_ip.clone();
            found = true;
            break;  // 該当IDを見つけたらループを抜ける
        }
    }

    if !found {
        return Err(format!("PLC with ID {} not found", id));
    }

    //jsonに書き込み
    let mut file = File::create(&config_path)
        .map_err(|e| format!("Failed to open file for writing: {}", e))?;
    let json_string = serde_json::to_string_pretty(&config)
        .expect("構造体をJSONに変換できませんでした");

    // ファイルに書き込み
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("jsonファイルへの書き込み異常: {}", e))?;

    Ok(config.plcs)

}

/// PLCの削除を実施
#[command]
pub fn delete_plc(
    plc_id: u32,
) -> Result<(), String> {
    // config.jsonを読み込む
    let config_path = get_config_path()?;

    // デバッグ用: パスを出力
    println!("Trying to read config from: {:?}", config_path);

    // ファイルを読み込む
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    // JSONをパース
    let mut config: Config = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    // 指定されたIDのPLCを探して削除
    let original_len = config.plcs.len();
    config.plcs.retain(|plc| plc.id != plc_id);

    // 削除されたか確認
    if config.plcs.len() == original_len {
        return Err(format!("PLC with ID {} not found", plc_id));
    }

    // jsonに書き込み
    let mut file = File::create(&config_path)
        .map_err(|e| format!("Failed to open file for writing: {}", e))?;
    let json_string = serde_json::to_string_pretty(&config)
        .expect("構造体をJSONに変換できませんでした");

    // ファイルに書き込み
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("jsonファイルへの書き込み異常: {}", e))?;

    println!("Successfully deleted PLC with ID: {}", plc_id);
    Ok(())
}