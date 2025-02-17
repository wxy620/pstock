mod est_zh_stock_a_trends;
mod est_zh_stock_a_basic;
mod est_zh_stock_a_hist;

pub mod est{
    use crate::crawl::est_zh_stock_a_trends;
    use crate::crawl::est_zh_stock_a_basic;
    use crate::crawl::est_zh_stock_a_hist;
    pub use est_zh_stock_a_trends::EstSseClient;
    pub use est_zh_stock_a_basic::est_stock_list;
    pub use est_zh_stock_a_hist::*;
}