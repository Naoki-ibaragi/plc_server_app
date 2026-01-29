use serde_json::Value;
use sqlx::{Postgres, Transaction};
use chrono::NaiveDateTime;
use std::collections::HashMap;

///ユニット名変換実施
fn convert_unit_name(unit_name:&str)->&str{
    let converted_name=match unit_name{
        "U2_TS1"=>"dc1", //DC1測定
        "U2_A2"=>"dc1", //DC1下流
        "U2_A3"=>"ac1", //AC1上流
        "U2_TS2"=>"ac1", //AC1測定
        "U2_A4"=>"ac1", //AC1下流
        "U3_A1"=>"ac2", //AC2上流
        "U3_TS1"=>"ac2", //AC2測定
        "U3_A2"=>"ac2", //AC2下流
        "U3_A3"=>"dc2", //DC2上流
        "U3_TS2"=>"dc2", //DC2測定
        "U3_A4"=>"dc2", //DC2下流
        "U1_AL"=>"ldunit", //ld_alarm用
        "U2_AL"=>"testunit1", //test1_alarm用
        "U3_AL"=>"testunit2", //test2_alarm用
        "U4_AL"=>"uldunit", //ip,uld_alarm用
        _=>{
            log::error!("unexpected unit name");
            ""
        }
    };

    converted_name
}

/// LDトレイピックアップ情報をDBに挿入
pub async fn regist2_u1_tr_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&mut HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let trayid = hash_map.get("trayid").and_then(|v| v.as_str()).unwrap_or("unknown");
    let trayarm = hash_map.get("trayarm").and_then(|v| v.as_str()).unwrap_or("unknown");
    let px = hash_map.get("px").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let py = hash_map.get("py").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let date_str = hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("1970-01-01 00:00:00");

    // TIMESTAMP型: YYYY-MM-DD hh:mm:ss形式をそのまま使用
    let ld_pickup_date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| NaiveDateTime::default());

    //ld_pickup_dateの管理マップを更新
    //古いlot_nameが存在すればそのhashmapを破棄する
    manage_ld_pickup_date_map
    .entry(machine_id)
    .or_insert_with(HashMap::new); // 1階層目を確保

    let lot_map = manage_ld_pickup_date_map.get_mut(&machine_id).unwrap();

    // 2階層目のキー存在チェック
    let lot_name_string = lot_name.to_string();

    // lot_nameが存在しない場合は、既存のマップをクリアして新しいロットを作成
    if !lot_map.contains_key(&lot_name_string) {
        lot_map.clear();
        lot_map.insert(lot_name_string.clone(), HashMap::new());
    }

    // 3階層目の HashMap（serial → NaiveDateTime）を更新
    lot_map
        .get_mut(&lot_name_string)
        .unwrap()
        .insert(serial, ld_pickup_date);


    log::debug!("manage_pickup_date_map:{:#?}",manage_ld_pickup_date_map);

    // LOTDATEテーブルを更新（start_dateは初回のみ、end_dateは毎回更新）
    sqlx::query(
        "INSERT INTO lotdate (lot_name, start_date, end_date, machine_id)
         VALUES ($1, $2, $2, $3)
         ON CONFLICT(lot_name)
         DO UPDATE SET end_date = EXCLUDED.end_date"
    )
    .bind(lot_name).bind(ld_pickup_date).bind(machine_id)
    .execute(&mut **tx).await?;

    // 同一のserial, lot_name, machine_idが存在する場合は削除（ld_pickup_dateも含めて上書きするため）
    sqlx::query(
        "DELETE FROM chipdata2
         WHERE lot_name = $1 AND serial = $2 AND machine_id = $3"
    )
    .bind(lot_name).bind(serial).bind(machine_id)
    .execute(&mut **tx).await?;

    // 新規レコードとして挿入
    sqlx::query(
        "INSERT INTO chipdata2 (
            machine_id, type_name, lot_name, serial, ld_pickup_date,
            ld_trayid, ld_tray_arm, ld_tray_pocket_x, ld_tray_pocket_y) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial)
    .bind(ld_pickup_date).bind(trayid).bind(trayarm).bind(px).bind(py)
    .execute(&mut **tx).await?;

    Ok(())
}

