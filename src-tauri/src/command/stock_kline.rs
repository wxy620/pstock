use std::collections::BTreeMap;
use std::ops::Add;
use std::path::Path;
use anyhow::anyhow;
use chrono::{NaiveDate, NaiveDateTime};
use itertools::Itertools;
use polars::frame::DataFrame;
use polars::prelude::{col, lit, IntoLazy};
use sqlx::{Pool, Sqlite};
use crate::error::SysError;
use crate::model::{AppState, StockHist};
use crate::model::s2s::IPayload;
use tauri::{AppHandle, Emitter, Manager};
use tauri::ipc::Origin::Local;
use tokio::sync::mpsc;
use crate::crawl::est::EstSseClient;
use crate::crawl::est::*;
use crate::database;
use crate::model::s2c::S2C_EMMIT_SYNC_KLINE_EVENT;

#[tauri::command]
pub async fn un_sync_kline_data(state: tauri::State<'_, AppState>) -> Result<(), SysError> {
    let tx_store = &state.tx_store;
    let mut store = tx_store.write().await;
    let tx = store.remove("kline");
    if let Some(tx) = tx {
        if !tx.is_closed(){
            let _ = tx.send(IPayload::CloseChannel).await;
        }
        drop(tx);
    }
    Ok(())
}


#[tauri::command]
pub async  fn sync_kline_data(app_handle: AppHandle,
                               symbol: String) -> Result<(), SysError> {
    let state = &app_handle.state::<AppState>();
    let market = "SSE";
    let (new_tx, mut rx) = mpsc::channel(100);
    let tx_clone = new_tx.clone();

    //acquire rwlock
    {
        let mut store = state.tx_store.write().await;
        let tx = store.remove("kline");
        if let Some(tx) = tx {
            if !tx.is_closed(){
                let _t = tx.send(IPayload::CloseChannel).await;
            }
            drop(tx);
        }
        store.insert("kline".to_string(), new_tx);
    }
    //release rwlock

    let t1 =tauri::async_runtime::spawn(async move {
        while let Some(ref payload) = rx.recv().await {
            match  payload {
                // IPayload::KLine{ .. } => {
                //     let _ = app_handle.emit("sync_kline_event", payload).is_err_and(|_e|{
                //         log::error!("failed emit! payload:{:?} ", payload);
                //         false
                //     });
                // },
                IPayload::KLineRT {data, ..} =>{
                    log::debug!("receive kline rt data. len: {}", data.len());
                    let _ = app_handle.emit(S2C_EMMIT_SYNC_KLINE_EVENT, payload).is_err_and(|_e|{
                        log::error!("failed emit! payload:{:?} ", payload);
                        false
                    });
                }
                IPayload::CloseChannel =>{
                    log::info!("receive close event, close channel.");
                    break;
                },
                _ => ()
            }
        }
        log::debug!("close rx");
        //关闭
        rx.close();
    });

    let t2 = tauri::async_runtime::spawn(async move {
        let tx = tx_clone.to_owned();
        //listen sse
        let now = chrono::Local::now().time();
        use chrono::NaiveTime;
        let after = NaiveTime::from_hms_opt(15,0,0).unwrap();
        let is_after = now < after;

        let symbol = symbol.clone();
        let mut market = 0;
        //TODO 后面需要换更精确的库表
        if symbol.starts_with(&['6']){
            market = 1;
        }

        let mut est_sse = EstSseClient::new(
            symbol.to_owned(),
            market,
            0,
            &tx,
        );
        //堵塞在这里
        let t= est_sse.start().await;
        if t.is_err(){
            log::error!("end crawl trends, err:{:?}", t.unwrap_err().to_string());
            if !tx.is_closed(){
                let _ = tx.send(IPayload::CloseChannel).await;
            }
        }
        drop(tx);
    });

    let _ = tokio::join!(t1, t2);
    Ok(())
}


