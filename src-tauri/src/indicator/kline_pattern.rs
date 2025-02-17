use std::fmt::{Display, Formatter};
use crate::error::SysError;
use anyhow::{anyhow, Context};
use chrono::NaiveDate;
use polars::prelude::{col, lit, DataFrame, IntoLazy, ParquetReader, SerReader};
use std::path::Path;
use serde_json::Value;
use tauri::Manager;

impl Display for KLinePattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KLinePattern::CDLDarkCloudCover => write!(f, "cDLDarkCloudCover"),
            KLinePattern::CDL2Crows => write!(f, "cDL2Crows"),
            KLinePattern::CDL3BlackCrows => write!(f, "cDL3BlackCrows"),
            KLinePattern::CDLUpSideGap2Crows => write!(f, "cDLUpSideGap2Crows"),
            KLinePattern::CDLDojiStar => write!(f, "cDLDojiStar"),
            KLinePattern::CDLEveningDojiStar => write!(f, "cDLEveningDojiStar"),
            KLinePattern::CDLIdentical3Crows => write!(f, "cDLIdentical3Crows"),
            KLinePattern::Unknown => write!(f, "unknown"),
            _ => {write!(f, "unknown")}
        }

    }
}



impl From<String> for KLinePattern{
    fn from(kp: String) -> Self {
        match kp {
            kp if kp == "cDLDarkCloudCover" => KLinePattern::CDLDarkCloudCover,
            kp if kp == "cDL2Crows" => KLinePattern::CDL2Crows,
            kp if kp == "cDL3BlackCrows" => KLinePattern::CDL3BlackCrows,
            kp if kp == "cDLUpSideGap2Crows" => KLinePattern::CDLUpSideGap2Crows,
            kp if kp == "cDLDojiStar" => KLinePattern::CDLDojiStar,
            kp if kp == "cDLEveningDojiStar" => KLinePattern::CDLEveningDojiStar,
            kp if kp == "cDLIdentical3Crows" => KLinePattern::CDLIdentical3Crows,
            _ => KLinePattern::Unknown,
        }
    }
}

#[derive(Debug, Copy, Clone , serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KLinePattern{
    //乌云压顶
    CDLDarkCloudCover,
    CDLDarkCloudCoverSP(f64),
    //两只乌鸦
    CDL2Crows,
    //三只乌鸦
    CDL3BlackCrows,
    //向上跳空的两只乌鸦
    CDLUpSideGap2Crows,
    //十字星
    CDLDojiStar,
    //十字暮星
    CDLEveningDojiStar,
    CDLEveningDojiStarSP(f64),
    //三胞胎乌鸦
    CDLIdentical3Crows,

    Unknown,
}


pub fn check_all_kp(check_date: NaiveDate, symbol: &str,
                     market: &str, adjust: &str, data_dir: &str) -> Result<(NaiveDate, Value), anyhow::Error>{

    let kps = vec![
        KLinePattern::CDLDarkCloudCover,
        KLinePattern::CDL2Crows,
        KLinePattern::CDL3BlackCrows,
        KLinePattern::CDLUpSideGap2Crows,
        KLinePattern::CDLDojiStar,
        KLinePattern::CDLEveningDojiStar,
        KLinePattern::CDLIdentical3Crows,
    ];

    let  mut kv =   serde_json::Map::new();
    for kp in kps {
        let v = check_kline_pattern(kp,
                                    check_date,
                                    symbol,
                                    market,
                                    adjust,
                                    data_dir)?;
        let key = kp.to_string();
        kv.insert(key, Value::from(v.1));
    }
    Ok((check_date, Value::Object(kv)))
}

pub fn batch_check_kline_pattern(kp_type: KLinePattern,
                                 start_date: NaiveDate, end_date: NaiveDate,
                                 symbol: &str, market : &str, adjust: &str,
                                 data_dir: &str)
                                 -> Result<Vec<(NaiveDate,i32)>, anyhow::Error> {
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
    let v : Result<Vec<(NaiveDate, i32)>, anyhow::Error> = date_df.column("date")?
        .date()?.as_date_iter().map(|date|{
        let date = date.context("parse error")?;
        let  t= check_kline_pattern_inner(kp_type, date, df.clone())?;
        Ok(t)
    }).collect();
    v
}

pub fn check_kline_pattern(kp_type: KLinePattern, check_date: NaiveDate,
                           symbol: &str, market: &str, adjust: &str,
                           data_dir: &str ) -> Result<(NaiveDate,i32), anyhow::Error> {

    //TODO 从global_app_handle 获取配置
    let dir = Path::new(data_dir);
    let name = format!("{symbol}_{market}_{adjust}.parquet");
    let file_path = dir.join(name);
    //eager mode 像 python pandas
    let mut file = std::fs::File::open(file_path)?;
    let df = ParquetReader::new(&mut file).finish()?;

    let check_df= df.clone().lazy()
        .filter(col("date").dt().date().lt_eq(lit(check_date)))
        .collect()?;
    //talib 乌云压顶检测最少12天
    let check_df = check_df.tail(Some(14));
    if check_df.height() != 14 {
        return Err(SysError::BREAK(1001, "缺少必要参数,无法继续执行!".to_string()).into());
    }
    check_kline_pattern_inner(kp_type, check_date, check_df)
}