/// LDのみ対象:上流アームコレット使用回数情報をDBに挿入
/// regist_ld_tr_infoがスキップされるチップがたまに存在するので、その場合ここで登録できるように変更
pub async fn regist2_ld_arm1_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&mut HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let wano = hash_map.get("wano").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let wax = hash_map.get("wax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let way = hash_map.get("way").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pax = hash_map.get("pax").and_then(|v| v.as_i64()).unwrap_or(0) as i32; //arm情報送信時にトレイトレイ補送信するように変更
    let pay = hash_map.get("pay").and_then(|v| v.as_i64()).unwrap_or(0) as i32; //arm情報送信時にトレイトレイ補送信するように変更
    let count = hash_map.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    // ld_pickup_dateが取得できない場合はここでpickupdateを登録する
    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            let date_str = hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("1970-01-01 00:00:00");
            // TIMESTAMP型: YYYY-MM-DD hh:mm:ss形式をそのまま使用
            let ld_pickup_date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|_| NaiveDateTime::default());
            //ld_pickup_dateの管理マップを更新
            //古いlot_nameが存在すればそのhashmapを破棄する
            manage_ld_pickup_date_map
            .entry(machine_id)
            .or_insert_with(HashMap::new); // 1階層目を確保

            let lot_map = manage_ld_pickup_date_map.get_mut(&machine_id).unwrap();

            // 2階層目のキー存在チェック
            let lot_name_string = lot_name.to_string();

            // lot_nameが存在しない場合は、既存のマップをクリアして新しいロットを作成
            if !lot_map.contains_key(&lot_name_string) {
                lot_map.clear();
                lot_map.insert(lot_name_string.clone(), HashMap::new());
            }

            // 3階層目の HashMap（serial → NaiveDateTime）を更新
            lot_map
                .get_mut(&lot_name_string)
                .unwrap()
                .insert(serial, ld_pickup_date);

            // LOTDATEテーブルの更新
            sqlx::query(
                "INSERT INTO lotdate (lot_name, start_date, end_date, machine_id)
                 VALUES ($1, $2, $2, $3)
                 ON CONFLICT(lot_name)
                 DO UPDATE SET end_date = EXCLUDED.end_date"
            )
            .bind(lot_name).bind(ld_pickup_date).bind(machine_id)
            .execute(&mut **tx).await?;

            ld_pickup_date
        }
    };

    sqlx::query(&format!(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, serial, ld_pickup_date, wano, wax, way,ld_arm1_collet,ld_tray_align_x, ld_tray_align_y)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET 
         wano = EXCLUDED.wano,wax = EXCLUDED.wax,way = EXCLUDED.way,ld_arm1_collet = EXCLUDED.ld_arm1_collet,ld_tray_align_x=EXCLUDED.ld_tray_align_x,ld_tray_align_y=EXCLUDED.ld_tray_align_y",
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date).bind(wano).bind(wax).bind(way).bind(count).bind(pax).bind(pay)
    .execute(&mut **tx).await?;

    Ok(())
}

/// DC1のみ対象:コレット使用回数情報,予熱部補正,トレイポケット補正情報をDBに挿入
pub async fn regist2_dc1_arm1_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let count = hash_map.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let ax = hash_map.get("ax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let ay = hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let at = hash_map.get("at").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    // ld_pickup_dateが取得できない場合はスキップ（U1_TRがまだ来ていない）
    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            log::warn!("ld_pickup_date not found for machine_id:{}, lot:{}, serial:{}", machine_id, lot_name, serial);
            return Ok(());
        }
    };

    sqlx::query(&format!(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, ld_pickup_date, serial, dc1_arm1_collet, dc1_pre_align_x, dc1_pre_align_y, dc1_pre_align_t)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET 
         dc1_arm1_collet = EXCLUDED.dc1_arm1_collet, dc1_pre_align_x=EXCLUDED.dc1_pre_align_x, dc1_pre_align_y=EXCLUDED.dc1_pre_align_y, dc1_pre_align_t=EXCLUDED.dc1_pre_align_t"
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(ld_pickup_date).bind(serial).bind(count).bind(ax).bind(ay).bind(at)
    .execute(&mut **tx).await?;

    Ok(())
}



