use serde_json::Value;
use rusqlite::{Connection, Result,params};
use tokio::net::tcp::ReuniteError;

//LDトレイピックアップ情報をDBに挿入するためのsql文を生成
pub fn regist_u1_tr_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);             //シリアル番号
    let wano=hash_map.get("wano").and_then(|v| v.as_i64()).unwrap_or(0);                 //wa番号
    let wax=hash_map.get("wax").and_then(|v| v.as_i64()).unwrap_or(0);                  //waX座標
    let way=hash_map.get("way").and_then(|v| v.as_i64()).unwrap_or(0);                  //waY座標
    let trayid=hash_map.get("trayid").and_then(|v| v.as_str()).unwrap_or("unknown");    //トレイID
    let trayarm=hash_map.get("trayarm").and_then(|v| v.as_str()).unwrap_or("unknown");  //トレイアーム位置
    let px=hash_map.get("px").and_then(|v| v.as_i64()).unwrap_or(0);                     //ポケット座標X
    let py=hash_map.get("py").and_then(|v| v.as_i64()).unwrap_or(0);                     //ポケット座標Y
    let pax=hash_map.get("pax").and_then(|v| v.as_i64()).unwrap_or(0);                   //ポケット補正量X
    let pay=hash_map.get("pay").and_then(|v| v.as_i64()).unwrap_or(0);                   //ポケット補正量Y
    let date=hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("unknown");        //ピックアップ時刻

        // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, 
        WANO, WAX, WAY, LD_PICKUP_DATE, LD_TRAYID, LD_TRAY_ARM, 
        LD_TRAY_POCKET_X, LD_TRAY_POCKET_Y, LD_TRAY_ALIGN_X, LD_TRAY_ALIGN_Y)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        ON CONFLICT(lot_name, serial)
        DO UPDATE SET 
        MACHINE_NAME = excluded.MACHINE_NAME, 
        TYPE_NAME = excluded.TYPE_NAME, 
        WANO = excluded.WANO, 
        WAX = excluded.WAX, 
        WAY = excluded.WAY,
        LD_PICKUP_DATE = excluded.LD_PICKUP_DATE, 
        LD_TRAYID = excluded.LD_TRAYID, 
        LD_TRAY_ARM = excluded.LD_TRAY_ARM,
        LD_TRAY_POCKET_X = excluded.LD_TRAY_POCKET_X,
        LD_TRAY_POCKET_Y = excluded.LD_TRAY_POCKET_Y,
        LD_TRAY_ALIGN_X = excluded.LD_TRAY_ALIGN_X,
        LD_TRAY_ALIGN_Y = excluded.LD_TRAY_ALIGN_Y;");

    conn.execute(&sql, params![machine_name,type_name,lot_name,serial,wano,wax,way,date,trayid,trayarm,px,py,pax,pay])?;
    Ok(()) 

}

pub fn regist_arm1_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,unit_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
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
        _=>{return Err(rusqlite::Error::InvalidQuery);}
    }; 
    let column = format!("{}_ARM1_COLLET", unit);

    // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, {column})
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         MACHINE_NAME = excluded.MACHINE_NAME, 
         TYPE_NAME = excluded.TYPE_NAME, 
         {column} = excluded.{column};");

    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, count])?;
    Ok(())

}

pub fn regist_arm2_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,unit_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
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
        _=>{return Err(rusqlite::Error::InvalidQuery);}
    }; 
    let column = format!("{}_ARM2_COLLET", unit);

    // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, {column})
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         MACHINE_NAME = excluded.MACHINE_NAME, 
         TYPE_NAME = excluded.TYPE_NAME, 
         {column} = excluded.{column};");

    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, count])?;
    Ok(())

}


