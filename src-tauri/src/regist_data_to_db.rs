use serde_json::Value;
use sqlx::{Postgres, Transaction};
use chrono::NaiveDateTime;

/// LDトレイピックアップ情報をDBに挿入
pub async fn regist_u1_tr_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let wano = hash_map.get("wano").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let wax = hash_map.get("wax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let way = hash_map.get("way").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let trayid = hash_map.get("trayid").and_then(|v| v.as_str()).unwrap_or("unknown");
    let trayarm = hash_map.get("trayarm").and_then(|v| v.as_str()).unwrap_or("unknown");
    let px = hash_map.get("px").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let py = hash_map.get("py").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pax = hash_map.get("pax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pay = hash_map.get("pay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let date_str = hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("1970-01-01 00:00:00");

    // TIMESTAMP型: YYYY-MM-DD hh:mm:ss形式をそのまま使用
    let ld_pickup_date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| NaiveDateTime::default());

    sqlx::query(
        "INSERT INTO chipdata (
            machine_id, type_name, lot_name, serial, wano, wax, way, ld_pickup_date,
            ld_trayid, ld_tray_arm, ld_tray_pocket_x, ld_tray_pocket_y,
            ld_tray_align_x, ld_tray_align_y
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
        DO UPDATE SET
            type_name = EXCLUDED.type_name, wano = EXCLUDED.wano, wax = EXCLUDED.wax,
            way = EXCLUDED.way, ld_trayid = EXCLUDED.ld_trayid, ld_tray_arm = EXCLUDED.ld_tray_arm,
            ld_tray_pocket_x = EXCLUDED.ld_tray_pocket_x, ld_tray_pocket_y = EXCLUDED.ld_tray_pocket_y,
            ld_tray_align_x = EXCLUDED.ld_tray_align_x, ld_tray_align_y = EXCLUDED.ld_tray_align_y"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial)
    .bind(wano).bind(wax).bind(way).bind(ld_pickup_date)
    .bind(trayid).bind(trayarm).bind(px).bind(py).bind(pax).bind(pay)
    .execute(&mut **tx).await?;

    Ok(())
}

/// 上流アームコレット使用回数情報をDBに挿入
pub async fn regist_arm1_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let col = hash_map.get("col").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let column_name = format!("{}_arm1_collet", unit_name.to_lowercase());

    sqlx::query(&format!(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date, {})
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET {} = EXCLUDED.{}",
        column_name, column_name, column_name
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial)
    .bind(ld_pickup_date).bind(col)
    .execute(&mut **tx).await?;

    Ok(())
}

/// 下流アームコレット使用回数情報をDBに挿入
pub async fn regist_arm2_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let col = hash_map.get("col").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let column_name = format!("{}_arm2_collet", unit_name.to_lowercase());

    sqlx::query(&format!(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date, {})
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET {} = EXCLUDED.{}",
        column_name, column_name, column_name
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial)
    .bind(ld_pickup_date).bind(col)
    .execute(&mut **tx).await?;

    Ok(())
}