/// DC1以外の上流アームコレット使用回数情報をDBに挿入
pub async fn regist2_arm1_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let count = hash_map.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let column_name = format!("{}_arm1_collet", convert_unit_name(unit_name).to_lowercase());

    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    // ld_pickup_dateが取得できない場合はスキップ（U1_TRがまだ来ていない）
    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            log::warn!("ld_pickup_date not found for machine_id:{}, lot:{}, serial:{}", machine_id, lot_name, serial);
            return Ok(());
        }
    };

    sqlx::query(&format!(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, serial, ld_pickup_date, {})
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET {} = EXCLUDED.{}",
        column_name, column_name, column_name
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date).bind(count)
    .execute(&mut **tx).await?;

    Ok(())
}

/// 下流アームコレット使用回数情報をDBに挿入
pub async fn regist2_arm2_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let count = hash_map.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let column_name = format!("{}_arm2_collet", convert_unit_name(unit_name).to_lowercase());

    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            log::warn!("ld_pickup_date not found for machine_id:{}, lot:{}, serial:{}", machine_id, lot_name, serial);
            return Ok(());
        }
    };

    sqlx::query(&format!(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, serial, ld_pickup_date, {})
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET {} = EXCLUDED.{}",
        column_name, column_name, column_name
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(count)
    .execute(&mut **tx).await?;

    Ok(())
}

