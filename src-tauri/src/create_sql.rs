use serde_json::Value;
use rusqlite::{Connection, Result,params};

//LDトレイピックアップ情報をDBに挿入するためのsql文を生成
pub fn create_u1_ph_sql(table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,timestamp:&str,value:&Value){
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let pf=hash_map.get("pf").and_then(|v| v.as_str()).unwrap_or("unknown");
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let arm_pos=hash_map.get("arm_pos").and_then(|v| v.as_str()).unwrap_or("unknown");
    let tpx=hash_map.get("tpx").and_then(|v| v.as_i64()).unwrap_or(0);      //ポケット座標X
    let tpy=hash_map.get("tpy").and_then(|v| v.as_i64()).unwrap_or(0);      //ポケット座標Y
    let tpax=hash_map.get("tpax").and_then(|v| v.as_i64()).unwrap_or(0);    //ポケット補正量X
    let tpay=hash_map.get("tpay").and_then(|v| v.as_i64()).unwrap_or(0);    //ポケット補正量Y

        // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO chipdata (lot_name, serial, type_name, ld_tray_time, ld_tray_pf, ld_tray_pos,ld_tray_pocket_x,ld_tray_pocket_y,ld_tray_align_x,ld_tray_align_y,machine_name)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(lot_name, serial)
        DO UPDATE SET 
        {c1} = excluded.{c1}, 
        {c2} = excluded.{c2}, 
        {c3} = excluded.{c3}, 
        {c4} = excluded.{c4}, 
        {c5} = excluded.{c5},
        {c6} = excluded.{c6}, 
        {c7} = excluded.{c7}, 
        {c8} = excluded.{c8}
        {c9} = excluded.{c9};", 
         c1 = "type_name", 
         c2 = "ld_tray_time", 
         c3 = "ld_tray_pf",
         c4 = "ld_tray_pos",
         c5 = "ld_tray_pocket_x",
         c6 = "ld_tray_pocket_y",
         c7 = "ld_tray_align_x",
         c8 = "ld_tray_align_y",
         c9 = "machine_name",
    );

    conn.execute(&sql, params![lot_name, serial, type_name, timestamp, pf, tray_arm,pocket_x,pocket_y,pocket_align_x,pocket_align_y,machine_name]); 

}

pub fn create_arm1_sql(table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,timestamp:&str,unit_name:&str,value:&Value){
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let pf=hash_map.get("pf").and_then(|v| v.as_str()).unwrap_or("unknown");
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let count=hash_map.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    // カラム名を動的に生成
    let unit=match unit_name{
        "U1"=>"LD",
        "U2"=>"DC1",
        "U3"=>"AC1",
        "U4"=>"AC2",
        "U5"=>"DC2",
        "U6"=>"IP",
        "U7"=>"ULD",
    };
    let column1 = format!("{}_ARM1_TIME", unit);
    let column2 = format!("{}_ARM1_PF", unit);
    let column3 = format!("{}_ARM1_COLLET_COUNT", unit);

    // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO chipdata (machine_name, type_name, lot_name, serial, {c1}, {c2}, {c3}, {c4})
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         {c1} = excluded.{c1}, 
         {c2} = excluded.{c2}, 
         {c3} = excluded.{c3},
         {c4} = excluded.{c4}, 
         {c5} = excluded.{c5};", 
         c1 = "machine_name", 
         c2 = "type_name", 
         c3 = column1, 
         c4 = column2, 
         c5 = column3
    );

    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, timestamp, pf, count]);

}

pub fn create_arm2_sql(table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,timestamp:&str,unit_name:&str,value:&Value){
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let pf=hash_map.get("pf").and_then(|v| v.as_str()).unwrap_or("unknown");
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let count=hash_map.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    // カラム名を動的に生成
    let unit=match unit_name{
        "U1"=>"LD",
        "U2"=>"DC1",
        "U3"=>"AC1",
        "U4"=>"AC2",
        "U5"=>"DC2",
        "U6"=>"IP",
        "U7"=>"ULD",
    };
    let column1 = format!("{}_ARM1_TIME", unit);
    let column2 = format!("{}_ARM1_PF", unit);
    let column3 = format!("{}_ARM1_COLLET_COUNT", unit);

    // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO chipdata (machine_name, type_name, lot_name, serial, {c1}, {c2}, {c3}, {c4})
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         {c1} = excluded.{c1}, 
         {c2} = excluded.{c2}, 
         {c3} = excluded.{c3},
         {c4} = excluded.{c4}, 
         {c5} = excluded.{c5};", 
         c1 = "machine_name", 
         c2 = "type_name", 
         c3 = column1, 
         c4 = column2, 
         c5 = column3
    );

    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, timestamp, pf, count]);

}

pub fn create_dc1_ph_sql(table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,timestamp:&str,value:&Value){
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let pf=hash_map.get("pf").and_then(|v| v.as_str()).unwrap_or("unknown");
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let ax=hash_map.get("ax").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量X
    let ay=hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量Y
    let at=hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量Θ

    // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO chipdata (machine_name, type_name, lot_name, serial, dc1_pre_time, dc1_pre_pf, dc1_pre_align_x,dc1_pre_align_y,dc1_pre_align_t)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6,?7,?8 ,?9)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         type_name=excluded.type_name,
         type_name = excluded.type_name, 
         {c1} = excluded.{c1}, 
         {c2} = excluded.{c2},
         {c3} = excluded.{c3},
         {c4} = excluded.{c4},
         {c5} = excluded.{c5},
         {c6} = excluded.{c6},
         {c7} = excluded.{c7};",
         c1 = "machine_name", 
         c2 = "type_name", 
         c3 = "dc1_pre_time", 
         c4 = "dc1_pre_pf", 
         c5 = "dc1_pre_align_x", 
         c6 = "dc1_pre_align_y", 
         c7 = "dc1_pre_align_t", 
    );

    //DBに登録
    conn.execute(&sql, params![lot_name, serial, type_name, timestamp, pf,ax,ay,at,]
    ) 

}