/// 予熱テーブルデータをDBに挿入
pub async fn regist_ph_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let stage_ser = hash_map.get("stage_ser").and_then(|v| v.as_str()).unwrap_or("unknown");
    let stage_cnt = hash_map.get("stage_cnt").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pax = hash_map.get("pax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pay = hash_map.get("pay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pat = hash_map.get("pat").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let unit_lower = unit_name.to_lowercase();
    let stage_serial_col = format!("{}_stage_serial", unit_lower);
    let stage_count_col = format!("{}_stage_count", unit_lower);
    let pre_align_x_col = format!("{}_pre_align_x", unit_lower);
    let pre_align_y_col = format!("{}_pre_align_y", unit_lower);
    let pre_align_t_col = format!("{}_pre_align_t", unit_lower);

    sqlx::query(&format!(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date,
         {}, {}, {}, {}, {})
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET
         {} = EXCLUDED.{}, {} = EXCLUDED.{}, {} = EXCLUDED.{},
         {} = EXCLUDED.{}, {} = EXCLUDED.{}",
        stage_serial_col, stage_count_col, pre_align_x_col, pre_align_y_col, pre_align_t_col,
        stage_serial_col, stage_serial_col, stage_count_col, stage_count_col,
        pre_align_x_col, pre_align_x_col, pre_align_y_col, pre_align_y_col,
        pre_align_t_col, pre_align_t_col
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(stage_ser).bind(stage_cnt).bind(pax).bind(pay).bind(pat)
    .execute(&mut **tx).await?;

    Ok(())
}

/// 検査テーブルデータをDBに挿入 (DC1~DC2)
pub async fn regist_ts_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let stage_ser = hash_map.get("stage_ser").and_then(|v| v.as_str()).unwrap_or("unknown");
    let stage_cnt = hash_map.get("stage_cnt").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let probe_ser = hash_map.get("probe_ser").and_then(|v| v.as_str()).unwrap_or("unknown");
    let probe_cnt = hash_map.get("probe_cnt").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let px1 = hash_map.get("px1").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let py1 = hash_map.get("py1").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let px2 = hash_map.get("px2").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let py2 = hash_map.get("py2").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let sz = hash_map.get("sz").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pz = hash_map.get("pz").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cax = hash_map.get("cax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cay = hash_map.get("cay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cat = hash_map.get("cat").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let bin = hash_map.get("bin").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let unit_lower = unit_name.to_lowercase();

    sqlx::query(&format!(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date,
         {0}_stage_serial, {0}_stage_count, {0}_probe_serial, {0}_probe_count,
         {0}_probe_x1, {0}_probe_y1, {0}_probe_x2, {0}_probe_y2,
         {0}_stage_z, {0}_pin_z, {0}_chip_align_x, {0}_chip_align_y,
         {0}_chip_align_t, {0}_test_bin)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET
         {0}_stage_serial = EXCLUDED.{0}_stage_serial, {0}_stage_count = EXCLUDED.{0}_stage_count,
         {0}_probe_serial = EXCLUDED.{0}_probe_serial, {0}_probe_count = EXCLUDED.{0}_probe_count,
         {0}_probe_x1 = EXCLUDED.{0}_probe_x1, {0}_probe_y1 = EXCLUDED.{0}_probe_y1,
         {0}_probe_x2 = EXCLUDED.{0}_probe_x2, {0}_probe_y2 = EXCLUDED.{0}_probe_y2,
         {0}_stage_z = EXCLUDED.{0}_stage_z, {0}_pin_z = EXCLUDED.{0}_pin_z,
         {0}_chip_align_x = EXCLUDED.{0}_chip_align_x, {0}_chip_align_y = EXCLUDED.{0}_chip_align_y,
         {0}_chip_align_t = EXCLUDED.{0}_chip_align_t, {0}_test_bin = EXCLUDED.{0}_test_bin",
        unit_lower
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(stage_ser).bind(stage_cnt).bind(probe_ser).bind(probe_cnt)
    .bind(px1).bind(py1).bind(px2).bind(py2)
    .bind(sz).bind(pz).bind(cax).bind(cay).bind(cat).bind(bin)
    .execute(&mut **tx).await?;

    Ok(())
}

/// IP検査テーブルデータをDBに挿入
pub async fn regist_ip_ts_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let stage_cnt = hash_map.get("stage_cnt").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    sqlx::query(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date, ip_stage_count)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET ip_stage_count = EXCLUDED.ip_stage_count"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial)
    .bind(ld_pickup_date).bind(stage_cnt)
    .execute(&mut **tx).await?;

    Ok(())
}

/// IP表面検査BINデータをDBに挿入
pub async fn regist_ip_surf_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let bin = hash_map.get("bin").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    sqlx::query(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date, ip_surf_bin)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET ip_surf_bin = EXCLUDED.ip_surf_bin"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial)
    .bind(ld_pickup_date).bind(bin)
    .execute(&mut **tx).await?;

    Ok(())
}

