use serde::Deserialize;
use crate::model::{AppState, CNStockBasic, StockHist};
use crate::database;
use crate::error::SysError;

#[tauri::command]
pub async fn fuzzy_query(state: tauri::State<'_, AppState>, q: String, market:Option<Vec<String>>) -> Result<Vec<CNStockBasic>, SysError>  {
    let pool = &state.sqlite_pool;
    let market = market.unwrap_or(vec!["sh".to_owned(), "sz".to_owned()]);
    let t = database::fuzzy_query_basic(&pool, q.as_str(), market, 3).await;
    t.map_err(|e|e.into())
}

#[derive(Deserialize, Debug, Clone)]
pub struct SortOrder {
    pub field: String,
    pub order: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct SearchParams{
    pub symbol: Option<String>,
    pub market: Option<String>,
    #[serde(rename = "followOnly")]
    pub follow_only: bool,
}


#[tauri::command]
pub async fn query_stock_list(state: tauri::State<'_, AppState>,
                              sort_orders: Option<Vec<SortOrder>>,
                              search_params: SearchParams) -> Result<Option<Vec<StockHist>>, SysError>{

    println!("{:#?}", sort_orders);
    println!("{:#?}", search_params);
    let pool = &state.sqlite_pool;
    let market = vec!["sh".to_owned(), "sz".to_owned()];

    let q = search_params.symbol.unwrap_or("6".to_string());
    let t = database::fuzzy_query_basic(&pool, q.as_str(), market, 10).await;
    let today = chrono::Local::now().date_naive();
    match t {
        Ok(res) => {
            //println!("{:#?}", res);
            let t = res.iter().map(|x|{
                StockHist{
                    date: today.to_owned(),
                    symbol:x.symbol.to_owned(),
                    name: x.name.to_owned(),
                    market: x.market.to_owned(),
                    open: 0.0,
                    close: 0.0,
                    high: 0.0,
                    low: 0.0,
                    volume: 0.0,
                    turnover: 0.0,
                    price_change: 0.0,
                    pct_chg: 0.0,
                    amplitude: 0.0,
                    turnover_rate: 0.0,
                    kp_indicators: None,
                }
            }).collect::<Vec<StockHist>>();

            Ok(Some(t))
        }
        Err(e) => {
            println!("{:#?}", e);
            Err(e.into())
        }
    }
}