pub fn create_teststage_sql(table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,timestamp:&str,unit_name:&str,value:&Value){
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let pf=hash_map.get("pf").and_then(|v| v.as_str()).unwrap_or("unknown");
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let stage_serial=hash_map.get("st_serial").and_then(|v| v.as_str()).unwrap_or("unknown");
    let stage_count=hash_map.get("st_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let probe_serial=hash_map.get("pr_serial").and_then(|v| v.as_str()).unwrap_or("unknown");
    let probe_count=hash_map.get("pr_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let prx1=hash_map.get("prx1").and_then(|v| v.as_i64()).unwrap_or(0);
    let pry1=hash_map.get("pry1").and_then(|v| v.as_i64()).unwrap_or(0);
    let prx2=hash_map.get("prx2").and_then(|v| v.as_i64()).unwrap_or(0);
    let pry2=hash_map.get("pry2").and_then(|v| v.as_i64()).unwrap_or(0);
    let cx=hash_map.get("cx").and_then(|v| v.as_i64()).unwrap_or(0);
    let cy=hash_map.get("cy").and_then(|v| v.as_i64()).unwrap_or(0);
    let cz=hash_map.get("cz").and_then(|v| v.as_i64()).unwrap_or(0);

    // カラム名を動的に生成
    let unit=match unit_name{
        "U2"=>"DC1",
        "U3"=>"AC1",
        "U4"=>"AC2",
        "U5"=>"DC2",
        _=>return
    };

    // カラム名を動的に生成
    let column1 = format!("{}_TEST_TIME", unit);
    let column2 = format!("{}_TEST_PF", unit);
    let column3 = format!("{}_TEST_STAGE_SERIAL", unit);
    let column4 = format!("{}_TEST_STAGE_COUNT", unit);
    let column5 = format!("{}_TEST_PROBE_SERIAL", unit);
    let column6 = format!("{}_TEST_PROBE_COUNT", unit);
    let column7 = format!("{}_TEST_PROBE_1_X", unit);
    let column8 = format!("{}_TEST_PROBE_1_Y", unit);
    let column9 = format!("{}_TEST_PROBE_2_X", unit);
    let column10 = format!("{}_TEST_PROBE_2_Y", unit);
    let column11 = format!("{}_TEST_ALIGN_X", unit);
    let column12 = format!("{}_TEST_ALIGN_Y", unit);
    let column13 = format!("{}_TEST_ALIGN_T", unit);

        // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO chipdata (lot_name, serial, type_name, 
        {c1}, {c2}, {c3},{c4},{c5},{c6},{c7},{c8},{c9},{c10},{c11},{c12},{c13})
        VALUES (?1, ?2, ?3, ?4, ?5, ?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         type_name=excluded.type_name,
         {c1} = excluded.{c1}, 
         {c2} = excluded.{c2}, 
         {c3} = excluded.{c3},
         {c4} = excluded.{c4},
         {c5} = excluded.{c5},
         {c6} = excluded.{c6},
         {c7} = excluded.{c7},
         {c8} = excluded.{c8},
         {c9} = excluded.{c9},
         {c10} = excluded.{c10},
         {c11} = excluded.{c11},
         {c12} = excluded.{c12},
         {c13} = excluded.{c13};", 
         c1 = column1, 
         c2 = column2, 
         c3 = column3, 
         c4 = column4, 
         c5 = column5, 
         c6 = column6, 
         c7 = column7, 
         c8 = column8, 
         c9 = column9, 
         c10 = column10, 
         c11 = column11, 
         c12 = column12, 
         c13 = column13, 
    );

    //DBに登録
    match conn.execute(
        &sql, 
        params![
        lot_name, 
        serial, 
        type_name, 
        time, 
        pf,
        stage_serial,
        stage_count,
        probe_serial,
        probe_count,
        probe_align1_x,
        probe_align1_y,
        probe_align2_x,
        probe_align2_y,
        chip_align_x,
        chip_align_y,
        chip_align_t,
        ]
    ) {
        Err(e) => println!("SQL Error: {}", e),
        Ok(_) => {},
    }

}


pub fn create_uld_ph_sql(table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,timestamp:&str,value:&Value){
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let pf=hash_map.get("pf").and_then(|v| v.as_str()).unwrap_or("unknown");
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let ax=hash_map.get("ax").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量X
    let ay=hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量Y
    let at=hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量Θ

    // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO chipdata (machine_name, type_name, lot_name, serial, uld_pre_time, uld_pre_pf, uld_pre_align_x,uld_pre_align_y,uld_pre_align_t)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6,?7,?8 ,?9)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         type_name=excluded.type_name,
         type_name = excluded.type_name, 
         {c1} = excluded.{c1}, 
         {c2} = excluded.{c2},
         {c3} = excluded.{c3},
         {c4} = excluded.{c4},
         {c5} = excluded.{c5},
         {c6} = excluded.{c6},
         {c7} = excluded.{c7};",
         c1 = "machine_name", 
         c2 = "type_name", 
         c3 = "uld_pre_time", 
         c4 = "uld_pre_pf", 
         c5 = "uld_pre_align_x", 
         c6 = "uld_pre_align_y", 
         c7 = "uld_pre_align_t", 
    );

    //DBに登録
    conn.execute(&sql, params![lot_name, serial, type_name, timestamp, pf,ax,ay,at,]
    ) 

}

