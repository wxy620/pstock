#[test]
pub fn test_sma() {
    let inReal: Vec<f64> = vec![
        1.087010, 1.087120, 1.087080, 1.087170, 1.087110, 1.087010, 1.087100, 1.087120, 1.087110,
        1.087080, 1.087000, 1.086630, 1.086630, 1.086610, 1.086630, 1.086640, 1.086650, 1.086650,
        1.086670, 1.086630,
    ];
    let (outReal, begin) = rust_ta_lib::wrapper::sma( 10,&inReal);
    for (index, value) in outReal.iter().enumerate() {
        println!("outs index {} = {}", begin + index as i32 + 1, value);
    }
}


use std::time::Instant;
use chrono::NaiveDate;
use crate::indicator::kline_pattern::{batch_check_kline_pattern,  check_kline_pattern, KLinePattern};
#[test]
pub fn test_batch_check_cdl3blackcrows() {
    let data_dir = "/Users/niko/Downloads/mystock_parquets";
    //000004_sz_hfq.parquet
    let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let now = Instant::now();
    let t = batch_check_kline_pattern(KLinePattern::CDL3BlackCrows, start, end,
                                      "000004", "sz", "hfq",
                                      data_dir
    ).unwrap();
    println!("{t:?}");
    println!("cost:{:?}ms", now.elapsed().as_millis());
}

#[test]
pub fn test_cdldarkcloudcover(){
    let data_dir = "/Users/niko/Downloads/mystock_parquets";
    let start = NaiveDate::from_ymd_opt(2023, 6, 25).unwrap();
    let end = NaiveDate::from_ymd_opt(2023,7,12).unwrap();
    let t = check_kline_pattern(
        KLinePattern::CDLDarkCloudCover, end,
        "000004",
        "sz", "hfq",
        data_dir
    ).unwrap();
    println!("{t:?}");
}

#[test]
pub fn test_cdl2crows(){
    let data_dir = "/Users/niko/Downloads/mystock_parquets";
    let start = NaiveDate::from_ymd_opt(2023, 6, 25).unwrap();
    let end = NaiveDate::from_ymd_opt(2023,7,12).unwrap();
    let t = check_kline_pattern(KLinePattern::CDL2Crows, end,
                                "000004",
                                "sz", "hfq", data_dir ).unwrap();
    println!("{t:?}");
}