/// 検査テーブルデータをDBに挿入 (DC1~DC2)
pub async fn regist2_ts_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let stage_ser = hash_map.get("stage_serial").and_then(|v| v.as_str()).unwrap_or("unknown");
    let stage_cnt = hash_map.get("stage_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let probe_ser = hash_map.get("probe_serial").and_then(|v| v.as_str()).unwrap_or("unknown");
    let probe_cnt = hash_map.get("probe_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let px1 = hash_map.get("pax").and_then(|v| v.as_i64()).unwrap_or(0) as i32; //プローブX位置
    let py1 = hash_map.get("pay").and_then(|v| v.as_i64()).unwrap_or(0) as i32; //プローブY位置
    let px2 = hash_map.get("pat").and_then(|v| v.as_i64()).unwrap_or(0) as i32; //プローブΘ位置
    let sz = hash_map.get("stage_z").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pz = hash_map.get("pin_z").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cax = hash_map.get("ax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cay = hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cat = hash_map.get("at").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let bin = hash_map.get("bin").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let unit_lower = convert_unit_name(unit_name).to_lowercase();

    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            log::warn!("ld_pickup_date not found for machine_id:{}, lot:{}, serial:{}", machine_id, lot_name, serial);
            return Ok(());
        }
    };

    sqlx::query(&format!(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, serial, ld_pickup_date,
         {0}_stage_serial, {0}_stage_count, {0}_probe_serial, {0}_probe_count,
         {0}_probe_align_x, {0}_probe_align_y, {0}_probe_align_t,
         {0}_stage_z, {0}_pin_z, {0}_chip_align_x, {0}_chip_align_y,
         {0}_chip_align_t, {0}_test_bin)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET
         {0}_stage_serial = EXCLUDED.{0}_stage_serial, {0}_stage_count = EXCLUDED.{0}_stage_count,
         {0}_probe_serial = EXCLUDED.{0}_probe_serial, {0}_probe_count = EXCLUDED.{0}_probe_count,
         {0}_probe_align_x = EXCLUDED.{0}_probe_align_x, {0}_probe_align_y = EXCLUDED.{0}_probe_align_y,{0}_probe_align_t = EXCLUDED.{0}_probe_align_t,
         {0}_stage_z = EXCLUDED.{0}_stage_z, {0}_pin_z = EXCLUDED.{0}_pin_z,
         {0}_chip_align_x = EXCLUDED.{0}_chip_align_x, {0}_chip_align_y = EXCLUDED.{0}_chip_align_y,
         {0}_chip_align_t = EXCLUDED.{0}_chip_align_t, {0}_test_bin = EXCLUDED.{0}_test_bin",
        unit_lower
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(stage_ser).bind(stage_cnt).bind(probe_ser).bind(probe_cnt)
    .bind(px1).bind(py1).bind(px2)
    .bind(sz).bind(pz).bind(cax).bind(cay).bind(cat).bind(bin)
    .execute(&mut **tx).await?;

    Ok(())
}

/// IP表面検査BINデータをDBに挿入
pub async fn regist2_ip_surf_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let bin = hash_map.get("surf_bin").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let count = hash_map.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let stage_count = hash_map.get("stage_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let ax = hash_map.get("ax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let ay = hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let at = hash_map.get("at").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            log::warn!("ld_pickup_date not found for machine_id:{}, lot:{}, serial:{}", machine_id, lot_name, serial);
            return Ok(());
        }
    };

    sqlx::query(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, serial, ld_pickup_date, ip_surf_bin,
        uld_pre_align_x, uld_pre_align_y, uld_pre_align_t, ip_stage_count, uld_arm1_collet)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET 
         ip_surf_bin = EXCLUDED.ip_surf_bin,
         uld_pre_align_x = EXCLUDED.uld_pre_align_x,
         uld_pre_align_y = EXCLUDED.uld_pre_align_y,
         uld_pre_align_t = EXCLUDED.uld_pre_align_t,
         ip_stage_count = EXCLUDED.ip_stage_count,
         uld_arm1_collet = EXCLUDED.uld_arm1_collet"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date).bind(bin)
    .bind(ax).bind(ay).bind(at).bind(stage_count).bind(count)
    .execute(&mut **tx).await?;

    Ok(())
}

/// IP裏面検査BINデータをDBに挿入
pub async fn regist2_ip_back_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let bin = hash_map.get("back_bin").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pf = hash_map.get("pf").and_then(|v| v.as_str()).unwrap_or("");
    let px = hash_map.get("px").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let py = hash_map.get("py").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pax = hash_map.get("pax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pay = hash_map.get("pay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            log::warn!("ld_pickup_date not found for machine_id:{}, lot:{}, serial:{}", machine_id, lot_name, serial);
            return Ok(());
        }
    };

    sqlx::query(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, serial, ld_pickup_date, ip_back_bin,
        uld_pf, uld_pocket_x, uld_pocket_y, uld_pocket_align_x, uld_pocket_align_y)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET 
         ip_back_bin = EXCLUDED.ip_back_bin,
         uld_pf = EXCLUDED.uld_pf,
         uld_pocket_x = EXCLUDED.uld_pocket_x,
         uld_pocket_y = EXCLUDED.uld_pocket_y,
         uld_pocket_align_x = EXCLUDED.uld_pocket_align_x,
         uld_pocket_align_y = EXCLUDED.uld_pocket_align_y"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(bin).bind(pf).bind(px).bind(py).bind(pax).bind(pay)
    .execute(&mut **tx).await?;

    Ok(())
}