pub fn regist_ph_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,unit_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let ax=hash_map.get("ax").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量X
    let ay=hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量Y
    let at=hash_map.get("at").and_then(|v| v.as_i64()).unwrap_or(0);    //補正量Θ

    // カラム名を動的に生成
    let unit=match unit_name{
        "U2"=>"DC1",
        "U7"=>"ULD",
        _ => {return Err(rusqlite::Error::InvalidQuery);}
    }; 
    let c1 = format!("{}_PRE_ALIGN_X", unit);
    let c2 = format!("{}_PRE_ALIGN_Y", unit);
    let c3 = format!("{}_PRE_ALIGN_T", unit);

    // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, {c1}, {c2}, {c3})
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         MACHINE_NAME = excluded.MACHINE_NAME, 
         TYPE_NAME = excluded.TYPE_NAME, 
         {c1} = excluded.{c1}, 
         {c2} = excluded.{c2}, 
         {c3} = excluded.{c3};");

    //DBに登録
    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, ax,ay,at])?;

    Ok(())

}

pub fn regist_ts_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,unit_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let stage_serial=hash_map.get("stage_serial").and_then(|v| v.as_str()).unwrap_or("unknown");
    let stage_count=hash_map.get("stage_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let stage_z=hash_map.get("stage_z").and_then(|v| v.as_i64()).unwrap_or(0);
    let pin_z=hash_map.get("pin_z").and_then(|v| v.as_i64()).unwrap_or(0);
    let probe_serial=hash_map.get("probe_serial").and_then(|v| v.as_str()).unwrap_or("unknown");
    let probe_count=hash_map.get("probe_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let prx1=hash_map.get("probe_x1").and_then(|v| v.as_i64()).unwrap_or(0);
    let pry1=hash_map.get("probe_y1").and_then(|v| v.as_i64()).unwrap_or(0);
    let prx2=hash_map.get("probe_x2").and_then(|v| v.as_i64()).unwrap_or(0);
    let pry2=hash_map.get("probe_y2").and_then(|v| v.as_i64()).unwrap_or(0);
    let ax=hash_map.get("ax").and_then(|v| v.as_i64()).unwrap_or(0);
    let ay=hash_map.get("ay").and_then(|v| v.as_i64()).unwrap_or(0);
    let at=hash_map.get("at").and_then(|v| v.as_i64()).unwrap_or(0);
    let bin=hash_map.get("bin").and_then(|v| v.as_i64()).unwrap_or(-1);

    // カラム名を動的に生成
    let unit=match unit_name{
        "U2"=>"DC1",
        "U3"=>"AC1",
        "U4"=>"AC2",
        "U5"=>"DC2",
        _ => {return Err(rusqlite::Error::InvalidQuery);}
    };

    // カラム名を動的に生成
    let c1 = format!("{}_STAGE_SERIAL", unit);
    let c2 = format!("{}_STAGE_COUNT", unit);
    let c3 = format!("{}_PROBE_SERIAL", unit);
    let c4 = format!("{}_PROBE_COUNT", unit);
    let c5 = format!("{}_PROBE_X1", unit);
    let c6 = format!("{}_PROBE_Y1", unit);
    let c7 = format!("{}_PROBE_X2", unit);
    let c8 = format!("{}_PROBE_Y2", unit);
    let c9 = format!("{}_STAGE_Z", unit);
    let c10 = format!("{}_PIN_Z", unit);
    let c11 = format!("{}_CHIP_ALIGN_X", unit);
    let c12 = format!("{}_CHIP_ALIGN_Y", unit);
    let c13 = format!("{}_CHIP_ALIGN_T", unit);
    let c14 = format!("{}_TEST_BIN", unit);

    // SQL文字列をformat!で構築
    let sql = format!(
        "INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, 
        {c1}, {c2}, {c3},{c4},{c5},{c6},{c7},{c8},{c9},{c10},{c11},{c12},{c13},{c14})
        VALUES (?1, ?2, ?3, ?4, ?5, ?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         MACHINE_NAME=excluded.MACHINE_NAME,
         TYPE_NAME=excluded.TYPE_NAME,
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
         {c13} = excluded.{c13},
         {c14} = excluded.{c14};");

    //DBに登録
    conn.execute(
        &sql, 
        params![
        machine_name,
        type_name,
        lot_name, 
        serial, 
        stage_serial,
        stage_count,
        probe_serial,
        probe_count,
        prx1,
        pry1,
        prx2,
        pry2,
        stage_z,
        pin_z,
        ax,
        ay,
        at,
        bin
        ]
    )?;

    Ok(()) 

}

pub fn regist_ip_ts_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let stage_count=hash_map.get("stage_count").and_then(|v| v.as_i64()).unwrap_or(0);

    // SQL文字列をformat!で構築
    let sql = 
        format!("INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, IP_STAGE_COUNT)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         MACHINE_NAME = excluded.MACHINE_NAME, 
         TYPE_NAME = excluded.TYPE_NAME, 
         IP_STAGE_COUNT = excluded.IP_STAGE_COUNT;");

    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, stage_count])?;
    Ok(())
}

