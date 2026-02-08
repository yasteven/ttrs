// src/bot.rs
//==================================================================
// ttrs/src/bot.rs - Core logic, channels, and runtime functions
//==================================================================
use crate::dat::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::interval;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use reqwest::{Client, header};
use serde_json::Value;
use rand::Rng;
use futures_util::{StreamExt, SinkExt};
//use std::fmt;
// =============================================================================
// External Types (for API usage)
// =============================================================================
#[derive(Debug, Clone)]
pub enum CoreError 
{   OAuth(String)
  , WebSocket(String)
  , TaskPanic(String)
  , Unrecoverable(String)
  , ParseFailure(String) 
  , ParseWarning(String)
  , ValidStartup(u64)
  , ReplyWarning(String)
  // ETC
}
pub struct CoreConfig 
{   pub error_tx: mpsc::UnboundedSender<CoreError>
  , pub shutdown_rx: tokio::sync::mpsc::Receiver<String>
}
pub fn make_core_api(chan_size: usize) -> (Thand, Tfoot) 
{ let ( get_accounts_info_tx, get_accounts_info_rx ) = mpsc::channel::<oneshot::Sender<Result<AccountsData>>>(chan_size);
  let ( get_trading_status_tx, get_trading_status_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<TradingStatus>>)>(chan_size);
  let ( get_positions_tx, get_positions_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<Vec<Position>>>)>(chan_size);
  let ( get_balances_tx, get_balances_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<Balance>>)>(chan_size);
  let ( get_transactions_tx, get_transactions_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<Vec<Transaction>>>)>(chan_size);
  let ( req_order_place_tx, place_order_rx ) = mpsc::channel::<(String, OrderRequest, oneshot::Sender<Result<OrderResponse>>)> (chan_size);
  let ( req_ticker_stream_tx, req_ticker_rx ) = mpsc::channel::<(String, mpsc::Sender<StreamData>, Option<oneshot::Sender<()>>)>(chan_size);
  let ( req_account_stream_tx, req_account_stream_rx ) = mpsc::channel::<(String, mpsc::Sender<Value>, oneshot::Sender<()>)>(chan_size);
  let ( get_live_orders_tx, get_live_orders_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<Vec<OrderResponse>>>)>(chan_size);
  let ( req_order_cancel_tx, cancel_order_rx ) = mpsc::channel::<(String, u64, oneshot::Sender<Result<OrderResponse>>)>(chan_size);
  let ( req_order_replace_tx, replace_order_rx ) = mpsc::channel::<(String, u64, OrderRequest, oneshot::Sender<Result<OrderResponse>>)>(chan_size);
  let ( req_order_dryrun_tx, dry_run_rx ) = mpsc::channel::<(String, OrderRequest, oneshot::Sender<Result<DryRunData>>)>(chan_size);
  let ( get_order_by_id_tx, get_order_by_id_rx ) = mpsc::channel::<(String, u64, oneshot::Sender<Result<OrderResponse>>)>(chan_size);
  let ( req_order_complex_tx, place_complex_order_rx ) = mpsc::channel::<(String, ComplexOrderRequest, oneshot::Sender<Result<ComplexOrderResponse>>)>(chan_size);
  let ( get_live_complex_orders_tx, get_live_complex_orders_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<Vec<ComplexOrderResponse>>>)>(chan_size);
  let ( get_option_chain_tx, get_option_chain_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<OptionChainData>>)>(chan_size);
  let ( get_margin_requirements_tx, get_margin_requirements_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<MarginRequirements>>)>(chan_size);
  let ( req_margin_order_dryrun_tx, margin_dry_run_rx ) = mpsc::channel::<(String, OrderRequest, oneshot::Sender<Result<DryRunData>>)>(chan_size);
  let ( get_bulk_quotes_tx, get_quotes_rx ) = mpsc::channel::<(Vec<String>, oneshot::Sender<Result<Vec<StreamQuote>>>)>(chan_size);
  let ( get_market_metrics_tx, get_market_metrics_rx ) = mpsc::channel::<(String, oneshot::Sender<Result<Vec<MarketMetric>>>)>(chan_size);
  let ( get_instrument_tx, get_instrument_rx ) = mpsc::channel:: < ( String , oneshot::Sender < Result < Instrument > >  )>(chan_size);
  let thand = Thand 
  {   get_accounts_info_tx
    , get_trading_status_tx
    , get_positions_tx
    , get_balances_tx
    , get_transactions_tx
    , get_order_by_id_tx
    , get_live_orders_tx
    , get_live_complex_orders_tx
    , get_option_chain_tx
    , get_margin_requirements_tx
    , get_bulk_quotes_tx
    , get_market_metrics_tx
    , get_instrument_tx

    , req_ticker_stream_tx
    , req_account_stream_tx
    , req_order_place_tx
    , req_order_cancel_tx
    , req_order_replace_tx
    , req_order_dryrun_tx
    , req_order_complex_tx
    , req_margin_order_dryrun_tx
  };
  let tfoot = Tfoot 
  {   get_accounts_info: get_accounts_info_rx
    , get_trading_status: get_trading_status_rx
    , get_positions: get_positions_rx
    , get_balances: get_balances_rx
    , get_transactions: get_transactions_rx
    , place_order: place_order_rx
    , req_ticker: req_ticker_rx
    , req_account_stream: req_account_stream_rx
    , get_live_orders: get_live_orders_rx
    , cancel_order: cancel_order_rx
    , replace_order: replace_order_rx
    , dry_run: dry_run_rx
    , get_order_by_id: get_order_by_id_rx
    , place_complex_order: place_complex_order_rx
    , get_live_complex_orders: get_live_complex_orders_rx
    , get_option_chain: get_option_chain_rx
    , get_margin_requirements: get_margin_requirements_rx
    , margin_dry_run: margin_dry_run_rx
    , get_quotes: get_quotes_rx
    , get_market_metrics: get_market_metrics_rx
    , get_instrument: get_instrument_rx,
  };
  (thand, tfoot)
}
// =============================================================================
// Thand: User-Facing Hand with Methods Returning Results or Pipes
// =============================================================================
#[derive(Debug, Clone)]
pub struct Thand 
{ pub get_accounts_info_tx: mpsc::Sender<oneshot::Sender<Result<AccountsData>>>,
  pub get_trading_status_tx: mpsc::Sender<(String, oneshot::Sender<Result<TradingStatus>>)>,
  pub get_positions_tx: mpsc::Sender<(String, oneshot::Sender<Result<Vec<Position>>>)>,
  pub get_balances_tx: mpsc::Sender<(String, oneshot::Sender<Result<Balance>>)>,
  pub get_transactions_tx: mpsc::Sender<(String, oneshot::Sender<Result<Vec<Transaction>>>)>,
  pub get_order_by_id_tx: mpsc::Sender<(String, u64, oneshot::Sender<Result<OrderResponse>>)>,
  pub get_live_orders_tx: mpsc::Sender<(String, oneshot::Sender<Result<Vec<OrderResponse>>>)>,
  pub get_live_complex_orders_tx: mpsc::Sender<(String, oneshot::Sender<Result<Vec<ComplexOrderResponse>>>)>,
  pub get_option_chain_tx: mpsc::Sender<(String, oneshot::Sender<Result<OptionChainData>>)>,
  pub get_margin_requirements_tx: mpsc::Sender<(String, oneshot::Sender<Result<MarginRequirements>>)>,
  pub get_bulk_quotes_tx: mpsc::Sender<(Vec<String>, oneshot::Sender<Result<Vec<StreamQuote>>>)>,
  pub get_market_metrics_tx: mpsc::Sender<(String, oneshot::Sender<Result<Vec<MarketMetric>>>)>,
  pub get_instrument_tx: mpsc::Sender<(String, oneshot::Sender<Result<Instrument>>)>,