/// IP裏面検査BINデータをDBに挿入
pub async fn regist_ip_back_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let bin = hash_map.get("bin").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    sqlx::query(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date, ip_back_bin)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET ip_back_bin = EXCLUDED.ip_back_bin"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial)
    .bind(ld_pickup_date).bind(bin)
    .execute(&mut **tx).await?;

    Ok(())
}

/// ULDポケット認識データをDBに挿入
pub async fn regist_uld_pocket_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pax = hash_map.get("pax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pay = hash_map.get("pay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pat = hash_map.get("pat").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let trayid = hash_map.get("trayid").and_then(|v| v.as_str()).unwrap_or("unknown");
    let px = hash_map.get("px").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let py = hash_map.get("py").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let ppax = hash_map.get("ppax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let ppay = hash_map.get("ppay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    sqlx::query(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date,
         uld_pre_align_x, uld_pre_align_y, uld_pre_align_t, uld_trayid,
         uld_pocket_x, uld_pocket_y, uld_pocket_align_x, uld_pocket_align_y)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET
         uld_pre_align_x = EXCLUDED.uld_pre_align_x, uld_pre_align_y = EXCLUDED.uld_pre_align_y,
         uld_pre_align_t = EXCLUDED.uld_pre_align_t, uld_trayid = EXCLUDED.uld_trayid,
         uld_pocket_x = EXCLUDED.uld_pocket_x, uld_pocket_y = EXCLUDED.uld_pocket_y,
         uld_pocket_align_x = EXCLUDED.uld_pocket_align_x, uld_pocket_align_y = EXCLUDED.uld_pocket_align_y"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(pax).bind(pay).bind(pat).bind(trayid)
    .bind(px).bind(py).bind(ppax).bind(ppay)
    .execute(&mut **tx).await?;

    Ok(())
}

/// ULDチップアライメントデータをDBに挿入
pub async fn regist_uld_chip_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let date_str = hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("1970-01-01 00:00:00");
    let cax = hash_map.get("cax").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cay = hash_map.get("cay").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let canum = hash_map.get("canum").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    // TIMESTAMP型: YYYY-MM-DD hh:mm:ss形式をそのまま使用
    let uld_put_date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| NaiveDateTime::default());

    sqlx::query(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date,
         uld_put_date, uld_chip_align_x, uld_chip_align_y, uld_chip_align_num)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET
         uld_put_date = EXCLUDED.uld_put_date,
         uld_chip_align_x = EXCLUDED.uld_chip_align_x,
         uld_chip_align_y = EXCLUDED.uld_chip_align_y,
         uld_chip_align_num = EXCLUDED.uld_chip_align_num"
    )
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial).bind(ld_pickup_date)
    .bind(uld_put_date).bind(cax).bind(cay).bind(canum)
    .execute(&mut **tx).await?;

    Ok(())
}

/// アラーム情報をDBに挿入
pub async fn regist_alarm_info(
    tx: &mut Transaction<'_, Postgres>,
    machine_id: i32,
    lot_name: &str,
    type_name: &str,
    unit_name: &str,
    value: &Value,
    ld_pickup_date: NaiveDateTime
) -> Result<(), sqlx::Error> {
    let hash_map = value.as_object().unwrap();
    let serial = hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let alarm = hash_map.get("alarm").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let column_name = format!("{}_alarm", unit_name.to_lowercase());

    sqlx::query(&format!(
        "INSERT INTO chipdata (machine_id, type_name, lot_name, serial, ld_pickup_date, {})
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT(lot_name, serial, ld_pickup_date, machine_id)
         DO UPDATE SET {} = EXCLUDED.{}",
        column_name, column_name, column_name
    ))
    .bind(machine_id).bind(type_name).bind(lot_name).bind(serial)
    .bind(ld_pickup_date).bind(alarm)
    .execute(&mut **tx).await?;

    Ok(())
}
