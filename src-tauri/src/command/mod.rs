mod stock_calendar;
mod stock_kline;
mod global;
mod stock_list;
mod backtest;

pub mod kline{
    pub use crate::command::stock_kline::sync_kline_data;
    pub use crate::command::stock_kline::un_sync_kline_data;
    pub use crate::command::stock_kline::query_kline_hist;
}


pub mod stock{
    pub use crate::command::stock_list::fuzzy_query;
    pub use crate::command::stock_list::query_stock_list;
}

pub use global::setup_global_monitor;
pub use backtest::call_my_sidecar;
