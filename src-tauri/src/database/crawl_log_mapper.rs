use chrono::{Local, NaiveDate};
use sqlx::{Pool, Sqlite};
use crate::model::StockHistCrawlLog;

pub async fn get_crawl_log(pool: &Pool<Sqlite>,
                                symbol: &str, market: &str, adjust: &str,)
            -> Result<Option<StockHistCrawlLog>, sqlx::Error> {
    let sql = format!(
        "select * from cn_stock_hist_crawl_log where symbol = '{symbol}' \
        and market = '{market}' and adjust = '{adjust}'"
    );
    sqlx::query_as(
        &sql.as_str()
    ).fetch_optional(pool).await

}

pub async fn save_or_update_crawl_log(pool: &Pool<Sqlite>,
                        symbol: &str, market: &str, adjust: &str,
                        crawl_start: &str, crawl_end: &str) -> Result<(), anyhow::Error>{
    let start = NaiveDate::parse_from_str(crawl_start, "%Y%m%d")?;
    let end  = NaiveDate::parse_from_str(crawl_end, "%Y%m%d")?;
    let now = Local::now().naive_local();

    let update_sql = r#"
        update `cn_stock_hist_crawl_log`
        set crawl_time = $1 , start_date = $2, end_date = $3
        where symbol = $4 AND market = $5 AND adjust = $6
    "#;
    let row = sqlx::query(update_sql)
        .bind(now)
        .bind(start)
        .bind(end)
        .bind(symbol)
        .bind(market)
        .bind(adjust)
        .execute(pool).await?.rows_affected();

    if row != 1 {
        let save_sql = r#"
             insert into
             `cn_stock_hist_crawl_log`(`symbol`, `market`, `adjust`, `start_date`, `end_date`, `crawl_time`)
             values( $1, $2, $3, $4, $5, $6);
        "#;
        let q =sqlx::query(save_sql)
            .bind(symbol)
            .bind(market)
            .bind(adjust)
            .bind(start)
            .bind(end)
            .bind(Local::now().naive_local());
        let _id = q.execute(pool).await?.last_insert_rowid();
    }

    Ok(())
}

