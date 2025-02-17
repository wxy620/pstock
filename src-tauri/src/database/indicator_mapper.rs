use anyhow::anyhow;
use chrono::{Local, NaiveDate};
use serde_json::Value;
use sqlx::{Pool, Sqlite, SqlitePool};
use sqlx::Error::Database;
use sqlx::error::ErrorKind;
use crate::model::StockIndicator;

pub async fn get_a_indicator(
    pool: &SqlitePool,
    date: NaiveDate,
    symbol: &str,
    market: &str,
    adjust: &str,
)-> Result<Option<StockIndicator>, anyhow::Error> {

    let sql = r#"select * from `cn_stock_indicators`
        where `symbol` = $1 and `market` = $2 and `adjust` = $3 and `date` = $4"#;

    let v:Option<StockIndicator> = sqlx::query_as(sql)
        .bind(symbol)
        .bind(market)
        .bind(adjust)
        .bind(date)
        .fetch_optional(pool).await?;
    Ok(v)
}

pub async fn query_indicators(pool: &Pool<Sqlite>, start: NaiveDate, end: NaiveDate,
                            symbol: &str, market: &str, adjust: &str) -> Result<Vec<StockIndicator>, anyhow::Error> {

    let t: Vec<StockIndicator>= sqlx::query_as(r#"SELECT * FROM `cn_stock_indicators`
        WHERE `date` > $1 AND `date` < $2 AND `symbol` = $3 AND `market` = $4 AND `adjust` = $5
        order by `date` asc"#)
        .bind(start)
        .bind(end)
        .bind(symbol)
        .bind(market)
        .bind(adjust)
        .fetch_all(pool).await?;

    Ok(t)
}


pub async fn save_or_update_indicator(pool: &Pool<Sqlite>,
                            kline_indicator: Value,
                            is_kline_special: bool,
                            date: NaiveDate,
                            symbol: &str,
                            market: &str,
                            adjust: &str) -> Result<(), anyhow::Error> {

    let save_sql = r#"
        INSERT INTO `cn_stock_indicators`(
            `symbol`,
            `market`,
            `adjust`,
            `date`,
            `kline_indicator`,
            `is_kline_special`,
            `modify_time`)
        VALUES ($1,$2,$3,$4,$5,$6,$7 )
    "#;
    let q =sqlx::query( save_sql)
        .bind(symbol)
        .bind(market)
        .bind(adjust)
        .bind(date)
        .bind(&kline_indicator)
        .bind(is_kline_special)
        .bind(Local::now().naive_local());
    let id = q.execute(pool).await;

    if id.is_err(){
        let err = id.unwrap_err();
        let try_update = match &err {
            Database(e) => {
                 e.kind() ==  ErrorKind::UniqueViolation
            }
            _=> { false}
        };
        if try_update {
            let update_sql = r#"
            UPDATE `cn_stock_indicators`
            SET
                `kline_indicator` = $1,
                `is_kline_special` = $2,
                `modify_time` = $3
            WHERE
                `date` = $4,
                `symbol` = $5,
                `market` = $6,
                `adjust` = $7
            "#;

            let row = sqlx::query(update_sql)
                .bind(&kline_indicator)
                .bind(is_kline_special)
                .bind(Local::now().naive_local())
                .bind(date)
                .bind(symbol).bind(market).bind(adjust)
                .execute(pool).await?.rows_affected();

            if row !=1 {
                return Err(anyhow!("error"))
            }
        }else{
            log::error!("{}", err);
            return Err(err.into());
        }

    }

    Ok(())
}