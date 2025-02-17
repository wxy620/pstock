use crate::error::SysError;
use crate::model::AppState;
use crate::model::CNStockMarketCalendar;

#[tauri::command]
pub async fn query_stock_calendar(state: tauri::State<'_, AppState>) -> Result<Vec<CNStockMarketCalendar>, SysError> {
    let pool = &state.sqlite_pool;
    let list:Vec<CNStockMarketCalendar>= sqlx::query_as(
        "select market, date, is_open from cn_stock_calendar"
    ).fetch_all(pool).await?;
    Ok(list)
}