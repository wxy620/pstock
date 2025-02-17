use std::io::Cursor;
use std::path::Path;
use std::str::FromStr;
use std::string::String;
use anyhow::Context;
use chrono::NaiveDate;
use itertools::izip;
use polars::datatypes::DataType;
use polars::prelude::{col, DataFrame, IntoLazy, JsonFormat, JsonReader, JsonWriter, ParquetReader, ParquetWriter, SerReader, SerWriter};
use reqwest::Client;
use serde_json::Value;
use crate::model::StockHist;


///
/// `start_date`: '20241212'
/// `end_date`: '20241212'
///
pub async fn crawl_stock_zh_a_hist(symbol:&str, name: &str, period: &str, start_date: &str, end_date: &str,
                                   market: &str, adjust: &str) ->Result<Vec<StockHist>, anyhow::Error>{

    //choice of {"qfq": "前复权", "hfq": "后复权", "": "不复权"}
    let adjust = match adjust {
        "qfq" => "1",
        "hfq" => "2",
        _ => "0",
    };

    //choice of {'daily', 'weekly', 'monthly'}
    let  period = match period{
        "daily" => "101",
        "weekly" => "102",
        "monthly" => "103",
        _ => "101",
    };

    let q_market = market;
    let q_market = if q_market == "sz".to_string(){
        "0"
    }else {
        "1"
    };

    let sec_id =  format!("{q_market}.{symbol}");
    let sec_id = sec_id.as_str();

    let url =  "https://push2his.eastmoney.com/api/qt/stock/kline/get";
    let t = chrono::Local::now().timestamp_millis();
    let t = t.to_string();
    let  query = vec![
        ("fields1", "f1,f2,f3,f4,f5,f6"),
        ("fields2", "f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61,f116"),
        ("ut", "7eea3edcaed734bea9cbfc24409ed989"),
        ("klt", period),
        ("fqt", adjust),
        ("secid", sec_id),
        ("beg", start_date),
        ("end", end_date),
        ("_", t.as_str()),
    ];

    let client = Client::new();
    let response = client.get(url)
        .query(&query)
        .send().await;


    //let mut response = response.map_err(|e| anyhow!("{:?}", e))?;
    let response = response?;
    let value = response.json::<Value>().await?;
    let value = value.get("data");
    let value = value.context("parse failed!")?
        .get("klines").context("parse failed!")?;

    let list :Result<Vec<_>, anyhow::Error> =
        value.as_array().context("parse failed")?.iter().map(|x|{
        // "日期",
        // "开盘",
        // "收盘",
        // "最高",
        // "最低",
        // "成交量",
        // "成交额",
        // "振幅",
        // "涨跌幅",
        // "涨跌额",
        // "换手率",
        let value = x.as_str().context("parse failed!")?;
        let data:Vec<&str>  = value.split(",").collect();
        let date = NaiveDate::parse_from_str(&data[0], "%Y-%m-%d")?;
        let open = f32::from_str(&data[1])?;
        let close = f32::from_str(&data[2])?;
        let high = f32::from_str(&data[3])?;
        let low = f32::from_str(&data[4])?;
        let volume = f64::from_str(&data[5])?;
        let turnover = f64::from_str(&data[6])?;
        let amplitude = f32::from_str(&data[7])?;
        let pct_chg = f32::from_str(&data[8])?;
        let price_change = f32::from_str(&data[9])?;
        let turnover_rate = f32::from_str(&data[10])?;
        let stock_hist = StockHist{
            date,
            symbol: symbol.to_owned(),
            name: name.to_owned(),
            market: market.to_owned(),
            open,
            close,
            high,
            low,
            volume,
            turnover,
            price_change,
            pct_chg,
            amplitude,
            turnover_rate,
            kp_indicators: None,
        };
        return Ok(stock_hist);

    }).collect();
    list
}

pub fn convert_to_df(list: &[StockHist]) -> Result<DataFrame, anyhow::Error> {
    let json = serde_json::to_string(list)?;
    let cursor = Cursor::new(json);
    let df = JsonReader::new(cursor)
        .finish()?;
    let df = df.lazy().with_columns([
        col("date").cast(DataType::Date),
    ]).collect()?;
    Ok(df)
}

/*
    性能差点
 */
pub fn parse_from_df(mut df: DataFrame) -> Result<Vec<StockHist>, anyhow::Error>{
    let mut json = Vec::<u8>::new();
    JsonWriter::new(&mut json)
        .with_json_format(JsonFormat::Json)
        .finish(&mut df)?;
    let rows = serde_json::from_slice::<Vec<StockHist>>(&json)?;
    Ok(rows)
}


/*
    使用的迭代器相比序列化效率高
 */
