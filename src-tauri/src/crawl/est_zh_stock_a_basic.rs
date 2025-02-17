use anyhow::Context;
use reqwest::Client;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use crate::error::SysError;
use crate::model::CNStockBasic;

fn default_market() -> String {
    "sh".to_string()
}

fn deserialize_from_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Value= Deserialize::deserialize(deserializer)?;
    if v.is_number(){
        let s = v.as_i64().unwrap().to_string();
        let year = &s[0..4];
        let month = &s[4..6];
        let day = &s[6..8];
        let str = String::from(year) + "-" + month + "-" + day ;
        Ok(Some(str))
    }else{
        Ok(None)
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EstStockBasic{
    #[serde(rename = "f12")]
    symbol: String,
    #[serde(rename = "f14")]
    name: String,
    #[serde(default="default_market")]
    market: String,
    #[serde(rename = "f26", deserialize_with = "deserialize_from_str")]
    publish_date: Option<String>,
}


pub async fn est_stock_list()-> Result<Vec<CNStockBasic>, SysError>{
    let mut list =  request_sh_stock_list().await?.iter()
        .map(|s|{
            CNStockBasic{
                symbol: s.symbol.to_owned(),
                name: s.name.to_owned(),
                market: s.market.to_owned(),
                publish_date: s.publish_date.to_owned(),
                is_delete: false
            }
        }).collect::<Vec<CNStockBasic>>();

    let mut sz_list = request_sz_stock_list().await?.iter()
        .map(|s|{
            CNStockBasic{
                symbol: s.symbol.to_owned(),
                name: s.name.to_owned(),
                market: s.market.to_owned(),
                publish_date: s.publish_date.to_owned(),
                is_delete: false
            }
        }).collect::<Vec<CNStockBasic>>();

    list.append(&mut sz_list);
    Ok(list)
}


async fn request_sh_stock_list()->Result<Vec<EstStockBasic>, anyhow::Error> {

    let t = chrono::Local::now().timestamp_millis();
    let t = t.to_string();
    let url =  "http://80.push2.eastmoney.com/api/qt/clist/get";
    let query = vec![
        ("pn","1"),
        ("pz","50000"),
        ("po", "1"),
        ("np", "1"),
        ("ut", "bd1d9ddb04089700cf9c27f6f7426281"),
        ("fltt", "2"),
        ("invt", "2"),
        ("fid", "f3"),
        ("fs", "m:1 t:2,m:1 t:23"),
        ("fields", "f12,f14,f26"),
        ("_", t.as_str()),
    ];
    let client = Client::new();
    let response = client.get(url)
        .query(&query)
        .send().await;

    let mut vec = vec![];

    let response = response?;
    println!("{:?}", response);
    let value = response.json::<Value>().await?;
    let value = value.get("data");
    let value = value.context("parse failed!")?
        .get("diff").context("parse failed!")?;
    //println!("{:?}", value);
    if value.is_array(){
        for x in value.as_array().unwrap(){
            let t = serde_json::from_value::<EstStockBasic>(x.to_owned())?;
            vec.push(t);
        }
    }

    Ok(vec)
}



async fn request_sz_stock_list()->Result<Vec<EstStockBasic>, anyhow::Error> {

    let t = chrono::Local::now().timestamp_millis();
    let t = t.to_string();
    let url =  "http://80.push2.eastmoney.com/api/qt/clist/get";

    let query = vec![
        ("pn","1"),
        ("pz","50000"),
        ("po", "1"),
        ("np", "1"),
        ("ut", "bd1d9ddb04089700cf9c27f6f7426281"),
        ("fltt", "2"),
        ("invt", "2"),
        ("fid", "f3"),
        ("fs", "m:0 t:6,m:0 t:80"),
        ("fields", "f12,f14,f26"),
        ("_", t.as_str()),
    ];

    let client = Client::new();
    let response = client.get(url)
        .query(&query)
        .send().await;

    let mut vec = vec![];

    let response = response?;
    println!("{:?}", response);
    let value = response.json::<Value>().await?;
    let value = value.get("data");
    let value = value.context("parse failed!")?
        .get("diff").context("parse failed!")?;
    //println!("{:?}", value);
    if value.is_array(){
        for x in value.as_array().unwrap(){
            let mut t = serde_json::from_value::<EstStockBasic>(x.to_owned())?;
            t.market = "sz".to_string();
            vec.push(t);
        }
    }

    Ok(vec)
}


#[tokio::test]
pub async fn test_request_stock_list(){
    let _ = request_sh_stock_list().await.unwrap();
    let _ = request_sz_stock_list().await.unwrap();
}