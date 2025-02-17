mod payload;
mod stock;
mod app_state;

pub use stock::*;
pub use app_state::*;


///
/// C2S 客户端 -> 服务端
/// S2C 服务端 -> 客户端
///
pub mod c2s{

    pub const C2S_START_GLOBAL_EVENT: &'static str = "start_global";
    use crate::model::payload;
    pub use payload::InitializePayload;
}

pub mod s2s{
    use crate::model::payload;
    pub use payload::IPayload;
}

pub mod s2c{
    pub const S2C_EMMIT_GLOBAL_EVENT: &'static str = "global_event";
    pub const S2C_EMMIT_SYNC_KLINE_EVENT:&'static str  = "sync_kline_event";
}

