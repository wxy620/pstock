use std::path::Path;
use anyhow::Context;
use chrono::NaiveDate;
use polars::prelude::*;
use serde_json::Value;
use sqlx::{Pool, Sqlite};
use crate::database;
use crate::indicator::{check_all_kp, KLinePattern};

pub async fn kline_pattern_job(pool: &Pool<Sqlite>,
                               symbol: &str,
                               market: &str,
                               adjust: &str,
                               start_date: NaiveDate,
                               end_date: NaiveDate,
                               data_dir: &str,)
                               -> Result<(), anyhow::Error>{

    let dir = Path::new(data_dir);
    let name = format!("{symbol}_{market}_{adjust}.parquet");
    let file_path = dir.join(name);
    //eager mode 像 python pandas
    let mut file = std::fs::File::open(file_path)?;
    let df = ParquetReader::new(&mut file).finish()?;
    let date_df = df.clone().lazy().filter(
        col("date").dt().date().lt_eq(lit(end_date)))
        .filter(
            col("date").dt().date().gt_eq(lit(start_date)))
        .select([
            col("date"),
        ]).collect()?;
    if date_df.is_empty(){
        let s = format!("没有股票行情数据,date: {} - {}", start_date, end_date);
        log::warn!("{}", s);
        anyhow::bail!(s)
    }else{
        let t: Vec<_> =
            date_df.column("date")?.date()?.as_date_iter().map(|date|{
            let date = date.context("parse error")?;
            let  t= check_all_kp(date,
                                 symbol,
                                 market,
                                 adjust,
                                 data_dir)?;

            Ok::<(NaiveDate, Value), anyhow::Error>(t)
        }).flatten().collect();
        let len = t.len();
        let mut res = vec![];
        for (day, val) in t{
            let is_sp = val.as_object().unwrap().iter()
                .filter(|&(_k, v)| v.as_i64().unwrap() != 0)
                .count() > 0;
            let v= database::save_or_update_indicator(
                pool,
                val,
                is_sp,
                day,
                symbol,
                market,
                adjust,
            ).await;

            if v.is_ok(){
                res.push(day);
            }
            log::info!("save indicator done. result:{:#?}, params:[day:{:#?}, symbol:{:#?}, market:{:#?}, adjust:{:#?}, is_sp:{:#?}]",
                    v, day, symbol, market, adjust, is_sp);
            if res.len() !=  len {
                anyhow::bail!("system error! 部分指标数据入库失败！请重新执行")
            }
        }
    }
    Ok(())
}

#[tokio::test]
pub async fn kline_pattern_job_test() {
    use sqlx:: sqlite::*;
    use std::str::FromStr;
    let resource_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let migrations_dir = resource_dir.join("migrations/").to_owned();
    let db_url = "sqlite:".to_owned() + &migrations_dir.join("sqlite.db").to_str().unwrap();
    let opts = SqliteConnectOptions::from_str(&db_url).unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    let pool =SqlitePoolOptions::new()
        .max_connections(5).connect_with(opts).await.unwrap();


    let data_dir = "/Users/niko/Downloads/mystock_parquets";
    let t = kline_pattern_job(&pool,
                      "000004",
                      "sz",
                      "hfq",
                      NaiveDate::from_ymd_opt(2024,12,01).unwrap(),
                      NaiveDate::from_ymd_opt(2024,12,02).unwrap(),
                      data_dir,
    ).await;


    let v = database::get_a_indicator(&pool,
                              NaiveDate::from_ymd_opt(2024,12,02).unwrap(),
                              "000004",
                              "sz",
                              "hfq",
    ).await;
    let v = v.unwrap().unwrap().kline_indicator;
    println!("{:#?}", v);
}