  pub req_ticker_stream_tx: mpsc::Sender<(String, mpsc::Sender<StreamData>, Option<oneshot::Sender<()>>)>,
  pub req_account_stream_tx: mpsc::Sender<(String, mpsc::Sender<Value>, oneshot::Sender<()>)>,
  pub req_order_place_tx: mpsc::Sender<(String, OrderRequest, oneshot::Sender<Result<OrderResponse>>)>,
  pub req_order_replace_tx: mpsc::Sender<(String, u64, OrderRequest, oneshot::Sender<Result<OrderResponse>>)>,
  pub req_order_cancel_tx: mpsc::Sender<(String, u64, oneshot::Sender<Result<OrderResponse>>)>,
  pub req_order_dryrun_tx: mpsc::Sender<(String, OrderRequest, oneshot::Sender<Result<DryRunData>>)>,
  pub req_order_complex_tx: mpsc::Sender<(String, ComplexOrderRequest, oneshot::Sender<Result<ComplexOrderResponse>>)>,
  pub req_margin_order_dryrun_tx: mpsc::Sender<(String, OrderRequest, oneshot::Sender<Result<DryRunData>>)>,
}
impl Thand // NOTE: although these functions are provided for convienence, optimal design would not use them. Thus, they do not have any logging.
{ pub async fn tell_tastytrade_to_get_bulk_quotes(&self, symbols: Vec<String>) -> Result<Vec<StreamQuote>>
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_bulk_quotes_tx.send((symbols, resp_tx)).await?;
    Ok(resp_rx.await??)
  }
  pub async fn tell_tastytrade_to_get_market_metrics(&self, acct: String) -> Result<Vec<MarketMetric>> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_market_metrics_tx.send((acct, resp_tx)).await?;
    Ok(resp_rx.await??)
  }
  pub async fn tell_tastytrade_to_get_instrument(&self, symbol: String) -> Result<Instrument> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_instrument_tx.send((symbol, resp_tx)).await?;
    Ok(resp_rx.await??)
  }
  pub async fn tell_tastytrade_to_get_accounts_info(&self) -> Result<AccountsData> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_accounts_info_tx
      .send(resp_tx)
      .await
      .context("Send failed")?;
    let res = resp_rx.await.context("Recv failed")??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_trading_status(&self, acct: String) -> Result<TradingStatus> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_trading_status_tx
        .send((acct, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_positions(&self, acct: String) -> Result<Vec<Position>> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_positions_tx
        .send((acct, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_balances(&self, acct: String) -> Result<Balance> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_balances_tx
      .send((acct, resp_tx))
      .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_transactions(&self, acct: String) -> Result<Vec<Transaction>> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_transactions_tx
    .send((acct, resp_tx))
    .await?;
        let res = resp_rx.await??;
        Ok(res)
  }
  pub async fn tell_tastytrade_to_place_order(&self, acct: String, order: OrderRequest) -> Result<OrderResponse> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    if true // false
    { log::warn!("TTRS is in debug mode - calls a dry run order whenever we do a order request: {:#?} ", order);
      match self.tell_tastytrade_to_dry_run_order
      ( acct.clone() // String
      , order.clone() // : OrderRequest
      ).await // -> Result<DryRunData> 
      { Ok(ok) => 
        { log::info!
          ( "\n\n\n\n     ================{}{}    ==================\n\n\n\n"
          , " \n\n\n\n TTRS dry run result \n"
          , format!("{:#?}\n\n\n\n",ok) 
          );
        }
        Err(e) =>
        { log::error!("Gonna panic because ttrs failed a debug requirment to request dryrun: {:#?}", e);
        }
      }
    }
    self.req_order_place_tx
        .send((acct, order, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_live_orders(&self, acct: String) -> Result<Vec<OrderResponse>> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_live_orders_tx
        .send((acct, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_cancel_order(&self, acct: String, order_id: u64) -> Result<OrderResponse> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.req_order_cancel_tx
        .send((acct, order_id, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  // pub async fn tell_tastytrade_to_replace_order(&self, acct: String, order_id: u64, updated_order: OrderRequest) -> Result<OrderResponse> 
  // { let (resp_tx, resp_rx) = oneshot::channel();
  //   self.req_order_place_tx
  //       .send((acct, order_id, updated_order, resp_tx))
  //       .await?;
  //   let res = resp_rx.await??;
  //   Ok(res)
  // }
  pub async fn tell_tastytrade_to_dry_run_order(&self, acct: String, order: OrderRequest) -> Result<DryRunData> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.req_order_dryrun_tx
        .send((acct, order, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_order_by_id(&self, acct: String, order_id: u64) -> Result<OrderResponse> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_order_by_id_tx
        .send((acct, order_id, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_place_complex_order(&self, acct: String, order: ComplexOrderRequest) -> Result<ComplexOrderResponse> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.req_order_complex_tx
        .send((acct, order, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_live_complex_orders(&self, acct: String) -> Result<Vec<ComplexOrderResponse>> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_live_complex_orders_tx
        .send((acct, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_option_chain(&self, symbol: String) -> Result<OptionChainData> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_option_chain_tx
        .send((symbol, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_margin_requirements(&self, acct: String) -> Result<MarginRequirements> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.get_margin_requirements_tx
        .send((acct, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_margin_dry_run(&self, acct: String, order: OrderRequest) -> Result<DryRunData> 
  { let (resp_tx, resp_rx) = oneshot::channel();
    self.req_margin_order_dryrun_tx
        .send((acct, order, resp_tx))
        .await?;
    let res = resp_rx.await??;
    Ok(res)
  }
  pub async fn tell_tastytrade_to_get_ticker_stream(&self, sym: String, user_tx: mpsc::Sender<StreamData>) 
  { //let (resp_tx, _resp_rx) = oneshot::channel(); // Optional ack
    self.req_ticker_stream_tx
        .send((sym, user_tx, None))
        .await
        .unwrap_or(()); // Fire-and-forget for streaming
  }
  pub async fn tell_tasytrade_to_get_accounts_stream(&self, acct: String, user_tx: mpsc::Sender<Value>) 
  { let (resp_tx, _resp_rx) = oneshot::channel();
    self.req_account_stream_tx
        .send((acct, user_tx, resp_tx))
        .await
        .unwrap_or(());
  }
}
// =============================================================================
// Tfoot: Struct of Named Receivers (core + subs)
// =============================================================================
pub struct Tfoot 
{ pub get_accounts_info: mpsc::Receiver<oneshot::Sender<Result<AccountsData>>>,
  pub get_trading_status: mpsc::Receiver<(String, oneshot::Sender<Result<TradingStatus>>)>,
  pub get_positions: mpsc::Receiver<(String, oneshot::Sender<Result<Vec<Position>>>)>,
  pub get_balances: mpsc::Receiver<(String, oneshot::Sender<Result<Balance>>)>,
  pub get_transactions: mpsc::Receiver<(String, oneshot::Sender<Result<Vec<Transaction>>>)>,
  pub place_order: mpsc::Receiver<(String, OrderRequest, oneshot::Sender<Result<OrderResponse>>)>,
  pub req_ticker: mpsc::Receiver<(String, mpsc::Sender<StreamData>, Option<oneshot::Sender<()>>)>,
  pub req_account_stream: mpsc::Receiver<(String, mpsc::Sender<Value>, oneshot::Sender<()>)>,
  pub get_live_orders: mpsc::Receiver<(String, oneshot::Sender<Result<Vec<OrderResponse>>>)>,
  pub cancel_order: mpsc::Receiver<(String, u64, oneshot::Sender<Result<OrderResponse>>)>,
  pub replace_order: mpsc::Receiver<(String, u64, OrderRequest, oneshot::Sender<Result<OrderResponse>>)>,
  pub dry_run: mpsc::Receiver<(String, OrderRequest, oneshot::Sender<Result<DryRunData>>)>,
  pub get_order_by_id: mpsc::Receiver<(String, u64, oneshot::Sender<Result<OrderResponse>>)>,
  pub place_complex_order: mpsc::Receiver<(String, ComplexOrderRequest, oneshot::Sender<Result<ComplexOrderResponse>>)>,
  pub get_live_complex_orders: mpsc::Receiver<(String, oneshot::Sender<Result<Vec<ComplexOrderResponse>>>)>,
  pub get_option_chain: mpsc::Receiver<(String, oneshot::Sender<Result<OptionChainData>>)>,
  pub get_margin_requirements: mpsc::Receiver<(String, oneshot::Sender<Result<MarginRequirements>>)>,
  pub margin_dry_run: mpsc::Receiver<(String, OrderRequest, oneshot::Sender<Result<DryRunData>>)>,
  pub get_quotes: mpsc::Receiver<(Vec<String>, oneshot::Sender<Result<Vec<StreamQuote>>>)>,
  pub get_market_metrics: mpsc::Receiver<(String, oneshot::Sender<Result<Vec<MarketMetric>>>)>,
  pub get_instrument: mpsc::Receiver<(String, oneshot::Sender<Result<Instrument>>)>,
}
// =============================================================================
// Shared HTTP client and token management
// =============================================================================
pub(crate) async fn ensure_token
(   http: &Client
  , base_url: &str
  , oauth: &OauthInfo
  , shared_token: Arc<Mutex<Option<AccessToken>>>,
) -> Result<String>
{ log::debug!("ENTRY! ensure_token with base_url = {}\n oauth = PRIVATE", base_url, );
  log::trace!("Dropping pre-exising token...");
  let token_opt = shared_token.lock().await;
  if let Some(t) = &*token_opt
  { if t.expires_at > Instant::now()
    { // clone the token while we still have the borrow and return it
      let tok = t.token.clone();
      log::debug!("Token still valid, returning cached token");
      return Ok(tok);
    }
  }
  drop(token_opt);
  log::trace!("ensure_token Requesting next token...");
  let url = format!("{}/oauth/token", base_url);
  log::debug!("Token endpoint URL: {}", url);
  let params =
  [ ("grant_type", "refresh_token")
    , ("refresh_token", &oauth.2)
    , ("client_secret", &oauth.1)
    , ("client_id", &oauth.0)
  ];
  log::trace!("Request parameters:");
  log::trace!("  grant_type: refresh_token");
  log::trace!
  ( "  refresh_token: {}...{}\n  client_id: {}...{}\n  client_secret: {}...{}",
    if oauth.2.len() > 4 { &oauth.2[..4] } else { &oauth.2 },
    if oauth.2.len() > 4 { &oauth.2[oauth.2.len()-4..] } else { "" },
    if oauth.0.len() > 4 { &oauth.0[..4] } else { &oauth.0 },
    if oauth.0.len() > 4 { &oauth.0[oauth.0.len()-4..] } else { "" },
    if oauth.1.len() > 4 { &oauth.1[..4] } else { &oauth.1 },
    if oauth.1.len() > 4 { &oauth.1[oauth.1.len()-4..] } else { "" }
  );
  log::trace!("Request built, sending with .form() method...");
  let resp = http
    .post(&url)
    .header("User-Agent", "ttrs/1.0")
    .form(&params)
    .send()
    .await
    .context("Failed to send token request")?;
  log::trace!("Response status: {}\nResponse headers: {:#?}", resp.status(), resp.headers());
  let body_text = resp.text().await.context("Failed to read response body")?;
  log::trace!("Response body (raw): {:.20} ... []", body_text);
  let resp: Value = serde_json::from_str(&body_text)
    .context("Failed to parse response JSON")?;
  log::trace!("Response parsed: {:.100}", format!("{:#?}",resp));
  // Check for error
  if let Some(error_code) = resp.get("error_code") 
  { log::error!("OAuth error: error_code = {:?}", error_code);
    if let Some(error_desc) = resp.get("error_description") 
    { log::error!("OAuth error description: {:?}", error_desc);
    }
  }
  let access_token = resp["access_token"]
    .as_str()
    .context("no access_token in response - check error_code and error_description above")?
    .to_string();
  log::trace!("ensure_token Extracing expiry...");
  let expires_in = resp["expires_in"].as_u64().unwrap_or(900);
  log::debug!("Token expires in: {} seconds", expires_in);
  let new_token = AccessToken 
  { token: access_token.clone(),
    expires_at: Instant::now() + Duration::from_secs(expires_in.saturating_sub(60)),
  };
  let mut token_opt = shared_token.lock().await;
  *token_opt = Some(new_token);
  log::debug!("OK access token! Token: {}...{}",
    if access_token.len() > 4 { &access_token[..4] } else { &access_token },
    if access_token.len() > 4 { &access_token[access_token.len()-4..] } else { "" }
  );
  Ok(access_token)
}

pub(crate) async fn ensure_token_resilient
(   http: &Client
  , base_url: &str
  , oauth: &OauthInfo
  , shared_token: Arc<Mutex<Option<AccessToken>>>
  , error_tx: mpsc::UnboundedSender<CoreError>,
) -> Result<String> 
{ const MAX_RETRIES: u32 = 3;
  for attempt in 0..MAX_RETRIES 
  { match ensure_token(http, base_url, oauth, shared_token.clone()).await 
    { Ok(token) => return Ok(token),
      Err(e) if attempt < MAX_RETRIES - 1 => 
      { let backoff = Duration::from_secs(2_u64.pow(attempt));
        log::warn!("Token refresh failed (attempt {}), retrying in {:?}: {}", attempt + 1, backoff, e);
        tokio::time::sleep(backoff).await;
      }, 
      Err(e) => 
      { let msg = format!("OAuth failed after {} attempts: {}", MAX_RETRIES, e);
        let _ = error_tx.send(CoreError::OAuth(msg.clone()));
        return Err(anyhow::anyhow!(msg));
      }
    }
  }
  Err(anyhow::anyhow!("Token refresh exhausted retries"))
}

// ==============================================================================
// Futures Instrument streamer_symbol Parsing
// ==============================================================================
// From the log, /ESZ5 returns:
//   "streamer-symbol":"/ESZ25:XCME"
// But for SPY it returns:
//   "streamer-symbol":"SPY"
// ==============================================================================
fn url_encode(input: &str) -> String 
{ input
  .bytes()
  .map(|b| match b {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
      _ => format!("%{:02X}", b),
  })
        .collect()
}

// Helper to detect equity option streamer symbols (e.g., .SPY260107P681)
fn is_equity_option_streamer(sym: &str) -> bool {
    if !sym.starts_with('.') {
        return false;
    }
    let stripped = &sym[1..];
    // Regex: root (letters) + 6 digits + P/C + digits (strike)
    let re = regex::Regex::new(r"^[A-Z]+(\d{6}[PC])\d+$").unwrap();
    re.is_match(stripped)
}

fn is_futures_option_streamer(sym: &str) -> bool {
    sym.starts_with("./") && sym.ends_with(":XCME") && sym.contains("C") || sym.contains("P")
}

async fn fetch_streamer_symbol
(   http: &Client
  , shared_token: Arc<Mutex<Option<AccessToken>>>
  , conn: &ConnectionInfo
  , sym: &str
) -> Result<String>
{ log::debug!("ENTRY - fetch_streamer_symbol for symbol: {}", sym);
  let token = ensure_token
  ( http
    , &conn.base_url
    , &conn.oauth
    , shared_token
  ).await?;

    if is_equity_option_streamer(sym) {
      // where is my request to the equities options?
        log::debug!("Detected equity option streamer symbol '{}'; bypassing instrument lookup (valid dxfeed format)", sym);
        return Ok(sym.to_string());
    }


    if is_futures_option_streamer(sym) {
      // where is my request to the equities options?
        log::debug!("Detected Futures option streamer symbol '{}'; bypassing instrument lookup (valid dxfeed format)", sym);
        return Ok(sym.to_string());
    }

  // ---- fetch_streamer_symbol UNIVERSAL ROUTING (subject to change) ----
  let url = if sym.starts_with('/') 
  { // FUTURES
    let enc = url_encode(&sym); // "/ESZ5" -> "%2FESZ5"
    format!("{}/instruments/futures/{}", conn.base_url, enc)
  } 
  else if sym.contains('/') 
  { // CRYPTO
    let enc = url_encode(&sym); // "BTC/USD" -> "BTC%2FUSD"
    format!("{}/instruments/cryptocurrencies/{}", conn.base_url, enc)
  } 
  else 
  { // EQUITIES
    format!("{}/instruments/equities/{}", conn.base_url, sym)
  };

  log::debug!("Fetching streamer symbol from: {}", url);
  let resp = http
    .get(&url)
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .header("User-Agent", "ttrs/1.0")
    .send()
    .await
    .context("Failed to send request to instrument endpoint")?;
    
  log::debug!("Response status: {}", resp.status());
  if !resp.status().is_success() 
  { let status = resp.status();
    let body = resp.text().await.unwrap_or_else(|_| "(empty)".into());
    return Err(anyhow::anyhow!(
        "Failed to fetch instrument {}: {} - {}",
        sym, status, body
    ));
  }
  let body_text = resp.text().await?;
  log::trace!("Response body: {}", body_text);
  let resp: InstrumentResp = {
  let deserializer = &mut serde_json::Deserializer::from_str(&body_text);
  serde_path_to_error::deserialize(deserializer)
    .map_err(|e| {
      let path = e.path().to_string();
      log::error!(
        "Failed to deserialize instrument for {} at path '{}': {}",
        sym, path, e
      );
      if let Ok(pretty) = serde_json::to_string_pretty(&serde_json::from_str::<Value>(&body_text).unwrap()) {
        log::debug!("Full JSON:\n{}", pretty);
      }
      anyhow::anyhow!("Parse error at '{}': {}", path, e)
    })?
  };
  // Check if streamer_symbol exists, otherwise use symbol
  let streamer_sym = resp.data.streamer_symbol
      .or_else(|| Some(resp.data.symbol.clone()))
      .ok_or_else(|| anyhow::anyhow!("No symbol found for {}", sym))?;
  
  log::debug!("Resolved {} -> {}", sym, streamer_sym);

  Ok(streamer_sym)
}
// ==============================================================================
// Core State Machines & helper functions:
// ==============================================================================
// TickerLoopCmd (for ticker_loop)
#[derive(Debug)]
pub(crate) enum TickerLoopCmd
{ AddRoute
  ( String // usr_sym
  , String // stream_sym
  , mpsc::Sender<StreamData> // user_tx
  )
  ,
  Shutdown
  ,
}

fn spawn_api_task_safe<T, Fut>
( fut: Fut
, resp_tx: oneshot::Sender<Result<T>>
, error_tx: mpsc::UnboundedSender<CoreError>
, description: &str
) where T: Send + 'static
, Fut: Future<Output = Result<T>> + Send + 'static,
{ let desc = description.to_string();
  tokio::spawn
  ( async move 
    { match fut.await 
      { Ok(result) => 
        { let _ = resp_tx.send(Ok(result));
        }
        Err(e) => 
        { let msg = format!("Task '{}' failed: {}", desc, e);
          let _ = error_tx.send(CoreError::ReplyWarning(msg.clone()));
          let _ = resp_tx.send(Err(anyhow::anyhow!(msg)));
        }
      }
    }
  );
}

fn spawn_api_task_deadly<T, Fut>
( fut: Fut
, resp_tx: oneshot::Sender<Result<T>>
, error_tx: mpsc::UnboundedSender<CoreError>
, description: &str
) where T: Send + 'static
, Fut: Future<Output = Result<T>> + Send + 'static,
{ let desc = description.to_string();
  tokio::spawn
  ( async move 
    { match fut.await 
      { Ok(result) => 
        { let _ = resp_tx.send(Ok(result));
        }
        Err(e) => 
        { let msg = format!("Task '{}' failed: {}", desc, e);
          let _ = error_tx.send(CoreError::Unrecoverable(msg.clone()));
          let _ = resp_tx.send(Err(anyhow::anyhow!(msg)));
        }
      }
    }
  );
}


macro_rules! api_req {
  (GET, $url:expr, $resp_type:ty, $http:expr, $token:expr, $base_url:expr, $oauth:expr, $error_tx:expr) => {
    {
      let token = ensure_token_resilient
        ( &$http
        , &$base_url
        , &$oauth
        , $token
        , $error_tx
        ).await?;
      let resp = $http
        .get(&$url)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header("User-Agent", "ttrs/1.0")
        .send()
        .await?;
      
      let status = resp.status();
      if !status.is_success() {
        let text = resp.text().await.unwrap_or_else(|_| "(empty)".to_string());
        log::error!("GET {} failed with {}: {}", $url, status, text);
        return Err(anyhow::anyhow!("Request failed: {} - {}", status, text));
      }
      
      let text = resp.text().await?;
      serde_json::from_str::<$resp_type>(&text).map_err(|e| {
        log::error!("Failed to parse JSON from successful GET {}: {}\nBody was: {}", $url, e, text);
        anyhow::anyhow!("JSON parse error: {}", e)
      })?
    }
  };
  
  (POST, $url:expr, $body:expr, $resp_type:ty, $http:expr, $token:expr, $base_url:expr, $oauth:expr, $error_tx:expr) => {
    {
      let token = ensure_token_resilient
        ( &$http
        , &$base_url
        , &$oauth
        , $token
        , $error_tx
        ).await?;
      let body = serde_json::to_string(&$body)?;
      let resp = $http
        .post(&$url)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .header("User-Agent", "ttrs/1.0")
        .body(body)
        .send()
        .await?;
      
      let status = resp.status();
      if !status.is_success() {
        let text = resp.text().await.unwrap_or_else(|_| "(empty)".to_string());
        log::error!("POST {} failed with {}: {}", $url, status, text);
        return Err(anyhow::anyhow!("Request failed: {} - {}", status, text));
      }
      
      let text = resp.text().await?;
      serde_json::from_str::<$resp_type>(&text).map_err(|e| {
        log::error!("Failed to parse JSON from successful POST {}: {}\nBody was: {}", $url, e, text);
        anyhow::anyhow!("JSON parse error: {}", e)
      })?
    }
  };
  
  (DELETE, $url:expr, $resp_type:ty, $http:expr, $token:expr, $base_url:expr, $oauth:expr, $error_tx:expr) => {
    {
      let token = ensure_token_resilient
        ( &$http
        , &$base_url
        , &$oauth
        , $token
        , $error_tx
        ).await?;
      let resp = $http
        .request(reqwest::Method::DELETE, &$url)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header("User-Agent", "ttrs/1.0")
        .send()
        .await?;
      
      let status = resp.status();
      if !status.is_success() {
        let text = resp.text().await.unwrap_or_else(|_| "(empty)".to_string());
        log::error!("DELETE {} failed with {}: {}", $url, status, text);
        return Err(anyhow::anyhow!("Request failed: {} - {}", status, text));
      }
      
      let text = resp.text().await?;
      serde_json::from_str::<$resp_type>(&text).map_err(|e| {
        log::error!("Failed to parse JSON from successful DELETE {}: {}\nBody was: {}", $url, e, text);
        anyhow::anyhow!("JSON parse error: {}", e)
      })?
    }
  };
  
  (PUT, $url:expr, $body:expr, $resp_type:ty, $http:expr, $token:expr, $base_url:expr, $oauth:expr, $error_tx:expr) => {
    {
      let token = ensure_token_resilient
        ( &$http
        , &$base_url
        , &$oauth
        , $token
        , $error_tx
        ).await?;
      let body = serde_json::to_string(&$body)?;
      let resp = $http
        .request(reqwest::Method::PUT, &$url)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .header("User-Agent", "ttrs/1.0")
        .body(body)
        .send()
        .await?;
      
      let status = resp.status();
      if !status.is_success() {
        let text = resp.text().await.unwrap_or_else(|_| "(empty)".to_string());
        log::error!("PUT {} failed with {}: {}", $url, status, text);
        return Err(anyhow::anyhow!("Request failed: {} - {}", status, text));
      }
      
      let text = resp.text().await?;
      serde_json::from_str::<$resp_type>(&text).map_err(|e| {
        log::error!("Failed to parse JSON from successful PUT {}: {}\nBody was: {}", $url, e, text);
        anyhow::anyhow!("JSON parse error: {}", e)
      })?
    }
  };
}

pub async fn fn_run_core
( mut tfoot: Tfoot
, conn_info: ConnectionInfo
, mut config: CoreConfig
) -> Result<()> 
{ // SPAWING TTRS CORE THREAD #0  
  let http0 = Client::new();
  let shared_token0 = Arc::new(Mutex::new(None::<AccessToken>));
  let shared_quote = Arc::new(Mutex::new(None::<QuoteToken>));
  // Start the streamer
  let (itx, irx) = mpsc::channel::<TickerLoopCmd>(1024);
  let conn_clone = conn_info.clone();
  let shared_token_clone = shared_token0.clone();
  let shared_quote_clone = shared_quote.clone();
  // SPAWING TTRS THREAD #1  
  let _ticker_handle = tokio::spawn
  ( ticker_loop_resilient
    (   irx
      , conn_clone
      , shared_token_clone
      , shared_quote_clone
      , config.error_tx.clone(),
    )
  );
  let tx_to_ticker_loop = itx;
  let mut account_handles: Vec<JoinHandle<()>> = Vec::new();
  let _ = config.error_tx.send(CoreError::ValidStartup(0));
  // ENTERING MAIN LOOP:  
  loop 
  { let http = http0.clone();
    let shared_token = shared_token0.clone();
    let conn = conn_info.clone();
    let error_tx_clone1 = config.error_tx.clone();
    let error_tx_clone2 = config.error_tx.clone();
        
    tokio::select! 
    { 
      res = config.shutdown_rx.recv() =>
      { log::info!("Shutdown signal =[{:#?}], cleaning up...", res);
        if let Err(e) = tx_to_ticker_loop.send(TickerLoopCmd::Shutdown).await 
        { log::warn!("Failed to send shutdown to ticker: {}", e);
        } 
        for handle in account_handles 
        { handle.abort();
        }
        break;
      }

      // ===== SIMPLE GET HANDLERS =====
      Some((acct, resp_tx)) = tfoot.get_trading_status.recv() => 
      { let url = format!("{}/accounts/{}/trading-status", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, TradingStatusResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() 
            { log::warn!
              ( "TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! \n  {}\n  {:#?}"
              , resp.context
              , resp.extra
              );
            }
            Ok(resp.data)
          }
          , resp_tx, error_tx_clone2, "get_trading_status"
        );
      }

      Some((acct, resp_tx)) = tfoot.get_balances.recv() => 
      { let url = format!("{}/accounts/{}/balances", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, BalancesResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() 
            { log::warn!
              ( "TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! \n  {}\n  {:#?}"
              , resp.context
              , resp.extra
              );
            }
            Ok(resp.data)
          }
          , resp_tx, error_tx_clone2, "get_balances"
        );
      }

      Some((acct, resp_tx)) = tfoot.get_transactions.recv() => 
      { let url = format!("{}/accounts/{}/transactions", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, TransactionsResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() 
            { log::warn!
              ( "TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! \n  {}\n  {:#?}\n  {:#?}"
              , resp.context
              , resp.extra
              , resp.pagination
              );
            }
            Ok(resp.data.items)
          }
          , resp_tx, error_tx_clone2, "get_transactions"
        );
      }

      Some((acct, resp_tx)) = tfoot.get_live_orders.recv() => 
      { let url = format!("{}/accounts/{}/orders/live", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, LiveOrdersResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?} {:#?}", resp, resp.data.context);}
            Ok(resp.data.items)
          }
          , resp_tx, error_tx_clone2, "get_live_orders"
        );
      }

      Some((acct, order_id, resp_tx)) = tfoot.get_order_by_id.recv() => 
      { let url = format!("{}/accounts/{}/orders/{}", conn.base_url, acct, order_id);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, SingleOrderResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.data.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data)
          }
          , resp_tx, error_tx_clone2, "get_order_by_id"
        );
      }

      Some((acct, resp_tx)) = tfoot.get_live_complex_orders.recv() => 
      { let url = format!("{}/accounts/{}/complex-orders/live", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, LiveOrdersResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            //if !resp.data.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data.items)
          }
          , resp_tx, error_tx_clone2, "get_live_complex_orders"
        );
      }

      Some((acct, resp_tx)) = tfoot.get_market_metrics.recv() => 
      { let url = format!("{}/accounts/{}/market-metrics", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, MarketMetricsResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data.items)
          }
          , resp_tx, error_tx_clone2, "get_market_metrics"
        );
      }

      Some((acct, resp_tx)) = tfoot.get_margin_requirements.recv() => 
      { let url = format!("{}/accounts/{}/margin-requirements", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, MarginRequirementsResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data)
          }
          , resp_tx, error_tx_clone2, "get_margin_requirements"
        );
      }

      Some((symbols, resp_tx)) = tfoot.get_quotes.recv() => 
      { let syms = symbols.join(",");
        let url = format!("{}/quotes?symbols={}", conn.base_url, syms);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, QuotesResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data.items)
          }
          , resp_tx, error_tx_clone2, "get_quotes"
        );
      }

      Some(resp_tx) = tfoot.get_accounts_info.recv() => 
      { let url = format!("{}/customers/me/accounts", conn.base_url);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, CustomerAccountsResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() 
            { log::warn!
              ( "TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! \n  {}\n  {:#?}"
              , resp.context
              , resp.extra
              );
            }
            Ok(resp.data)
          }
          , resp_tx, error_tx_clone2, "get_accountss"
        );
      }

      Some((acct, resp_tx)) = tfoot.get_positions.recv() => 
      { let url = format!("{}/accounts/{}/positions", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(GET, url, PositionsResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() 
            { log::warn!
              ( "TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! \n  {}\n  {:#?}"
              , resp.context
              , resp.extra
              );
            }
            Ok(resp.data.items)
          }
          , resp_tx, error_tx_clone2, "get_positions"
        );
      }

      // ===== SIMPLE POST HANDLERS =====
      Some((acct, order, resp_tx)) = tfoot.place_order.recv() => 
      { let url = format!("{}/accounts/{}/orders", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(POST, url, order, OrderSubmitResp, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.data.order.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data.order)
          }
          , resp_tx, error_tx_clone2, "place_order"
        );
      }

      Some((acct, order, resp_tx)) = tfoot.dry_run.recv() => 
      { let url = format!("{}/accounts/{}/orders/dry-run", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(POST, url, order, DryRunResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data)
          }
          , resp_tx, error_tx_clone2, "dry_run"
        );
      }

      Some((acct, order, resp_tx)) = tfoot.place_complex_order.recv() => 
      { let url = format!("{}/accounts/{}/complex-orders", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(POST, url, order, OrderSubmitResp, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.data.order.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data.order)
          }
          , resp_tx, error_tx_clone2, "place_complex_order"
        );
      }

      Some((acct, order, resp_tx)) = tfoot.margin_dry_run.recv() => 
      { let url = format!("{}/margin/accounts/{}/dry-run", conn.base_url, acct);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(POST, url, order, DryRunResponse, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data)
          }
          , resp_tx, error_tx_clone2, "margin_dry_run"
        );
      }

      // ===== SIMPLE DELETE HANDLERS =====
      Some((acct, order_id, resp_tx)) = tfoot.cancel_order.recv() => 
      { let url = format!("{}/accounts/{}/orders/{}", conn.base_url, acct, order_id);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(DELETE, url, OrderSubmitResp, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.data.order.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data.order)
          }
          , resp_tx, error_tx_clone2, "cancel_order"
        );
      }

      // ===== SIMPLE PUT HANDLERS =====
      Some((acct, order_id, updated_order, resp_tx)) = tfoot.replace_order.recv() => 
      { let url = format!("{}/accounts/{}/orders/{}", conn.base_url, acct, order_id);
        spawn_api_task_safe
        ( async move 
          { let resp = api_req!(PUT, url, updated_order, OrderSubmitResp, http, shared_token, conn.base_url, conn.oauth, error_tx_clone1);
            if !resp.data.order.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data.order)
          }
          , resp_tx, error_tx_clone2, "replace_order"
        );
      }

      // ===== COMPLEX HANDLERS (CUSTOM LOGIC) =====
      Some((underlying, resp_tx)) = tfoot.get_option_chain.recv() =>
      { let http = http.clone();
        let shared_token = shared_token.clone();
        let conn = conn_info.clone();
        let error_tx = config.error_tx.clone();
        spawn_api_task_safe
        ( async move
          { use chrono::{Datelike, Duration, Utc, Weekday,DateTime,TimeZone};

            let dbgspt = "[TTRS_BOT_GET_OPTION_CHAIN]";

            // Current date as specified
            let today = Utc::now(); 
            //let today = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();  // or .with_timezone(&Utc) if needed
            // let today = Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap(); // at least this compiled

            // Helper to parse "YYYY-MM-DD" into chrono::Date<Utc>
            let parse_date = |s: &str| -> Result<DateTime<Utc>> {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|nd| nd.and_hms_opt(0, 0, 0).unwrap())
                    .map(|naive| Utc.from_utc_datetime(&naive))
                    .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))
            };

            if underlying.starts_with('/') {
              log::trace!("{} Futures option chain request for {}", dbgspt, underlying);

              // Extract product root code (e.g. "ES" from "/ESH6")
              let mut root = underlying.trim_start_matches('/').to_string();

              // Remove trailing year digits
              while root.chars().last().map_or(false, |c| c.is_ascii_digit()) {
                root.pop();
              }
              // Remove trailing month letter
              if root.chars().last().map_or(false, |c| c.is_alphabetic()) {
                root.pop();
              }

              log::debug!("{} Using detailed flat endpoint for root: {}", dbgspt, root);

              // Use the detailed (non-nested) endpoint
              let base_url = format!("{}/futures-option-chains/{}", conn.base_url, root);

              // Build list of desired expiration dates
              let mut desired_expirations = std::collections::HashSet::new();

              let mut date = today + Duration::days(1);

              // Next 2 months: daily business days (Mon-Fri)
              let two_months = today + Duration::days(60);
              while date < two_months {
                if matches!(date.weekday(), Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri) {
                  desired_expirations.insert(date);
                }
                date += Duration::days(1);
              }

              // Next 2 months after that: Mon, Wed, Fri
              let four_months = today + Duration::days(120);
              while date < four_months {
                if matches!(date.weekday(), Weekday::Mon | Weekday::Wed | Weekday::Fri) {
                  desired_expirations.insert(date);
                }
                date += Duration::days(1);
              }

              // Next 2 months after that: Fridays only
              let six_months = today + Duration::days(180);
              while date < six_months {
                if date.weekday() == Weekday::Fri {
                  desired_expirations.insert(date);
                }
                date += Duration::days(1);
              }

              // Monthlies (3rd Friday) for next 6 months beyond 6 months
              let mut monthly_cursor = today + Duration::days(180);
              let twelve_months_from_now = today + Duration::days(365 + 180);
              while monthly_cursor < twelve_months_from_now {
                let year = monthly_cursor.year();
                let month = monthly_cursor.month();

                let mut first_day = Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap();

                let today = Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap(); // unwrap is safe for valid dates

                // Find first Friday
                while first_day.weekday() != Weekday::Fri {
                  first_day += Duration::days(1);
                }
                let third_friday = first_day + Duration::days(14); // 1st + 14 days = 3rd Friday

                if third_friday >= today {
                  desired_expirations.insert(third_friday);
                }

                // Next month
                monthly_cursor = if month == 12 {
                  Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
                } 
                  else {
                  Utc.with_ymd_and_hms(year, month+1, 1, 0, 0, 0).unwrap()
                };
              }

              log::debug!("{} Generated {} desired expiration dates", dbgspt, desired_expirations.len());

              // Pagination handling
              let mut all_items = Vec::new();
              let mut offset = 0;
              let per_page = 500; // Max allowed; adjust if needed

              loop {
                let url = format!("{}?per-page={}&page-offset={}", base_url, per_page, offset);

                log::debug!("{} Fetching page offset={} from {}", dbgspt, offset, url);

                let token = ensure_token_resilient(&http, &conn.base_url, &conn.oauth, shared_token.clone(), error_tx.clone()).await?;

                let resp = http.get(&url)
                  .header(header::AUTHORIZATION, format!("Bearer {}", token))
                  .header("User-Agent", "ttrs/1.0")
                  .send()
                  .await
                  .context("Failed to fetch futures option chain page")?;

                if !resp.status().is_success() {
                  return Err(anyhow::anyhow!("Futures option chain request failed: {}", resp.status()));
                }



                let body_text = resp.text().await.context("Failed to read response body")?;
                let json: serde_json::Value = serde_json::from_str(&body_text)
                  .context("Failed to parse JSON")?;

                let items = match json["data"]["items"].as_array() {
                  Some(a) => a,
                  None => return Err(anyhow::anyhow!("No items in response")),
                };

                if items.is_empty() {
                  break; // No more pages
                }

                let mut page_filtered = Vec::new();

                for item in items {
                  if let Some(exp_str) = item["expiration-date"].as_str() {
                    if let Ok(exp_date) = parse_date(exp_str) {
                      if desired_expirations.contains(&exp_date) {
                        // You may want to clone or map to your internal Option type here
                        page_filtered.push(item.clone());
                      }
                    }
                  }
                }
                let pfl = page_filtered.len();
                all_items.extend(page_filtered);
                log::debug!("{} Added {} filtered items from this page (total so far: {})", dbgspt, pfl, all_items.len());

                // Check pagination
                let total = json["pagination"]["total-items"].as_u64().unwrap_or(0);
                if (offset + per_page) as u64 >= total {
                  break;
                }

                offset += per_page;
              }

              log::info!("{} Successfully fetched and filtered {} futures options for {}", dbgspt, all_items.len(), underlying);

              // // Construct final OptionChainData (adjust field names to match your struct)

              use chrono::{DateTime, Utc};
              use std::collections::{HashMap, BTreeMap};

              // After collecting all_items: Vec<Value>

              log::info!("{} Successfully fetched and filtered {} futures options for {}", dbgspt, all_items.len(), underlying);

              // Group by expiration date first
              let mut expirations_map: HashMap<String, (DateTime<Utc>, BTreeMap<String, (Value, Value)>)> = HashMap::new();

              for item in all_items {
                  let exp_date_str = match item["expiration-date"].as_str() {
                      Some(s) => s,
                      None => continue,
                  };
                  let strike = match item["strike-price"].as_str() {
                      Some(s) => s.to_string(),
                      None => continue,
                  };

                  let is_call = match item["option-type"].as_str() {
                      Some("C") => true,
                      Some("P") => false,
                      _ => { log::warn!("TTRS - UNKNOWN OPTION TYPE for item {:#?}", item); continue; },
                  };

                  use serde_json::Value;
                  let exp_entry = expirations_map
                      .entry(exp_date_str.to_string())
                      .or_insert_with(|| {
                          let dt = parse_date(exp_date_str).expect("valid expiration date");
                          (dt, BTreeMap::new())
                      });

                  let strike_entry = exp_entry.1 // the BTreeMap<String, (Value, Value)>
                      .entry(strike)
                      .or_insert((Value::Null, Value::Null));

                  if is_call {
                      strike_entry.0 = item.clone(); // replace call
                  } else {
                      strike_entry.1 = item.clone(); // replace put
                  }
              }

              // Now build the nested structure
              let mut expirations: Vec<OptionExpiration> = Vec::new();

              for (exp_date_str, (exp_dt, strikes_map)) in expirations_map {
                  let days_to_exp = (exp_dt - today).num_days() as u32;

                  let mut strikes: Vec<Strike> = Vec::new();

                  for (strike_price, (call_val, put_val)) in strikes_map {
                      let call_symbol = call_val["symbol"].as_str().unwrap_or("");
                      let call_streamer = call_val["streamer-symbol"].as_str().unwrap_or("");
                      let put_symbol = put_val["symbol"].as_str().unwrap_or("");
                      let put_streamer = put_val["streamer-symbol"].as_str().unwrap_or("");

                      // Skip if missing one side (though you filtered to paired expirations)
                      if call_symbol.is_empty() || put_symbol.is_empty() {
                          continue;
                      }

                      strikes.push(Strike {
                          strike_price: strike_price,
                          call: call_symbol.to_string(),
                          call_streamer_symbol: call_streamer.to_string(),
                          put: put_symbol.to_string(),
                          put_streamer_symbol: put_streamer.to_string(),
                          extra: HashMap::new(),
                      });
                  }

                  // Determine expiration type (simplified)
                  let expiration_type = if exp_dt.weekday() == Weekday::Fri {
                      "Weekly".to_string()
                  } else {
                      "Monthly".to_string()
                  };

                  // Settlement type — most ES are physical delivery via future
                  let settlement_type = "Future".to_string();

                  expirations.push(OptionExpiration {
                      days_to_expiration: days_to_exp,
                      expiration_date: exp_date_str,
                      expiration_type,
                      settlement_type,
                      strikes,
                      extra: HashMap::new(),
                  });
              }

              // Sort expirations by date
              expirations.sort_by_key(|e| e.expiration_date.clone());

              // Build the root
              let root = OptionChainRoot {
                  root_symbol: "/ES".to_string(),
                  underlying_symbol: underlying.to_string(),
                  shares_per_contract: 50, // E-mini S&P 500 multiplier
                  option_chain_type: "standard".to_string(),
                  tick_sizes: vec![], // You can populate from future-product if needed
                  deliverables: vec![],
                  expirations,
                  extra: HashMap::new(),
              };

              // Final response
              let chain_data = OptionChainData {
                  items: vec![root],
                  extra: HashMap::new(),
              };

              Ok(chain_data)
            } 
            else
            {
              // === EQUITY PATH (unchanged from original) ===
              let chain_url = format!("{}/option-chains/{}/nested", conn.base_url, underlying);
              log::debug!("{} Equity option chain URL: {}", dbgspt, chain_url);

              let token = ensure_token_resilient(&http, &conn.base_url, &conn.oauth, shared_token, error_tx.clone()).await?;

              let reqwest_resp = http
                .get(&chain_url)
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header("User-Agent", "ttrs/1.0")
                .send()
                .await
                .context("Failed to send equity option chain request")?;

              if !reqwest_resp.status().is_success() {
                return Err(anyhow::anyhow!("Equity option chain request failed: {}", reqwest_resp.status()));
              }

              let body_text = reqwest_resp.text().await.context("Failed to read raw body")?;
              log::trace!("{} Equity chain raw response: {:.200} ... [{}]", dbgspt, body_text, body_text.len());

              let mut deserializer = serde_json::Deserializer::from_str(&body_text);
              let resp: OptionChainResponse = serde_path_to_error::deserialize(&mut deserializer)
                .map_err(|e| {
                  log::error!(
                    "{} Equity chain deserialization failed at path '{}': {}\nRaw body:\n{}",
                    dbgspt, e.path(), e.inner(), body_text
                  );
                  anyhow::anyhow!("Deserialization failed at '{}': {}", e.path(), e.inner())
                })?;

              if !resp.extra.is_empty() {
                log::warn!(
                  "{} Detected extra unprocessed data in equity chain response",
                  dbgspt
                );
              }

              log::debug!(
                "{} Equity option chain fetched successfully: {} items",
                dbgspt, resp.data.items.len()
              );

              Ok(resp.data)
            }
          }
          , resp_tx, error_tx_clone2, "get_option_chain"
        );
      }


      Some((symbol, resp_tx)) = tfoot.get_instrument.recv() => 
      { let http = http.clone();
        let shared_token = shared_token.clone();
        let conn = conn_info.clone();
        let error_tx = config.error_tx.clone();
        spawn_api_task_safe
        ( async move 
          { let url = if symbol.starts_with('/') 
            { let enc = url_encode(&symbol);
              format!("{}/instruments/futures/{}", conn.base_url, enc)
            } 
            else if symbol.contains('/') 
            { let enc = url_encode(&symbol);
              format!("{}/instruments/cryptocurrencies/{}", conn.base_url, enc)
            } 
            else 
            { format!("{}/instruments/equities/{}", conn.base_url, symbol)
            };
            log::debug!("Instrument lookup URL: {}", url);
            let resp = api_req!(GET, url, InstrumentResp, http, shared_token, conn.base_url, conn.oauth, error_tx);
            if !resp.extra.is_empty() { log::warn!("TTRS - Tastytrade gave us EXTRA DATA IN RESPONSE! {:#?}", resp);}
            Ok(resp.data)
          }
          , resp_tx, error_tx_clone2, "get_instrument"
        );
      }

      // ===== SPECIAL PATTERN HANDLERS =====
      Some((sym, user_tx, resp_tx)) = tfoot.req_ticker.recv() => 
      { let http = http.clone();
        let shared_token = shared_token.clone();
        let conn = conn_info.clone();
        let tx_to_ticker_loop = tx_to_ticker_loop.clone();
        let error_tx = config.error_tx.clone();
        tokio::spawn
        ( async move 
          { match fetch_streamer_symbol
            ( &http, shared_token.clone(), &conn, &sym ).await 
            { Ok(streamer_sym) =>
              { let _ = tx_to_ticker_loop.send
                ( TickerLoopCmd::AddRoute(sym, streamer_sym, user_tx)
                ).await;
              }
              Err(e) => 
              { let msg = format!("Failed to fetch streamer symbol for {}: {}", sym, e);
                log::error!("{}", msg);
                let _ = error_tx.send(CoreError::WebSocket(msg));
              }
            }
          }
        );
        if let Some(resp_tx) = resp_tx
        { let _ = resp_tx.send(());
        }
      }

      Some((acct, user_tx, resp_tx)) = tfoot.req_account_stream.recv() => 
      { let shared_token = shared_token.clone();
        let conn = conn_info.clone();
        let error_tx = config.error_tx.clone();
        let handle = tokio::spawn
        ( account_stream_loop
          (   acct
            , user_tx
            , conn
            , shared_token
            , error_tx
          )
        );
        account_handles.push(handle);
        let _ = resp_tx.send(());
      }
    }
  }
  Ok(())
}

