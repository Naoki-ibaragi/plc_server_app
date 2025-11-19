///PLCから受け取ったデータのハンドラー
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use std::collections::HashMap;
use serde_json::Value;
use std::env;

use crate::regist_data_to_db::*;

lazy_static! {
    static ref DB_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);
}

//テーブルを作成するためのsql文を読み込み
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

    tx
}

/// DB書き込み専用スレッドを起動する
/// チャネルの送信側を返すので、呼び出し側で clone して使用する
fn start_db_writer_thread() -> Result<mpsc::UnboundedSender<DbWriteRequest>,rusqlite::Error> {
    let (tx, mut rx) = mpsc::unbounded_channel::<DbWriteRequest>();

    // DB書き込み専用スレッドを起動
    std::thread::spawn(move || {
        println!("DB writer thread started");

        while let Some(request) = rx.blocking_recv() {
            let table_name = format!("clt_data_{}", request.plc_id);

            let db = DB_CONNECTION.lock().unwrap();
            if let Some(conn) = db.as_ref() {
                conn.execute("BEGIN TRANSACTION",[]);
                //PLCから受信したjson形式データをhashmapに変換する
                let recv_data: HashMap<String, Value> = serde_json::from_str(&request.message).unwrap();
                
                //ロット番号のとりだし
                let lot_name = recv_data
                    .get("LOT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                //機種名のとりだし
                let type_name = recv_data
                    .get("TYPE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                //装置名のとりだし
                let machine_name = recv_data
                    .get("MACHINE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                println!("lot_name:{}",lot_name);
                println!("type_name:{}",type_name);
                println!("machine_name:{}",machine_name);

                //各ユニット情報の取り出し
                for (key,value) in &recv_data{
                    if key.contains("U1_TR"){ //LD TRAYデータを登録
                        match regist_u1_tr_info(&conn,&table_name,&machine_name,&lot_name,&type_name,value){
                            Err(e)=>{
                                println!("regist tray data  error:{}",e);
                                continue;
                            },
                            _=>{}
                        };
                    }else if key.contains("_A1_"){ //上流アームコレットの使用回数データを登録
                        //アームのユニット名を取得
                        let unit_name = match key.split('_').next() {
                            Some(v) => v,
                            None => continue,  // ここで continue が使える！
                        };
                        match regist_arm1_info(&conn,&table_name,&machine_name,&lot_name,&type_name,&unit_name,value){
                            Err(e)=>{
                                println!("regist arm1 data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("_A2_"){ //下流アームコレットの使用回数データを登録
                        //アームのユニット名を取得
                        let unit_name = match key.split('_').next() {
                            Some(v) => v,
                            None => continue,  // ここで continue が使える！
                        };
                        match regist_arm2_info(&conn,&table_name,&machine_name,&lot_name,&type_name,&unit_name,value){
                            Err(e)=>{
                                println!("regist arm2 data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("_PH_"){ //DC1,ULD予熱テーブルのデータを登録
                        //アームのユニット名を取得
                        let unit_name = match key.split('_').next() {
                            Some(v) => v,
                            None => continue,  // ここで continue が使える！
                        };
                        match regist_ph_info(&conn,&table_name,&machine_name,&lot_name,&type_name,&unit_name,value){
                            Err(e)=>{
                                println!("regist preheat data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("_TS_") && !key.contains("U6"){ //DC1~DC2検査テーブルのデータを登録
                        //ユニット名を取得
                        let unit_name = match key.split('_').next() {
                            Some(v) => v,
                            None => continue,  // ここで continue が使える！
                        };
                        match regist_ts_info(&conn,&table_name,&machine_name,&lot_name,&type_name,&unit_name,value){
                            Err(e)=>{
                                println!("regist teststage data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("_TS_") && key.contains("U6"){ //IP検査テーブルのデータを登録
                        match regist_ip_ts_info(&conn,&table_name,&machine_name,&lot_name,&type_name,value){
                            Err(e)=>{
                                println!("regist ph data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("U6_T1_"){ //IP表面検査のBINデータを登録
                        match regist_ip_surf_info(&conn,&table_name,&machine_name,&lot_name,&type_name,value){
                            Err(e)=>{
                                println!("regist ph data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("U6_T2_"){ //IP裏面検検のBINデータを登録
                        match regist_ip_back_info(&conn,&table_name,&machine_name,&lot_name,&type_name,value){
                            Err(e)=>{
                                println!("regist ph data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("U7_PI_"){ //ULDポケット認識時のデータを登録
                        match regist_uld_pocket_info(&conn,&table_name,&machine_name,&lot_name,&type_name,value){
                            Err(e)=>{
                                println!("regist ph data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("U7_CI_"){ //ULDポケット挿入時のデータを登録
                        match regist_uld_chip_info(&conn,&table_name,&machine_name,&lot_name,&type_name,value){
                            Err(e)=>{
                                println!("regist ph data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }else if key.contains("_AL_"){ //アラーム情報の登録
                        //ユニット名を取得
                        let unit_name = match key.split('_').next() {
                            Some(v) => v,
                            None => continue,  // ここで continue が使える！
                        };
                        match regist_alarm_info(&conn,&table_name,&machine_name,&lot_name,&type_name,&unit_name,value){
                            Err(e)=>{
                                println!("regist alarm data error:{}",e);
                                continue;
                            },
                            _=>{}
                        }
                    }
                }
                conn.execute("COMMIT",[]);
            }
        }//<-threadの終端

        println!("DB writer thread stopped");
    });

    Ok(tx)
}

/// データベースのパスを取得
fn get_database_path() -> PathBuf {
    // 実行ファイルのディレクトリにデータベースを配置
    let path:String=env::var("DB_PATH").unwrap_or("C:\\chiptest.db".to_string());
    PathBuf::from(path)
}

/// PLC IDに基づいてテーブルを作成する
/// テーブル名: plc_data_{plc_id}
pub fn create_table_for_plc(plc_id: u32) -> Result<()> {
    let table_name = format!("clt_data_{}", plc_id);
    let sql=CREATE_TABLE_SQL.replace("{TABLE_NAME}",&table_name);

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