fn check_kline_pattern_inner(kp_type: KLinePattern, check_date: NaiveDate, check_df: DataFrame) -> Result<(NaiveDate,i32), anyhow::Error>{
    let  res = match kp_type{
        KLinePattern::CDLDarkCloudCover => {
            check_cdldarkcloudcover(check_date, check_df, 0.5)
        }
        KLinePattern::CDLDarkCloudCoverSP(t) => {
            check_cdldarkcloudcover(check_date, check_df, t)
        }
        KLinePattern::CDL2Crows => {
            check_cdl2crows(check_date, check_df)
        }
        KLinePattern::CDL3BlackCrows => {
            check_cdl3blackcrows(check_date, check_df)
        }
        KLinePattern::CDLUpSideGap2Crows => {
            check_cdlupsidegap2crows(check_date, check_df)
        }
        KLinePattern::CDLDojiStar => {
            check_cdldojistar(check_date, check_df)
        }
        KLinePattern::CDLEveningDojiStar => {
            check_cdleveningdojistar(check_date, check_df, 0.5)
        }
        KLinePattern::CDLEveningDojiStarSP(t) => {
            check_cdleveningdojistar(check_date, check_df, t)
        }
        KLinePattern::CDLIdentical3Crows => {
            check_cdlidentical3crows(check_date, check_df)
        }
        _ => Err(anyhow!("kline pattern not implemented")),
    };
    res
}

/*
3只乌鸦
 */
fn check_cdl3blackcrows(check_date: NaiveDate, df:DataFrame)
    -> Result<(NaiveDate,i32), anyhow::Error>{
    let (open, close, high, low ) = get_o_h_l_c_data(&df)?;
    let (out, _out_begin)  = rust_ta_lib::wrapper::cdl3blackcrows(
        &open,
        &high,
        &low,
        &close,
    );
    let last = *out.last().unwrap_or(&0);
    Ok((check_date,last))
}


/*
2只乌鸦
 */
fn check_cdl2crows(check_date: NaiveDate, df:DataFrame)
                              -> Result<(NaiveDate,i32), anyhow::Error>{
    let (open, close, high, low ) = get_o_h_l_c_data(&df)?;
    let (out, _out_begin)  = rust_ta_lib::wrapper::cdl2crows(
        &open,
        &high,
        &low,
        &close,
    );
    let last = *out.last().unwrap_or(&0);
    Ok((check_date, last))
}


/*
向上跳空的两只乌鸦
 */
fn check_cdlupsidegap2crows(check_date: NaiveDate, df:DataFrame)
                         -> Result<(NaiveDate,i32), anyhow::Error>{
    let (open, close, high, low ) = get_o_h_l_c_data(&df)?;
    let (out, _out_begin)  = rust_ta_lib::wrapper::cdlupsidegap2crows(
        &open,
        &high,
        &low,
        &close,
    );
    let last = *out.last().unwrap_or(&0);
    Ok((check_date, last))
}

//乌云压顶
fn  check_cdldarkcloudcover(check_date: NaiveDate, df:DataFrame, penetration: f64)
                                  -> Result<(NaiveDate,i32), anyhow::Error>{
    let (open, close, high, low ) = get_o_h_l_c_data(&df)?;
    let (out, _out_begin)  = rust_ta_lib::wrapper::cdldarkcloudcover(
        &open,
        &high,
        &low,
        &close,
        penetration,
    );
    let last = *out.last().unwrap_or(&0);
    Ok((check_date,last))
}

/*
十字星
 */
fn check_cdldojistar(check_date: NaiveDate, df:DataFrame,)-> Result<(NaiveDate,i32), anyhow::Error>{
    let (open, close, high, low ) = get_o_h_l_c_data(&df)?;
    let (out, _out_begin)  = rust_ta_lib::wrapper::cdldojistar(
        &open,
        &high,
        &low,
        &close,
    );
    let last = *out.last().unwrap_or(&0);
    Ok((check_date, last))
}

//十字暮星
fn check_cdleveningdojistar(check_date: NaiveDate, df:DataFrame, penetration: f64)-> Result<(NaiveDate,i32), anyhow::Error>{
    let (open, close, high, low ) = get_o_h_l_c_data(&df)?;
    let (out, _out_begin)  = rust_ta_lib::wrapper::cdleveningdojistar(
        &open,
        &high,
        &low,
        &close,
        penetration,
    );
    let last = *out.last().unwrap_or(&0);
    Ok((check_date, last))
}


//三胞胎乌鸦
fn check_cdlidentical3crows(check_date: NaiveDate, df:DataFrame)
                        -> Result<(NaiveDate,i32), anyhow::Error>{
    let (open, close, high, low ) = get_o_h_l_c_data(&df)?;
    let (out, _out_begin)  = rust_ta_lib::wrapper::cdlidentical3crows(
        &open,
        &high,
        &low,
        &close,
    );
    let last = *out.last().unwrap_or(&0);
    Ok((check_date,last))
}



fn get_o_h_l_c_data(df: &DataFrame) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), anyhow::Error>{

    let open: Vec<_> =  df.column("open")?.f64()?.iter().map(|x| {
        x.unwrap()
    } ).collect();

    let close:Vec<_> = df.column("close")?.f64()?.iter().map(|x| {
        x.unwrap()
    } ).collect();

    let high:Vec<_> = df.column("high")?.f64()?.iter().map(|x| {
        x.unwrap()
    } ).collect();

    let low:Vec<_> = df.column("low")?.f64()?.iter().map(|x| {
        x.unwrap()
    } ).collect();

    Ok((open, close, high, low))
}




