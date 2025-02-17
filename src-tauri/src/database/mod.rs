mod migrations;
mod stock_mapper;
mod indicator_mapper;
mod crawl_log_mapper;
mod test;

pub use crate::database::migrations::do_migrations;

pub use crate::database::stock_mapper::update_stock_list;
pub use crate::database::stock_mapper::fuzzy_query_basic;


pub use crate::database::crawl_log_mapper::save_or_update_crawl_log;
pub use crate::database::crawl_log_mapper::get_crawl_log;


pub use crate::database::indicator_mapper::save_or_update_indicator;
pub use crate::database::indicator_mapper::get_a_indicator;
pub use crate::database::indicator_mapper::query_indicators;