pub fn regist_ip_surf_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let bin=hash_map.get("bin").and_then(|v| v.as_i64()).unwrap_or(0);

    // SQL文字列をformat!で構築
    let sql = 
        format!("INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, IP_SURF_BIN)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         MACHINE_NAME = excluded.MACHINE_NAME, 
         TYPE_NAME = excluded.TYPE_NAME, 
         IP_SURF_BIN = excluded.IP_SURF_BIN;");

    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, bin])?;
    Ok(())
}

pub fn regist_ip_back_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);
    let bin=hash_map.get("bin").and_then(|v| v.as_i64()).unwrap_or(0);

    // SQL文字列をformat!で構築
    let sql = 
        format!("INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, IP_BACK_BIN)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET 
         MACHINE_NAME = excluded.MACHINE_NAME, 
         TYPE_NAME = excluded.TYPE_NAME, 
         IP_BACk_BIN = excluded.IP_BACK_BIN;");

    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, bin])?;
    Ok(())
}

//ULDポケットアライメント情報をDBに登録
pub fn regist_uld_pocket_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);             //シリアル番号
    let trayid=hash_map.get("trayid").and_then(|v| v.as_str()).unwrap_or("unknown");    //トレイID
    let px=hash_map.get("px").and_then(|v| v.as_i64()).unwrap_or(0);                     //ポケット座標X
    let py=hash_map.get("py").and_then(|v| v.as_i64()).unwrap_or(0);                     //ポケット座標Y
    let pax=hash_map.get("pax").and_then(|v| v.as_i64()).unwrap_or(0);                   //ポケット補正量X
    let pay=hash_map.get("pay").and_then(|v| v.as_i64()).unwrap_or(0);                   //ポケット補正量Y

        // SQL文字列をformat!で構築
    let sql = 
        format!("INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, 
        ULD_TRAYID, ULD_POCKET_X, ULD_POCKET_Y, ULD_POCKET_ALIGN_X, ULD_POCKET_ALIGN_Y)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(lot_name, serial)
        DO UPDATE SET 
        MACHINE_NAME = excluded.MACHINE_NAME,
        TYPE_NAME = excluded.TYPE_NAME,
        ULD_TRAYID = excluded.ULD_TRAYID, 
        ULD_POCKET_X = excluded.ULD_POCKET_X,
        ULD_POCKET_Y = excluded.ULD_POCKET_Y,
        ULD_POCKET_ALIGN_X = excluded.ULD_POCKET_ALIGN_X,
        ULD_POCKET_ALIGN_Y = excluded.ULD_POCKET_ALIGN_Y;");

    conn.execute(&sql, params![machine_name,type_name,lot_name,serial,trayid,px,py,pax,pay])?;
    Ok(()) 

}

