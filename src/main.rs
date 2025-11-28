// src/main.rs
// SHOULD be ttrs::
use ttrs::{ConnectionInfo, OrderLeg, OrderRequest, StreamData, make_core_api, fn_run_core};
use anyhow::Result;
use serde_json::Value;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> 
{
  let conn_info = ConnectionInfo 
  { base_url: "https://api.cert.tastyworks.com".to_string(),
    oauth: 
    ( format!("[YORUR client_id]")
    , format!("[YOUR client_secret]")
    , format!("[YOUR refresh_token]")
    ),
  };

  // Add CoreConfig setup
  let (error_tx, mut error_rx) = mpsc::unbounded_channel();
  let (shutdown_tx, shutdown_rx) = mpsc::channel::<String>(1);
  let config = ttrs::CoreConfig { error_tx, shutdown_rx };

  let (thand, tfoot) = make_core_api(10);
  let conn_handle = tokio::spawn(fn_run_core(tfoot, conn_info.clone(), config));
  let accts = thand.tell_tastytrade_to_get_accounts_info().await?;
  let acct = &accts.items[0].account;
  println!("Account: {}", acct.account_number);

    // Ticker example from user desc
    let (ticker_tx, mut ticker_rx) = mpsc::channel::<StreamData>(1024);
    let _ = thand.tell_tastytrade_to_get_ticker_stream("AAPL".to_string(), ticker_tx).await;
    if let Some(tick) = ticker_rx.recv().await {
        println!("ticker: {:#?}", tick);
    }

    // Dummy order for demo (adjust as needed)
    let order = OrderRequest {
        time_in_force: "Day".to_string(),
        order_type: "Market".to_string(),
        price: None,
        price_effect: None,
        legs: vec![OrderLeg {
            symbol: "AAPL".to_string(),
            action: "Buy to Open".to_string(),
            quantity: 0.001,
            instrument_type: "Equity".to_string(),
            extra:std::collections::HashMap::new()
        }],
        extra:std::collections::HashMap::new()
    };
    thand
        .tell_tastytrade_to_place_order(acct.account_number.clone(), order)
        .await?;

    // Account stream example
    let (stream_tx, mut stream_rx) = mpsc::channel::<Value>(1024);
    let _ = thand.tell_tasytrade_to_get_accounts_stream(acct.account_number.clone(), stream_tx).await;
    if let Some(update) = stream_rx.recv().await {
        println!("Account update: {}", update);
    }

    drop(thand); // Triggers shutdowns ? 

    let _ = shutdown_tx.send(format!("This is another way to trigger shutdown")).await?;

    let _ = conn_handle.await?;

    let reson = error_rx.recv().await;

    log::info!("Shudtown from ttrscore: [{:#?}]", reson);

    Ok(())
}