use sqlx::{Pool, Sqlite};
use tokio::time::Instant;
use crate::model::CNStockBasic;


#[allow(dead_code)]
pub async fn update_stock_list(pool: &Pool<Sqlite>, stock_list: Vec<CNStockBasic>, is_force: bool) -> anyhow::Result<()> {
    if is_force {
        let count =
            sqlx::query("delete from cn_stock_basic")
                .execute(pool).await?.rows_affected();
        //TODO batch
        println!("force delete, row_affected:{}", count)
    }
    let start = Instant::now();
    for ref stock in stock_list {
        let rows: i64 = sqlx::query_scalar(
            "SELECT count(1) from cn_stock_basic where symbol = $1 and  is_delete = $2")
            .bind(stock.symbol.to_owned())
            .bind(stock.is_delete.to_owned())
            .fetch_one(pool).await?;
        if rows == 0{
            let q =sqlx::query(r#" insert into cn_stock_basic(`symbol`,
`name`, `market`, `publish_date`, `is_delete`) values( $1, $2, $3, $4, $5); "#
            )   .bind(stock.symbol.to_owned())
                .bind(stock.name.to_owned())
                .bind(stock.market.to_owned())
                .bind(stock.publish_date.clone())
                .bind(stock.is_delete.to_owned());
            let id = q.execute(pool).await?.last_insert_rowid();
            println!("last_insert_row id:{}", id)
        }

    }
    println!("finished save or update stocks. cost:{}ms", start.elapsed().as_millis());
    Ok(())
}

pub async fn fuzzy_query_basic(pool: &Pool<Sqlite>, q: &str, market:Vec<String>, limit: u8) -> anyhow::Result<Vec<CNStockBasic>> {


    // let market = market.iter().cloned()
    //     .map(|x| format!("'{x}'")).collect::<Vec<String>>().join(",");
    // let sql = format!(" \
    //     select * from cn_stock_basic where (symbol like  '%{q}%' OR  name like '%{q}%') \
    //     and market in ({market}) order by symbol asc limit 5 ;"
    // );

    let mut sqls = vec![];
    for x in market {
        let sql = format!(
            "select * from (select * from cn_stock_basic where (symbol like  '%{q}%' OR  name like '%{q}%') \
        and market = '{x}' order by symbol asc limit {limit}) "
        );
        sqls.push(sql);
    }
    let sql  = sqls.join( " UNION ");
    let result: Vec<CNStockBasic>= sqlx::query_as(&sql).fetch_all(pool).await?;
    log::debug!("fetch size:{:?}", result.len());
    Ok(result)

}


