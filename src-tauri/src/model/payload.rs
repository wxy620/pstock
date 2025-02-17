use crate::model::{StockHist, StockTimeSharing};

///
/// c2s
///
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InitializePayload{
    #[serde(rename = "msgId")]
    pub msg_id: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", tag = "event")]
pub enum IPayload {
    Dashboard{data: StockHist, msg_id:Option<String>},
    KLine{ data: StockHist,  data_type:String, msg_id:Option<String>, timestamp:Option<i64>},
    KLineRT{data: Vec<StockTimeSharing>,
        #[serde(rename = "isFull")]
        is_full:u8,
        #[serde(rename = "msgId")]
        msg_id:Option<String>,
        timestamp:Option<i64>
    },
    CloseChannel,
}

