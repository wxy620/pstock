use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::Value;

///
/// 历史数据
///
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct StockHist{
    pub date: NaiveDate,
    pub symbol: String, //股票代码
    pub name: String,
    pub market: String, //国内填写 SSE
    pub open: f32,
    pub close: f32,
    pub high: f32,
    pub low: f32,
    pub volume: f64, //成交量(股)
    pub turnover: f64,// 成交金额
    pub price_change: f32, //涨跌额
    pub pct_chg: f32, //涨跌幅 %
    pub amplitude: f32, //振幅 %
    pub turnover_rate: f32, //换手率 %
    pub kp_indicators: Option<String>, //序列化
}
/*
kp_indicators  example:
1,0,0,0,0,-1
1,-1 都表示出现了指标反映，比如十字星，在地位是正面的返回1，在高位时反面的返回-1，0表示没有出现特殊状况
指标顺序如下：

三只乌鸦，两只乌鸦，三同胞乌鸦， 十字星，十字晨星

 */


///
/// 分时
///
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct StockTimeSharing{
    pub date: Option<NaiveDate>,
    pub symbol: Option<String>,
    pub timestamp: i64,
    pub open: f32,
    pub close: f32,
    pub high: f32,
    pub low: f32,
    pub volume: f32,
    pub turnover: f32,
    pub new_price: Option<f32>,
}


#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct CNStockMarketCalendar{
    market: String,
    date: NaiveDate,
    is_open: bool,
}



#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct CNStockBasic{
    pub symbol: String,
    pub name: String,
    pub market: String,
    pub publish_date: Option<String>,
    pub is_delete: bool,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct StockFollow{
    pub symbol: String,
    pub market: String,
    pub follow_time: Option<NaiveDateTime>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct StockHistCrawlLog{
    pub symbol: String,
    pub market: String,
    pub adjust: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub crawl_time: NaiveDateTime,
}


#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct StockIndicator{
    pub symbol: String,
    pub market: String,
    pub adjust: String,
    pub date: NaiveDate,
    pub kline_indicator: Value,
    pub is_kline_special: bool,
    pub modify_time: NaiveDateTime,
    pub create_time: NaiveDateTime,
}