//ULD挿入後チップアライメント情報をDBに登録
pub fn regist_uld_chip_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    //pf,serial,arm_pos,tpx,tpy,tpax,tpayを抜き出す
    let hash_map=value.as_object().unwrap();
    let serial=hash_map.get("serial").and_then(|v| v.as_i64()).unwrap_or(0);             //シリアル番号
    let px=hash_map.get("px").and_then(|v| v.as_i64()).unwrap_or(0);                     //ポケット座標X
    let py=hash_map.get("py").and_then(|v| v.as_i64()).unwrap_or(0);                     //ポケット座標Y
    let cax=hash_map.get("cax").and_then(|v| v.as_i64()).unwrap_or(0);                   //ポケット補正量X
    let cay=hash_map.get("cay").and_then(|v| v.as_i64()).unwrap_or(0);                   //ポケット補正量Y
    let date=hash_map.get("date").and_then(|v| v.as_str()).unwrap_or("unknown");        //挿入時刻

    //lot_name,serialのULD_CHIP_ALIGN_NUMを取得する
    //nullであれば、align_numを0にする、数値であれば+1する
    let get_align_num_sql=
        format!("SELECT ULD_CHIP_ALIGN_NUM FROM {} WHERE LOT_NAME='{}' AND SERIAL={}",table_name,lot_name,serial);

    let align_num: i64 = conn.query_row(&get_align_num_sql, [], |row| {
        row.get::<_, Option<i64>>(0)
    })
    .unwrap_or(None)
    .map(|num| num + 1)
    .unwrap_or(1);

    // SQL文字列をformat!で構築
    let sql = 
        format!("INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, 
        ULD_POCKET_X, ULD_POCKET_Y, ULD_CHIP_ALIGN_X, ULD_CHIP_ALIGN_Y,ULD_PUT_DATE,ULD_CHIP_ALIGN_NUM)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(lot_name, serial)
        DO UPDATE SET 
        MACHINE_NAME = excluded.MACHINE_NAME,
        TYPE_NAME = excluded.TYPE_NAME,
        ULD_POCKET_X = excluded.ULD_POCKET_X,
        ULD_POCKET_Y = excluded.ULD_POCKET_X,
        ULD_CHIP_ALIGN_X = excluded.ULD_CHIP_ALIGN_X,
        ULD_CHIP_ALIGN_Y = excluded.ULD_CHIP_ALIGN_Y,
        ULD_PUT_DATE = excluded.ULD_PUT_DATE,
        ULD_CHIP_ALIGN_NUM = excluded.ULD_CHIP_ALIGN_NUM;");

    conn.execute(&sql, params![machine_name,type_name,lot_name,serial,px,py,cax,cay,date,align_num])?;
    Ok(()) 

}

pub fn regist_alarm_info(conn:&Connection,table_name:&str,machine_name:&str,lot_name:&str,type_name:&str,unit_name:&str,value:&Value)->Result<(),rusqlite::Error>{
    let hash_map=value.as_object().unwrap();
    let alarm_num=hash_map.get("alarm_num").and_then(|v| v.as_i64()).unwrap_or(0);

    // カラム名を動的に生成
    let unit=match unit_name{
        "U1"=>"LD",
        "U2"=>"DC1",
        "U3"=>"AC1",
        "U4"=>"AC2",
        "U5"=>"DC2",
        "U6"=>"IP",
        "U7"=>"ULD",
        _=>{return Err(rusqlite::Error::InvalidQuery);}
    }; 
    let column = format!("{}_ALARM", unit);

    //シリアルリストをVec<i32>として受け取る
    let serial_list: Vec<i32> = hash_map
        .get("serial")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64())
                .map(|v| v as i32)
                .collect()
        })
        .unwrap_or_default();

    //最初の0でない要素を取得
    let serial = match serial_list.iter().find(|&&s| s != 0) {
        Some(&s) => s,
        None => {
            //全要素が0の場合は登録せずにリターン
            return Ok(());
        }
    };

    // SQL文字列をformat!で構築
    let sql =
        format!("INSERT INTO {table_name} (MACHINE_NAME, TYPE_NAME, LOT_NAME, SERIAL, {column})
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(lot_name, serial)
         DO UPDATE SET
         MACHINE_NAME = excluded.MACHINE_NAME,
         TYPE_NAME = excluded.TYPE_NAME,
         {column} = excluded.{column};");

    conn.execute(&sql, params![machine_name,type_name,lot_name, serial, alarm_num])?;
    Ok(())
}