#[tauri::command]
pub async  fn query_kline_hist(state: tauri::State<'_, AppState>,
                               symbol: String,
                               name: String,
                               start_date: Option<NaiveDate>,
                               end_date: Option<NaiveDate>,
                               market:String, adjust: Option<String>)
    -> Result<Vec<StockHist>, SysError> {

    let end_date = end_date.or_else(|| Some(get_default_end_date()));
    let mut data_dir:String = Default::default();

    //acquire lock
    {
        let mut user_config = state.user_config.read().await;
        data_dir = user_config.data_dir.clone();
    }
    //release lock

    let adjust = adjust.unwrap_or("qfq".to_string());
    let pool = &state.sqlite_pool;
    let crawl_start = "19700101";
    let crawl_end = "20991231";

    let crawl_log =
        database::get_crawl_log(pool, &symbol[..], &market[..], &adjust[..]).await?;

    let mut to_crawl = true;
    if crawl_log.is_some() {
        let crawl_log = crawl_log.unwrap();
        to_crawl =  check_crawl_by_time(crawl_log.crawl_time)
    }

    let file_name = format!("{symbol}_{market}_{adjust}.parquet");
    let dir = Path::new(&data_dir);
    let df = if !to_crawl {
        let res = read_local_parquet(dir, file_name.as_str());
        if res.is_err(){
            let df = remote_crawl(&symbol[..],
                                  &name[..],
                                  &market[..],
                                  &adjust[..],
                                  crawl_start,
                                  crawl_end,
                                  dir,
                                  file_name.as_str()).await?;
            to_crawl = true;
            df
        }else{
            res?
        }
    }else{
        let df = remote_crawl(&symbol[..],
                              &name[..],
                              &market[..],
                              &adjust[..],
                              crawl_start,
                              crawl_end,
                              dir,
                              file_name.as_str()).await?;
        df
    };

    let mut lazy = df.lazy();
    if start_date.is_some(){
        let start_date = start_date.unwrap();
        lazy = lazy.filter(col("date").dt().date().gt_eq(lit(start_date)));
    }
    if end_date.is_some(){
        let end_date = end_date.unwrap();
        lazy = lazy.filter(col("date").dt().date().lt_eq(lit(end_date)));
    }
    let df = lazy.collect().map_err(|e| anyhow!(e))?;
    println!("{:#?}", df.tail(Some(1)));
    let v = parse_from_df_by_iter(df)?;

    if to_crawl{
        let _ = database::save_or_update_crawl_log(
            pool,
            &symbol[..],
            &market[..],
            &adjust[..],
            crawl_start,
            crawl_end,
        ).await?;
    }

    let v = build_kp_indicator(pool, start_date, end_date,
                       &symbol[..],
                       &market[..],
                       &adjust[..],
                                v).await?;
    Ok(v)
}

async fn build_kp_indicator(pool: &Pool<Sqlite>, start_date: Option<NaiveDate>,
                            end_date: Option<NaiveDate>,
                            symbol: &str, market: &str, adjust: &str, mut list: Vec<StockHist>) -> Result<Vec<StockHist>, anyhow::Error> {
    // 19700101 - 20991231
    let mut start = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let mut end = NaiveDate::from_ymd_opt(2099, 12, 31).unwrap();
    if start_date.is_some(){
        start = start_date.unwrap();
    }
    if end_date.is_some(){
        end = end_date.unwrap();
    }
    let vec= database::query_indicators(pool, start, end, symbol, market, adjust).await?;
    let map  = vec
        .into_iter()
        .map(|x| (x.date,  x))
        .collect::<BTreeMap<_, _>>();

    for t in list.iter_mut(){
        let ind = map.get(&t.date);
        if ind.is_some(){
            let ind = &ind.unwrap().kline_indicator;
            log::debug!("date:{}, kp_indicators: {}", t.date, ind.to_string());
            t.kp_indicators = Option::from(ind.to_string());
        }
    }
    Ok(list)
}



async  fn remote_crawl(symbol: &str, name: &str,
                       market: &str,adjust: &str,
                       crawl_start: &str,
                       crawl_end: &str,
                       dir: &Path, file_name: &str,) -> Result<DataFrame, anyhow::Error> {
    let list = crawl_stock_zh_a_hist(
        symbol,
        name,
        "daily",
        crawl_start,
        crawl_end,
        market,
        adjust,
    ).await?;
    let df =convert_to_df(&list[..])?;
    let _ = save_as_parquet(dir, file_name, df.clone())?;
    Ok(df)
}

/*
    @return true: before market close
 */
fn get_default_end_date() -> NaiveDate{
    let now = chrono::Local::now().naive_local();
    let market_close_time = now.date().and_hms_opt(15,0,0).unwrap();
    if now < market_close_time{
        now.add(chrono::Duration::days(-1)).date()
    }else{
        now.date()
    }
}

fn check_crawl_by_time(crawl_time: NaiveDateTime) -> bool {
    let now = chrono::Local::now().naive_local();
    let market_close_time = now.date().and_hms_opt(15,0,0).unwrap();
    if crawl_time > market_close_time {
        return false;
    }
    let yest_market_close_time =
    now.add(chrono::Duration::days(-1)).date().and_hms_opt(15,0,0).unwrap() ;
    if now < market_close_time  && crawl_time > yest_market_close_time {
        return false;
    }
    true
}