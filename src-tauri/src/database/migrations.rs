use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use sqlx::{Pool, Sqlite};

pub async fn do_migrations(pool: &Pool<Sqlite>, migrations_dir:  &PathBuf) {

    /*
        1 create table
     */
    let tb_c_s_calendar = r#"
        CREATE TABLE IF NOT EXISTS cn_stock_calendar (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            is_open BOOLEAN NOT NULL DEFAULT true
        );
    "#;
    let uq_c_s_calendar = "CREATE UNIQUE INDEX IF NOT EXISTS unq_date ON cn_stock_calendar(date);";

    let tb_c_s_basic = r#"
        CREATE TABLE IF NOT EXISTS cn_stock_basic (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            name TEXT NOT NULL,
            market TEXT NOT NULL,
            publish_date TEXT,
            related_ids TEXT,
            is_delete INTEGER NOT NULL DEFAULT 0
        );
    "#;

    let uq_c_s_basic = "CREATE UNIQUE INDEX IF NOT EXISTS unq_symbol ON cn_stock_basic(symbol, is_delete);";


    let tb_c_s_follow = r#"
        CREATE TABLE IF NOT EXISTS user_stock_follow (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            market TEXT NOT NULL,
            follow_time TEXT NOT NULL
        )
    "#;

    let uq_c_s_follow = "CREATE UNIQUE INDEX IF NOT EXISTS unq_symbol ON user_stock_follow(symbol, market);";

    let tb_c_s_crawl_log = r#"
        CREATE TABLE IF NOT EXISTS cn_stock_hist_crawl_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            market TEXT NOT NULL,
            adjust TEXT NOT NULL,
            start_date TEXT NOT NULL,
            end_date TEXT NOT NULL,
            crawl_time TEXT NOT NULL
        )
    "#;

    let uq_c_s_crawl_log = r#"
    CREATE UNIQUE INDEX IF NOT EXISTS unq_symbol_market_adjust ON cn_stock_hist_crawl_log(symbol, market, adjust);
    "#;



    let tb_c_s_stock_indicators = r#"
        CREATE TABLE IF NOT EXISTS cn_stock_indicators (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            market TEXT NOT NULL,
            adjust TEXT NOT NULL,
            date TEXT NOT NULL,
            kline_indicator TEXT NOT NULL,
            is_kline_special BOOLEAN NOT NULL,
            create_time TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            modify_time TEXT NOT NULL
        )
    "#;


    let uq_c_s_stock_indicators = r#"
    CREATE UNIQUE INDEX IF NOT EXISTS unq_symbol_market_adjust_date ON cn_stock_indicators(symbol, market, adjust, date);
    "#;


    let result = sqlx::query(tb_c_s_calendar).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration. create table `cn_stock_calendar`");
    }
    let result = sqlx::query(uq_c_s_calendar).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration");
    }
    let result = sqlx::query(tb_c_s_basic).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration. create table `cn_stock_basic`");
    }
    let result = sqlx::query(uq_c_s_basic).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration");
    }

    let result = sqlx::query(tb_c_s_follow).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration. create table `cn_stock_basic`");
    }

    let result = sqlx::query(uq_c_s_follow).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration");
    }


    let result = sqlx::query(tb_c_s_crawl_log).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration. create table `cn_stock_hist_crawl_log`");
    }

    let result = sqlx::query(uq_c_s_crawl_log).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration");
    }


    let result = sqlx::query(tb_c_s_stock_indicators).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration. create table `cn_stock_indicators`");
    }

    let result = sqlx::query(uq_c_s_stock_indicators).execute(pool).await;
    if result.is_err() {
        panic!("Error while executing migration");
    }


    /*
        2 init data
     */
    let check_sql = "select count(*) from cn_stock_calendar";
    let mut count: i64 = 0;
    let result  = sqlx::query_scalar(check_sql).fetch_one(pool).await;
    if result.is_err() {
        panic!("Error while init stock calendar");
    }
    count = result.unwrap();
    println!("cn_stock_calendar rows: {:?}", count);
    if count == 0 {
        let path = migrations_dir.join("cn_stock_calendar.sql");
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let vec = reader.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
        let vec:Vec<Vec<String>> = vec.chunks(100).map(|x| x.into()).collect();
        for i in 0..vec.len() {
            let mut tx = pool.begin().await.unwrap();
            for  sql in vec[i].iter() {
                let _t = sqlx::query(sql).execute(&mut *tx).await;
            }
            tx.commit().await.unwrap();
        }
    }
}