pub(crate) async fn ticker_loop_resilient
( mut rx: mpsc::Receiver<TickerLoopCmd>
  , conn: ConnectionInfo
  , shared_token: Arc<Mutex<Option<AccessToken>>>
  , shared_quote: Arc<Mutex<Option<QuoteToken>>>
  , error_tx: mpsc::UnboundedSender<CoreError>
) 
{ let mut tlc = 0;
  loop 
  { tlc = tlc + 1;
    match ticker_loop
    ( &mut rx
    , &conn
    , shared_token.clone()
    , shared_quote.clone()
    , error_tx.clone()
    , tlc
    ).await
    { Ok(_) => break,
      Err(e) => 
      {
        let msg = format!("Ticker loop failed: {}. Reconnecting in 5s...", e);
        log::warn!("{}", msg);
        let _ = error_tx.send(CoreError::WebSocket(msg));
        tokio::time::sleep(Duration::from_secs(5)).await;
      }
    }
  }
}

async fn ticker_loop
( rx: &mut mpsc::Receiver<TickerLoopCmd>
, conn: &ConnectionInfo
, shared_token: Arc<Mutex<Option<AccessToken>>>
, shared_quote: Arc<Mutex<Option<QuoteToken>>>
, error_tx: mpsc::UnboundedSender<CoreError>
, lcounter: u64
) -> Result<()>
{ log::debug!
  ( "TTRS TICKER_LOOP (TTRS_TL) STARTED | base_url={}"
  , conn.base_url
  );
  let http = Client::new();
  log::trace!("// ------------------------------------------------------------");
  log::debug!("TTRS_TL 1. Ensure valid QuoteToken...");
  log::trace!("// ------------------------------------------------------------");
  { let mut quote_guard = shared_quote.lock().await;
    if quote_guard.is_none()
    { log::trace!("TTRS_TL: fetching fresh QuoteToken");
      let access = match ensure_token_resilient
      ( &http
      , &conn.base_url
      , &conn.oauth
      , shared_token.clone()
      , error_tx.clone()
      ).await
      { Ok(tok) => tok,
        Err(e) =>
        { let msg = format!("ticker_loop: access token failed: {e}");
          log::debug!("TTRS_TL EXIT FAIL - {msg}");
          let _ = error_tx.send(CoreError::OAuth(msg.clone()));
          return Err(anyhow::anyhow!(msg));
        }
      };
      let url = format!("{}/api-quote-tokens", conn.base_url);
      let resp: QuoteTokenResp = match http.get(&url)
        .header(header::AUTHORIZATION, format!("Bearer {access}"))
        .header("User-Agent", "ttrs/0.1")
        .send()
        .await
      { Ok(r) => r,
        Err(e) =>
        { let msg = format!("ticker_loop: quote token request failed: {e}");
          log::debug!("TTRS_TL EXIT FAIL - {msg}");
          let _ = error_tx.send(CoreError::WebSocket(msg.clone()));
          return Err(anyhow::anyhow!(msg));
        }
      }
      .json()
      .await
      .map_err(|e|
      { let msg = format!("ticker_loop: failed to parse quote token JSON: {e}");
        log::debug!("TTRS_TL EXIT FAIL - {msg}");
        let _ = error_tx.send(CoreError::WebSocket(msg.clone()));
        anyhow::anyhow!(msg)
      })?;
      let qt = QuoteToken
      { token: resp.data.token.clone(),
        url: resp.data.dxlink_url.clone(),
        expires_at: Instant::now() + Duration::from_secs(23 * 3600),
      };
      *quote_guard = Some(qt);
      log::trace!("TTRS_TL: fresh QuoteToken acquired");
    }
  }
  let qt =
  { let guard = shared_quote.lock().await;
    match guard.as_ref()
    { Some(q) => q.clone(),
      None =>
      { let msg = "ticker_loop: QuoteToken disappeared after acquire";
        log::debug!("TTRS_TL EXIT FAIL - {msg}");
        let _ = error_tx.send(CoreError::Unrecoverable(msg.to_string()));
        return Err(anyhow::anyhow!(msg));
      }
    }
  };
  log::trace!("// ------------------------------------------------------------");
  log::debug!("TTRS_TL 2. Connect WebSocket");
  log::trace!("// ------------------------------------------------------------");
  let ws_url = match Url::parse(&qt.url)
  { Ok(u) => u,
    Err(e) =>
    { let msg = format!("ticker_loop: invalid dxlink URL: {e}");
      log::debug!("TTRS_TL EXIT FAIL - {msg}");
      let _ = error_tx.send(CoreError::WebSocket(msg.clone()));
      return Err(anyhow::anyhow!(msg));
    }
  };
  let (ws_stream, _) = match connect_async(ws_url).await
  { Ok(pair) => pair,
    Err(e) =>
    { let msg = format!("ticker_loop: WebSocket connect failed: {e}");
      log::debug!("TTRS_TL EXIT FAIL - {msg}");
      let _ = error_tx.send(CoreError::WebSocket(msg.clone()));
      return Err(anyhow::anyhow!(msg));
    }
  };

  let _ = error_tx.send(CoreError::ValidStartup(lcounter));

  let (mut ws_sink, mut ws_src) = ws_stream.split();
  log::trace!("// ------------------------------------------------------------");
  log::debug!("TTRS_TL 3. DXLink handshake");
  log::trace!("// ------------------------------------------------------------");
  macro_rules! send_dx
  { ($msg:expr, $desc:expr) =>
    {{ if let Err(e) = ws_sink.send(Message::Text($msg.to_string())).await
      { let msg = format!("ticker_loop: {}: {e}", $desc);
        log::debug!("TTRS_TL EXIT FAIL - {msg}");
        let _ = error_tx.send(CoreError::WebSocket(msg.clone()));
        return Err(anyhow::anyhow!(msg));
      }
    }};
  }
  log::trace!("TTRS_TL: sending SETUP");
  send_dx!
  ( r#"{"type":"SETUP","channel":0,"version":"0.1-DXF-JS/0.3.0","keepaliveTimeout":60,"acceptKeepaliveTimeout":60}"#
  , "failed to send SETUP"
  );
  log::trace!("TTRS_TL: sending AUTH");
  send_dx!
  ( format!(r#"{{"type":"AUTH","channel":0,"token":"{}"}}"#, qt.token)
  , "failed to send AUTH"
  );
  log::trace!("TTRS_TL: sending CHANNEL_REQUEST");
  send_dx!
  ( r#"{"type":"CHANNEL_REQUEST","channel":3,"service":"FEED","parameters":{"contract":"AUTO"}}"#
  , "failed to send CHANNEL_REQUEST"
  );
  log::trace!("TTRS_TL: sending FEED_SETUP (COMPACT)");
  send_dx!
  ( r#"{
        "type":"FEED_SETUP",
        "channel":3,
        "acceptAggregationPeriod":0.1,
        "acceptDataFormat":"COMPACT",
        "acceptEventFields":{"Quote":["eventType","eventSymbol","bidPrice","askPrice","bidSize","askSize"]}
    }"#
  , "failed to send FEED_SETUP"
  );
  let mut routes: HashMap<String, (String, mpsc::Sender<StreamData>)> = HashMap::new();
  let mut hb_interval = interval(Duration::from_secs(30));
  log::trace!("// ------------------------------------------------------------");
  log::debug!("TTRS_TL 4. Main loop");
  log::trace!("// ------------------------------------------------------------");
  loop
  { tokio::select!
    { Some(cmd) = rx.recv() =>
      { match cmd
        { TickerLoopCmd::AddRoute(user_sym, streamer_sym, user_tx) =>
          { log::trace!("TTRS_TL: AddRoute user={user_sym} streamer={streamer_sym}");
            routes.insert(streamer_sym.clone(), (user_sym.clone(), user_tx));
            // TODO: alloow this to be set by the user:
            let sub = if streamer_sym.starts_with('.')
            { format!
              ( r#"{{
                    "type":"FEED_SUBSCRIPTION",
                    "channel":3,
                    "reset":false,
                    "add":[
                        {{"type":"Quote", "symbol":"{}"}},
                        {{"type":"Trade", "symbol":"{}"}},
                        {{"type":"Profile", "symbol":"{}"}},
                        {{"type":"Summary", "symbol":"{}"}},
                        {{"type":"Greeks", "symbol":"{}"}}
                    ]
                }}"#
                , streamer_sym, streamer_sym, streamer_sym
                , streamer_sym, streamer_sym, 
              )
            } 
            else
            { format!
              ( r#"{{
                    "type":"FEED_SUBSCRIPTION",
                    "channel":3,
                    "reset":false,
                    "add":[
                        {{"type":"Quote", "symbol":"{}"}},
                        {{"type":"Trade", "symbol":"{}"}},
                        {{"type":"Profile", "symbol":"{}"}},
                        {{"type":"Summary", "symbol":"{}"}}
                    ]
                }}"#
                , streamer_sym, streamer_sym, streamer_sym
                , streamer_sym,
              )
            };

            if let Err(e) = ws_sink.send(Message::Text(sub)).await
            { let msg = format!("ticker_loop: subscription failed for {streamer_sym}: {e}");
              log::debug!("TTRS_TL EXIT FAIL - {msg}");
              let _ = error_tx.send(CoreError::WebSocket(msg.clone()));
              return Err(anyhow::anyhow!(msg));
            }
          }
          TickerLoopCmd::Shutdown =>
          { log::debug!("TTRS_TL: received Shutdown - exiting loop");
            break;
          }
        }
      }
      Some(ws_result) = ws_src.next() =>
      { match ws_result
        { Ok(msg) =>
          { if let Message::Text(text) = msg
            { match serde_json::from_str::<Value>(&text)
              { Ok(v) if v["type"].as_str() == Some("FEED_DATA") =>
                { log::trace!("TTRS_TL: FEED_DATA raw: {text}");
                  let data_arr = match v["data"].as_array()
                  { Some(a) => a,
                    None =>
                    { let msg = format!("TTRS_TL: FEED_DATA without data array: {v:?}");
                      log::trace!("{msg}");
                      let _ = error_tx.send(CoreError::ParseWarning(msg));
                      continue;
                    }
                  };
                  let mut i = 0;
                  while i + 1 < data_arr.len()
                  { let ev_type = match data_arr[i].as_str()
                    { Some(s) => s,
                      None =>
                      { let msg = format!
                        ( "TTRS_TL: expected event type string at index {i} : {:#?}"
                        , data_arr[i]
                        );
                        log::trace!("{msg}");
                        let _ = error_tx.send(CoreError::ParseWarning(msg));
                        i += 2;
                        continue;
                      }
                    };
                    let inner_val = &data_arr[i + 1];

                    let to_opt_f64 = |v: &Value| -> Option<f64>
                    { match v
                      { Value::Null => None,
                        Value::Number(n) => n.as_f64(),
                        Value::String(s) =>
                        { let s = s.trim();
                          if s.is_empty() || s == "NaN" { None } else { s.parse::<f64>().ok() }
                        },
                        _ => None,
                      }
                    };
                    match ev_type 
                    {
                      "Quote" => 
                      {
                          if let Some(inner_arr) = inner_val.as_array() {
                              let mut j = 0;
                              while j < inner_arr.len() {
                                  // Each Quote event: ["Quote", symbol, bid_price, ask_price, bid_size, ask_size]
                                  if j + 5 >= inner_arr.len() {
                                      break; // incomplete
                                  }
                                  if inner_arr[j].as_str() != Some("Quote") {
                                      j += 1;
                                      continue;
                                  }
                                  let symbol = inner_arr[j + 1].as_str().unwrap_or("").to_string();
                                  
                                  let bid = to_opt_f64(&inner_arr[j + 2]);
                                  let ask = to_opt_f64(&inner_arr[j + 3]);
                                  let bid_size = to_opt_f64(&inner_arr[j + 4]);
                                  let ask_size = to_opt_f64(&inner_arr[j + 5]);

                                  let quote = StreamQuote {
                                      event_type: "Quote".into(),
                                      symbol: symbol.clone(),
                                      bid_price: bid,
                                      ask_price: ask,
                                      bid_size,
                                      ask_size,
                                  };

                                  if let Some((user_sym, tx)) = routes.get(&symbol) {
                                      let mut q = quote;
                                      q.symbol = user_sym.clone();
                                      let _ = tx.send(StreamData::Quote(q)).await;
                                  }

                                  j += 6;
                              }
                          }
                      }
                      ,
                      "Profile" | "Summary" | "Trade" => 
                      {
                          // Handle batched non-Quote events
                          if let Some(inner_arr) = inner_val.as_array() {
                              let mut j = 0;
                              while j < inner_arr.len() {
                                  if j + 1 >= inner_arr.len() {
                                      break;
                                  }
                                  let event_type = match inner_arr[j].as_str() {
                                      Some(s) => s,
                                      None => { j += 1; continue; }
                                  };

                                  // Skip if not the expected type (in case mixed batch)
                                  if event_type != ev_type {
                                      j += 1;
                                      continue;
                                  }

                                  let event_data = &inner_arr[j + 1..]; // remaining fields for this event
                                  let raw_val = Value::Array(inner_arr[j..].to_vec());

                                  match ev_type {
                                      "Trade" => {
                                          if let Ok(raw) = serde_json::from_value::<RawTrade>(raw_val.clone()) {
                                              let mut trade: StreamTrade = raw.into();
                                              if let Some((user_sym, tx)) = routes.get(&trade.symbol) {
                                                  trade.symbol = user_sym.clone();
                                                  let _ = tx.send(StreamData::Trade(trade)).await;
                                              }
                                          } else {
                                              log::error!("Trade parse FAILURE | raw={:?}", raw_val);
                                          }
                                      }
                                      "Summary" => {
                                          if let Ok(raw) = serde_json::from_value::<RawSummary>(raw_val.clone()) {
                                              let mut summary: StreamSummary = raw.into();
                                              if let Some((user_sym, tx)) = routes.get(&summary.symbol) {
                                                  summary.symbol = user_sym.clone();
                                                  let _ = tx.send(StreamData::Summary(summary)).await;
                                              }
                                          } else {
                                              log::error!("Summary parse FAILURE | raw={:?}", raw_val);
                                          }
                                      }
                                      "Profile" => {
                                          if let Ok(raw) = serde_json::from_value::<RawProfile>(raw_val.clone()) {
                                              let mut profile: StreamProfile = raw.into();
                                              if let Some((user_sym, tx)) = routes.get(&profile.symbol) {
                                                  profile.symbol = user_sym.clone();
                                                  let _ = tx.send(StreamData::Profile(profile)).await;
                                              }
                                          } else {
                                              log::error!("Profile parse FAILURE | raw={:?}", raw_val);
                                          }
                                      }
                                      _ => {}
                                  }

                                  // Advance by number of fields used (1 for type + fields)
                                  // We don't know exact length, so just move past this event
                                  j += event_data.len() + 1;
                              }
                          }
                      }
                      ,
                     "Greeks" => 
                      { if let Some(inner_arr) = inner_val.as_array() 
                        { let mut j = 0;
                          while j + 13 < inner_arr.len() 
                          { // Need at least 14 elements
                            if inner_arr[j].as_str() != Some("Greeks") 
                            { j += 1;
                              continue;
                            } 
                            let symbol = inner_arr[j + 1]
                              .as_str()
                              .unwrap_or("")
                              .to_string();

                            // Correct mapping based on real tastytrade/dxfeed output
                            let event_time   = to_opt_f64(&inner_arr[j + 2]);  // often 0
                            // skip flags, index
                            let calc_time    = to_opt_f64(&inner_arr[j + 5]);  // the big timestamp (e.g. 1735948219000)
                            let _sequence    = to_opt_f64(&inner_arr[j + 6]);

                            let theo_price   = to_opt_f64(&inner_arr[j + 7]);
                            let volatility   = to_opt_f64(&inner_arr[j + 8]);
                            let delta        = to_opt_f64(&inner_arr[j + 9]);
                            let gamma        = to_opt_f64(&inner_arr[j + 10]);
                            let theta        = to_opt_f64(&inner_arr[j + 11]);
                            let vega         = to_opt_f64(&inner_arr[j + 12]);
                            let rho          = to_opt_f64(&inner_arr[j + 13]);

                            let greeks = StreamGreeks {
                                symbol: symbol.clone(),
                                event_time,
                                calc_time,
                                theo_price,
                                volatility,
                                delta,
                                gamma,
                                theta,
                                vega,
                                rho,
                            };

                            if let Some((user_sym, tx)) = routes.get(&symbol) 
                            { let mut g = greeks;
                              g.symbol = user_sym.clone();
                              let _ = tx.send(StreamData::Greeks(g)).await;
                            }

                            j += 14;  // Skip past all 14 fields
                          }
                        }
                      }
                      ,
                      other =>
                      { let msg = format!("TTRS_TL: unhandled FEED_DATA event type: {other}");
                        log::warn!("{msg}");
                        let _ = error_tx.send(CoreError::ParseWarning(msg));
                      }
                    }
                    i += 2;
                  }
                }
                Ok(v) if v["type"].as_str() == Some("KEEPALIVE") =>
                { let _ = error_tx.send
                  ( CoreError::ParseWarning
                   ( format!
                     ( "KEEPALIVE:{:#?}", v
                     )
                   )
                  );
                }
                Ok(v) =>
                { let msg = format!
                  ( "TTRS_TL: ignored message type: {} {}"
                  , v["type"]
                  , { if v["type"] == "ERROR"
                      { format!("=> {:#?}",v)
                      } 
                      else
                      { format!("...")
                      }
                    }
                  );
                  log::warn!("{msg}");
                  let _ = error_tx.send(CoreError::ParseWarning(msg));
                }
                Err(e) =>
                { let msg = format!("TTRS_TL: JSON parse error: {e} | raw: {text}");
                  log::warn!("{msg}");
                  let _ = error_tx.send(CoreError::ParseFailure(msg));
                }
              }
            }
          }
          Err(e) =>
          { let msg = format!("ticker_loop: WebSocket stream error: {e}");
            log::debug!("TTRS_TL EXIT FAIL - {msg}");
            let _ = error_tx.send(CoreError::WebSocket(msg.clone()));
            return Err(anyhow::anyhow!(msg));
          }
        }
      }
      _ = hb_interval.tick() =>
      { let hb = r#"{"type":"KEEPALIVE","channel":0}"#.to_string();
        if let Err(e) = ws_sink.send(Message::Text(hb)).await
        { let msg = format!("ticker_loop: heartbeat failed: {e}");
          log::debug!("TTRS_TL EXIT FAIL - {msg}");
          let _ = error_tx.send(CoreError::WebSocket(msg.clone()));
          return Err(anyhow::anyhow!(msg));
        }
      }
    }
  }
  log::trace!("// ------------------------------------------------------------");
  log::debug!("TTRS_TL: GOOD EXIT - graceful shutdown complete");
  log::trace!("// ------------------------------------------------------------");
  Ok(())
}

