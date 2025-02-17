use crate::database::{fuzzy_query_basic, save_or_update_crawl_log};

#[tokio::test]
pub async fn test_save_or_update_crawl_log() {
    use std::env;
    use std::path::Path;
    use std::str::FromStr;
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations_dir  = Path::new(&crate_dir)
        .join("migrations");

    let db_url = "sqlite:".to_owned() + &migrations_dir.join("sqlite.db").to_str().unwrap();
    let opts = SqliteConnectOptions::from_str(&db_url).unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    let pool =SqlitePoolOptions::new()
        .max_connections(5).connect_with(opts).await.unwrap();

    save_or_update_crawl_log(&pool,
                             "000004",
                             "sz",
                             "hfq",
                             "19700101",
                             "20991231"
    ).await.unwrap();
}




#[tokio::test]
pub async fn test_update_stock_list() {
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::path::Path;
    use std::env;
    use std::str::FromStr;
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations_dir  = Path::new(&crate_dir)
        .join("migrations");

    let db_url = "sqlite:".to_owned() + &migrations_dir.join("sqlite.db").to_str().unwrap();
    let opts = SqliteConnectOptions::from_str(&db_url).unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    let pool =SqlitePoolOptions::new()
        .max_connections(5).connect_with(opts).await.unwrap();

    // let d = CNStockBasic{
    //     symbol:"test".to_string(),
    //     is_delete:false,
    //     name:"test".to_string(),
    //     market: "test".to_string(),
    //     publish_date: Some("2024-12-12".to_string()),
    // };
    // let list = vec![d];
    // let t= update_stock_list(&pool, list, true).await;
    // println!("{:?}", t)

    fuzzy_query_basic(&pool, "600",
                      vec!["sh".to_owned(), "sz".to_owned()], 3).await.unwrap();

}
