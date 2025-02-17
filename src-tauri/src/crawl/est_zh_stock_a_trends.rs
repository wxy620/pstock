use anyhow::anyhow;
use chrono::{NaiveDateTime, TimeZone};
use reqwest_eventsource::{EventSource, Event};
use tokio::sync::mpsc;
use tokio_stream::StreamExt as _;
use crate::error::SysError;
use crate::model::StockTimeSharing;
use crate::model::s2s::IPayload;


#[derive(Debug, serde::Serialize, serde::Deserialize )]
struct EstSsePayload{
    pub rc: u8,
    pub rt: u16,
    pub svr: u64,
    pub lt: u8,
    pub full: u8,
    pub dlmkts: Option<String>,
    pub data: Option<EstSseData>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize )]
#[serde(untagged)]
enum  EstSseData{
    Details(EstDetailsData),
    Trends2(EstTrends2Data),
}

#[derive(Debug, serde::Serialize, serde::Deserialize )]
struct EstDetailsData{
    pub code: String,
    pub market: u8,
    pub details: Vec<String>,
}


#[derive(Debug, serde::Serialize, serde::Deserialize )]
struct EstTrends2Data{
    pub code: Option<String>,
    pub market: Option<u8>,
    pub name: Option<String>,
    pub trends: Vec<String>,
}



pub struct  EstSseClient<'a>{
    tx: &'a mpsc::Sender<IPayload>,
    es: Option<EventSource>,
    #[allow(dead_code)]
    symbol: String,
    #[allow(dead_code)]
    market: u8,
    #[allow(dead_code)]
    sse_type: u8,
}

impl <'a> EstSseClient<'a> {
    pub fn new( symbol: String,
            market: u8,
            sse_type: u8,
            tx: &'a mpsc::Sender<IPayload>) -> Self {

        let mut url = String::default();
        if sse_type == 0 {
            //分时
            url =format!("https://28.push2.eastmoney.com/api/qt/stock/trends2/sse\
    ?fields1=f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13,f14,f17\
    &fields2=f51,f52,f53,f54,f55,f56,f57,f58\
    &mpi=1000&ut=fa5fd1943c7b386f172d6893dbfba10b&secid={market}.{symbol}&ndays=1&iscr=0&iscca=0");
        } else if sse_type == 1 {
            // 逐笔
            url = format!("https://89.push2.eastmoney.com/api/qt/stock/details/sse?fields1=f1,f2,f3,f4\
    &fields2=f51,f52,f53,f54,f55\
    &mpi=2000&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2\
    &pos=-0&secid={market}.{symbol}");
        }

        let mut es = None;
        if url != String::default() {
            es = Some(EventSource::get(&url));
        }
        Self{
            symbol,
            market,
            sse_type,
            es,
            tx,
        }
    }

    pub async fn start(&mut self) -> Result<(), SysError> {
        if self.es.is_none() {
            return Err(anyhow!("init failed. no event source").into());
        }
        let es = self.es.as_mut().unwrap();
        let tx = self.tx;
        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => println!("Connection Open!"),
                Ok(Event::Message(message)) => {
                    if tx.is_closed() {
                        let err = SysError::EstSseError(5001,"tx isClosed".to_string());
                        return Err(err);
                    }
                    let payload = serde_json::from_str::<EstSsePayload>(
                        message.data.as_str()).unwrap();
                    let payload = Self::parse_message(&self.symbol,&payload).await;
                    if payload.is_some(){
                        let _ = tx.send(payload.unwrap()).await;
                    }
                },
                Err(err) => {
                    self.es.as_mut().unwrap().close();
                    log::error!("catch sse: {}", err);
                    //TODO 特殊error
                    return Err(err.into());
                }
            }
        }
        Ok(())
    }


    async fn parse_message(symbol: &str, payload: &EstSsePayload) -> Option<IPayload> {
        let full = payload.full;
        if let Some(ref payload) = payload.data{
            match payload{
                EstSseData::Trends2(data)=>{
                    //log::debug!("trends2: {:#?}",data);
                    let trends = data.trends.iter().map(|x|
                        Self::parse_hist_min(symbol, x)).collect::<Vec<StockTimeSharing>>();
                    if trends.len() > 1{
                        log::debug!(" est data len:{:?}, is_full:{}", trends.len(), full);
                    }
                    let trends = IPayload::KLineRT {data: trends, is_full: full, msg_id: None, timestamp: None };
                    Some(trends)
                },
                _ => None
            }
        }else{
            log::debug!("no data ! payload: {:?}",payload);
            None
        }

    }

    pub fn parse_hist_min(symbol: &str, line: &str) -> StockTimeSharing {
        //println!("{}", line);
        let line = line.split(",").collect::<Vec<&str>>();
        let target = line.get(0).unwrap().to_string();
        let fmt = "%Y-%m-%d %H:%M";

        let target = NaiveDateTime::parse_from_str(target.as_str(), fmt).unwrap();
        //UTC
        //let timestamp =  target.and_utc().timestamp_millis();
        //UTC +8
        // let hour = 3600;
        // let tz = chrono::FixedOffset::east_opt(8 * hour).unwrap();
        // let timestamp = target.and_local_timezone(tz).unwrap().timestamp_millis();
        // Local 取系统时区
        let timestamp = chrono::Local.from_local_datetime(&target).unwrap().timestamp_millis();
        let date = target.date();
        let open = line.get(1).unwrap().to_string().parse::<f32>().unwrap();
        let close = line.get(2).unwrap().to_string().parse::<f32>().unwrap();
        let high = line.get(3).unwrap().to_string().parse::<f32>().unwrap();
        let low = line.get(4).unwrap().to_string().parse::<f32>().unwrap();
        let volume = line.get(5).unwrap().to_string().parse::<f32>().unwrap();
        let turnover = line.get(6).unwrap().to_string().parse::<f32>().unwrap();
        let new_price = line.get(6).unwrap().to_string().parse::<f32>().unwrap();
        StockTimeSharing {
            date: Some(date),
            symbol: Some(symbol.to_string().clone()),
            timestamp,
            open,
            close,
            high,
            low,
            volume,
            turnover,
            new_price: Some(new_price),
        }
    }

}