pub(crate) async fn account_stream_loop
(   acct: String
  , user_tx: mpsc::Sender<Value>
  , conn: ConnectionInfo
  , shared_token: Arc<Mutex<Option<AccessToken>>>
  , error_tx: mpsc::UnboundedSender<CoreError>  
) 
{ if let Err(e) = account_stream_loop_inner
  (   acct
    , user_tx
    , conn
    , shared_token
    , error_tx.clone()
    ,
  ).await 
  { let msg = format!("Account stream failed: {}", e);
    log::error!("{}", msg);
    let _ = error_tx.send(CoreError::WebSocket(msg));
  }
}

async fn account_stream_loop_inner
( acct: String
  , user_tx: mpsc::Sender<Value>
  , conn: ConnectionInfo
  , shared_token: Arc<Mutex<Option<AccessToken>>>
  , error_tx: mpsc::UnboundedSender<CoreError>
) -> Result<()> 
{ let http = Client::new();
  let access = ensure_token(&http, &conn.base_url, &conn.oauth, shared_token).await
    .context("Failed to get access token for account stream")?;
  let streamer_url = if conn.base_url.contains("cert") 
  { "wss://streamer.cert.tastyworks.com"
  } 
  else 
  { "wss://streamer.tastyworks.com"
  }.to_string();
  let url = Url::parse(&streamer_url)
    .context("Failed to parse streamer URL")?;
  let (ws_stream, _) = connect_async(url).await
    .context("Failed to connect to account stream WebSocket")?;
  let (ws_sink, mut ws_src) = ws_stream.split();
  let ws_sink = Arc::new(Mutex::new(ws_sink));
  // Connect
  let connect_msg = serde_json::json!
  ( {
      "action": "connect",
      "value": [acct],
      "auth-token": access.clone(),
      "request-id": rand::thread_rng().gen::<u64>()
    }
  );
  ws_sink.lock().await.send(Message::Text(connect_msg.to_string())).await
    .context("Failed to send connect message")?;
   
  // Heartbeat task
  let ws_sink_hb = ws_sink.clone();
  let access_hb = access.clone();
  let error_tx_hb = error_tx.clone();
  tokio::spawn
  ( async move 
    { let mut interval = interval(Duration::from_secs(30));
      loop 
      { interval.tick().await;
        let hb = serde_json::json!
        ( {
            "action": "heartbeat",
            "auth-token": &access_hb,
            "request-id": rand::thread_rng().gen::<u64>()
          }
        );

        if let Err(e) = ws_sink_hb.lock().await.send(Message::Text(hb.to_string())).await 
        { let msg = format!("Account stream heartbeat failed: {}", e);
          log::warn!("{}", msg);
          let _ = error_tx_hb.send(CoreError::WebSocket(msg));
          break;
        }
      }
    }
  );

  // Receive loop
  loop 
  {
    if let Some(msg) = ws_src.next().await 
    { match msg 
      { Ok(Message::Text(text)) => 
        { if let Ok(v) = serde_json::from_str::<Value>(&text) 
          { // Filter notifications (no "status" key)
            if v.get("status").is_none() && v.get("type").is_some() && v.get("data").is_some() 
            { if let Err(e) = user_tx.send(v).await 
              { return Err(anyhow::anyhow!("Failed to forward account stream data: {}", e));
              }
            }
          }
        },
        Ok(_) => 
        { log::warn!("Received non-text WebSocket message in account stream");
        },
        Err(e) => 
        { return Err(anyhow::anyhow!("WebSocket error in account stream: {}", e));
        }
      }
    } 
    else 
    { return Err(anyhow::anyhow!("Account stream WebSocket closed"));
    }
  }
}