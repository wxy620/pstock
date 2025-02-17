use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use sqlx::{Pool, Sqlite};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tauri::AppHandle;
use tokio::sync::mpsc;
use crate::model::s2s::IPayload;
use crate::database;

#[allow(dead_code)]
#[derive(Debug)]
pub struct UserConfig{
    pub theme: String,
    pub language: String,
    pub data_dir: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SystemProps{
    pub buffer_size: usize,
}


#[allow(dead_code)]
#[derive(Debug)]
pub struct AppState {
    pub sqlite_pool: Pool<Sqlite>,
    pub user_config: RwLock<UserConfig>,
    pub sys_props: SystemProps,
    pub tx_store: RwLock<HashMap<String, mpsc::Sender<IPayload>>>,
    pub resource_dir: PathBuf,
}

pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[allow(dead_code)]
pub fn get_global_app_handle<'a>() -> &'a AppHandle {
    APP_HANDLE.get().unwrap()
}


pub fn default_data_dir() -> String{
    "/Users/niko/Downloads/mystock_parquets".to_string()
}

impl AppState {
    pub fn new(resource_dir: PathBuf) -> AppState {
        let resource_dir_clone = resource_dir.to_owned();
        let user_config = RwLock::new(UserConfig{
            theme: "dark".to_string(),
            language: "zh_CN".to_string(),
            data_dir: default_data_dir(),
        });
        let sys_props = SystemProps{buffer_size: 100};
        let tx_store = RwLock::new(HashMap::new());
        let sqlite_pool = tauri::async_runtime::block_on(async move {
            let sqlite_pool = init_sqlite3(resource_dir).await;
            sqlite_pool
        });
        AppState{
            sqlite_pool,
            user_config,
            sys_props,
            tx_store,
            resource_dir: resource_dir_clone,
        }
    }
}


pub async fn init_sqlite3(resource_dir: PathBuf) -> Pool<Sqlite>{
    let migrations_dir = resource_dir.join("migrations/").to_owned();
    let db_url = "sqlite:".to_owned() + &migrations_dir.join("sqlite.db").to_str().unwrap();
    let opts = SqliteConnectOptions::from_str(&db_url).unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    let pool =SqlitePoolOptions::new()
        .max_connections(5).connect_with(opts).await.unwrap();

    //init local db
    database::do_migrations(&pool, &migrations_dir).await;

    pool
}