/// ULDポケット認識データをDBに挿入
pub async fn regist2_uld_pocket_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cax = hash_map.get("cax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cay = hash_map.get("cay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let trayid = hash_map.get("trayid").and_then(|v| v.as_str()).unwrap_or("unknown");
    let date_str = hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("1970-01-01 00:00:00");

    // TIMESTAMP型: YYYY-MM-DD hh:mm:ss形式をそのまま使用
    let uld_put_date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| NaiveDateTime::default());


    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            log::warn!("ld_pickup_date not found for machine_id:{}, lot:{}, serial:{}", machine_id, lot_name, serial);
            return Ok(());
        }
    };

    sqlx::query(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, serial, ld_pickup_date,
         uld_trayid,uld_chip_align_x, uld_chip_align_y, uld_put_date)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET
         uld_trayid = EXCLUDED.uld_trayid,
         uld_chip_align_x = EXCLUDED.uld_chip_align_x, uld_chip_align_y = EXCLUDED.uld_chip_align_y, uld_put_date=EXCLUDED.uld_put_date"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(trayid).bind(cax).bind(cay).bind(uld_put_date)
    .execute(&mut **tx).await?;

    Ok(())
}

/// アラーム情報をDBに挿入
/// eventsテーブルにも情報を挿入する
pub async fn regist2_alarm_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    manage_ld_pickup_date_map:&HashMap<i32,HashMap<String,HashMap<i32,NaiveDateTime>>>
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();

    //まずeventsテーブルに情報を登録する(alarm_numとalarm_dateをhash_mapから取得して登録)
    let alarm = hash_map.get("alarm_num").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let date_str = hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("1970-01-01 00:00:00");
    // TIMESTAMP型: YYYY-MM-DD hh:mm:ss形式をそのまま使用
    let event_date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| NaiveDateTime::default());

    sqlx::query(&format!(
        "INSERT INTO events (machine_id, type_name, lot_name, date, event_type, alarm_unit, alarm_code)
         VALUES ($1, $2, $3, $4, $5, $6, $7)")
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(event_date).bind("ALARM").bind(convert_unit_name(unit_name)).bind(alarm)
    .execute(&mut **tx).await?;

    // serialは配列形式で来る（例: [1,2,0,0]）
    let serial_array = hash_map.get("serial").and_then(|v| v.as_array());

    // 配列から最初の0以外の要素を取得
    let serial = match serial_array {
        Some(arr) => {
            arr.iter()
                .filter_map(|v| v.as_i64())
                .map(|v| v as i32)
                .find(|&v| v != 0)
        },
        None => None
    };

    // serialが見つからない（全て0）場合は処理をスキップ
    let serial = match serial {
        Some(s) => s,
        None => {
            log::debug!("All serial values are 0, skipping alarm info for machine_id:{}, lot:{}", machine_id, lot_name);
            return Ok(());
        }
    };

    let column_name = format!("{}_alarm", convert_unit_name(unit_name).to_lowercase());

    //ld_pickup_date取得
    let ld_pickup_date = manage_ld_pickup_date_map
        .get(&machine_id)
        .and_then(|lot_map| lot_map.get(lot_name))
        .and_then(|serial_map| serial_map.get(&serial))
        .copied();

    let ld_pickup_date = match ld_pickup_date {
        Some(date) => date,
        None => {
            log::warn!("ld_pickup_date not found for machine_id:{}, lot:{}, serial:{}", machine_id, lot_name, serial);
            return Ok(());
        }
    };

    sqlx::query(&format!(
        "INSERT INTO chipdata2 (machine_id, type_name, lot_name, serial, ld_pickup_date, {})
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET {} = EXCLUDED.{}",
        column_name, column_name, column_name
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(alarm)
    .execute(&mut **tx).await?;

    Ok(())
}

/// イベント情報をEVENTSテーブルに登録
pub async fn regist2_event_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    event_type: &str,
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();

    let date_str = hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("1970-01-01 00:00:00");
    // TIMESTAMP型: YYYY-MM-DD hh:mm:ss形式をそのまま使用
    let event_date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| NaiveDateTime::default());

    sqlx::query(&format!(
        "INSERT INTO events (machine_id, type_name, lot_name, date, event_type)
         VALUES ($1, $2, $3, $4, $5)")
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(event_date).bind(event_type)
    .execute(&mut **tx).await?;

    Ok(())
}