pub fn parse_from_df_by_iter(df: DataFrame) -> Result<Vec<StockHist>, anyhow::Error>{

    let symbol = df.column("symbol")?.str()?.iter();
    let name = df.column("name")?.str()?.iter();
    let market = df.column("market")?.str()?.iter();
    let open = df.column("open")?.f64()?.iter();
    let close = df.column("close")?.f64()?.iter();
    let high = df.column("high")?.f64()?.iter();
    let low = df.column("low")?.f64()?.iter();
    let volume = df.column("volume")?.f64()?.iter();
    let turnover = df.column("turnover")?.f64()?.iter();
    let price_change = df.column("price_change")?.f64()?.iter();
    let pct_chg = df.column("pct_chg")?.f64()?.iter();
    let amplitude = df.column("amplitude")?.f64()?.iter();
    let date = df.column("date")?.date()?.as_date_iter();
    let turnover_rate = df.column("turnover_rate")?.f64()?.iter();

    let combined = izip!(symbol,name,market,
                                              open,close,high,low,
                                            volume,turnover, price_change,pct_chg,amplitude,
                                        turnover_rate,date
                                        );


    //TODO
    // 魔法糖 `collect` auto change:
    // Vec<Result<_,Error>> to  Result<Vec<_>,Error>
    let res: Result<Vec<_>, anyhow::Error>= combined.map(|(symbol_, name_,market_,
                                       open_,close_,high_,low_,
                                       volume_,turnover_,
                                       price_change_,pct_chg_,amplitude_,
                                       turnover_rate_, date_,)|{

        let symbol_ = symbol_ as Option::<&str>;
        let symbol_ = symbol_.context("`symbol` parse failed!")?;
        let market_ = market_ as Option::<&str>;
        let market_ = market_.context("`market` parse failed!")?;
        let name_ = name_ as Option::<&str>;
        let name_ = name_.context("`name` parse failed!")?;
        // let open_= open_ as Option::<f64>;

        let t = StockHist{
            date: date_.unwrap(),
            symbol: symbol_.to_string(),
            name: name_.to_string(),
            market: market_.to_string() ,
            open: open_.unwrap_or(0.0) as f32,
            close: close_.unwrap_or(0.0) as f32,
            high: high_.unwrap_or(0.0) as f32,
            low: low_.unwrap_or(0.0) as f32,
            volume: volume_.unwrap_or(0.0),
            turnover: turnover_.unwrap_or(0.0),
            price_change: price_change_.unwrap_or(0.0) as f32,
            pct_chg: pct_chg_.unwrap_or(0.0) as f32,
            amplitude: amplitude_.unwrap_or(0.0) as f32,
            turnover_rate: turnover_rate_.unwrap_or(0.0) as f32,
            kp_indicators: None,
        };
        Ok(t)

     }).collect();
    res
}


pub fn save_as_parquet(dir: &Path, file_name: &str, mut df: DataFrame) -> Result<String, anyhow::Error>{
    let file_path = dir.join(file_name);
    let str  = file_path.to_str().context("no path")?.to_string();
    let mut file = std::fs::File::create(file_path)?;
    ParquetWriter::new(&mut file).finish(&mut df)?;
    Ok(str)
}


pub fn read_local_parquet(dir: &Path,file_name: &str) -> Result<DataFrame, anyhow::Error>{
    let  file_path = dir.join(file_name);
    //eager mode 像 python pandas
    let mut file = std::fs::File::open(file_path)?;
    let df = ParquetReader::new(&mut file).finish()?;
    Ok(df)
}


#[tokio::test]
pub async fn test_request_stock_list() {
    let name = "000004_sz_hfq.parquet".to_string();
    let list = crawl_stock_zh_a_hist(
        "000004",
        "国华网安",
        "daily",
        "19700101",
        "20500101",
        "sz",
        "hfq",
    ).await.unwrap();
    // println!("{:?}", list);
    let t = convert_to_df(&list[..]).unwrap();
    println!("{t:?}");
    // 6ms
    // let time = Instant::now();
    // let _res = parse_from_df_by_zip(t.clone()).unwrap();
    // println!("cost {:?}ms", time.elapsed().as_millis());
    // // 76ms
    // let time = Instant::now();
    // let _res = parse_from_df(t.clone()).unwrap();
    // println!("cost {:?}ms", time.elapsed().as_millis());

    let dir = Path::new("/Users/niko/Downloads/mystock_parquets");
    let _ = save_as_parquet(dir, name.as_str(), t).unwrap();

}

#[tokio::test]
pub async fn test_request_stock_hist() {
    use polars::prelude::*;
    let dir = Path::new("/Users/niko/Downloads/mystock_parquets");
    let name = "000004_sz_hfq.parquet".to_string();
    let df = read_local_parquet(dir, name.as_str()).unwrap();
    println!("{df:?}");

    let day_end = NaiveDate::parse_from_str("2025-01-19","%Y-%m-%d").unwrap();
    let day_start = NaiveDate::parse_from_str("2024-01-19","%Y-%m-%d").unwrap();

    let t= df.clone().lazy()
        .filter(col("date").dt().date().lt_eq(lit(day_end)))
        .filter(col("date").dt().date().gt_eq(lit(day_start)))
        .collect().unwrap();
    println!("{t:?}");

}

