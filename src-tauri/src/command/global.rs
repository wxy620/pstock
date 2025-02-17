use std::time::Duration;
use crate::model::c2s::InitializePayload;
use crate::model::s2s::IPayload;
use crate::model::AppState;
use tauri::{AppHandle, Emitter, Manager};
use tauri::async_runtime::JoinHandle;
use tokio::sync::mpsc;
use crate::model::s2c::S2C_EMMIT_GLOBAL_EVENT;

// 全局注册一个，用于推送大盘最新报价， 交易信息， 系统通知等
pub async fn setup_global_monitor(app_handle: &AppHandle, payload: InitializePayload ){

    log::debug!("recv global event ! msg_id: {:?}", payload.msg_id);
    let state = app_handle.state::<AppState>();
    let buffer_size = state.sys_props.buffer_size;
    let (stock_tx, mut stock_rx) =
        mpsc::channel::<IPayload>(buffer_size);
    // acquire rwlock
    {
        let mut store = state.tx_store.write().await;
        store.insert("stock".to_string(), stock_tx);

    }
    //release rwlock
    /*
       TODO
        如果有多个事项，且不想相互干扰，启动多个线程，最后join
    */
    //事项1 获取大盘最新数据
    let app_handle_clone = app_handle.clone();
    let stock_job_handle = tauri::async_runtime::spawn(async move {
        let app_handle = app_handle_clone;
        while let Some(ref payload) = stock_rx.recv().await{
            match payload {
                //emit基本没有开销 500微秒
                IPayload::Dashboard{data, msg_id} => {
                    let _ = app_handle.emit(S2C_EMMIT_GLOBAL_EVENT, payload).is_err_and(|_e|{
                           log::warn!("failed emit! data:{:?} msg_id {:?}", data, msg_id);
                           false
                       }
                    );

                    //tokio::time::sleep(Duration::from_secs(3)).await;
               },
                _ =>()
            }
        }
        //多个 select!
    });

    let app_handle_clone = app_handle.clone();
    let mock_job_handle = fetch_dashboard_task(app_handle_clone).await;

    //单个
    //stock_job_handle.await.unwrap();
    //多个用 tokio::join! or try_join!
    let r = tokio::join!(stock_job_handle, mock_job_handle);
    log::debug!("r.0 = {:?}, r.1={:?}", r.0, r.1);
    //事项2 告警
}

async fn fetch_dashboard_task(app_handle: AppHandle) -> JoinHandle<()>{
    let mock_job_handle = tauri::async_runtime::spawn(async move {
        let  state = &app_handle.state::<AppState>();
        loop {
            {
                // let stock_hist = get_mock_data(&state).await;
                // if stock_hist.is_ok(){
                //     let stock_hist = stock_hist.unwrap();
                //     let dashboard = IPayload::Dashboard {
                //         data: stock_hist,
                //         msg_id: None,
                //     };
                //     //acquire rwlock
                //     {
                //         let mut tx_store =state.tx_store.write().await;
                //         let tx = tx_store.get_mut("stock").unwrap();
                //         let _ = tx.send(dashboard).await.is_err_and(|e|{
                //             log::warn!("failed to fetch stock data for {:?}", e);
                //             false
                //         });
                //         //等待缓冲队列的实现方式
                //         // if let Ok(permit) = tx.reserve().await {
                //         //     permit.send(dashboard);
                //         // }else{
                //         //     println!("failed to reserve stock");
                //         // }
                //     }
                //     //release rwlock
                // }
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }

    });
    mock_job_handle
}


// async fn get_mock_data(state: &State<'_,AppState>)->Result<StockHist, SysError>{
//     let pool = &state.db_pool;
//
//     let symbol = "600000";
//     let market = "SSE";
//     let raw_sql =
//         format!("select * from stock_hist_data where `symbol` = '{symbol}'  AND `market` = '{market}' limit 1");
//     let stock_hist: StockHist =sqlx::query_as(&raw_sql).fetch_one(pool).await?;
//     Ok(stock_hist)
// }