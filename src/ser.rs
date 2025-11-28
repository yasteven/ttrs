//ttrs/src/ser.rs - seralization functions
// src/ser.rs
use serde::{de::{Deserializer, Visitor}, Deserialize};
use serde_json::Value;
use std::fmt;

/// Flex f64: accepts number, int, or string → f64
pub(crate) fn de_f64_flex<'de, D>(deserializer: D) 
-> Result<f64, D::Error> 
where D: Deserializer<'de>,
{ struct F64FlexVisitor;
  impl<'de> Visitor<'de> for F64FlexVisitor 
  { type Value = f64;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result 
    { f.write_str("f64|i64|u64|string representation of float")
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> 
    { Ok(v as f64) 
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> 
    { Ok(v as f64) 
    }
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> 
    { Ok(v) 
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) 
    -> Result<Self::Value, E> 
    { v.parse::<f64>().map_err(serde::de::Error::custom)
    }
  }
  deserializer.deserialize_any(F64FlexVisitor)
}

/// Option<f64> that also accepts strings and treats "NaN"/null as None
pub(crate) fn de_opt_f64_flex<'de, D>(deserializer: D) 
-> Result<Option<f64>, D::Error>
where D: Deserializer<'de>,
{ let opt = Option::<Value>::deserialize(deserializer)?;
  match opt 
  { Some(val) => 
    { if val.is_null() { return Ok(None); }
      let f = match val 
      { Value::Number(n) => n.as_f64().ok_or_else(|| serde::de::Error::custom("invalid number"))?,
        Value::String(s) => s.parse::<f64>().map_err(serde::de::Error::custom)?,
        _ => return Err(serde::de::Error::custom("invalid type for Option<f64>")),
      };
      Ok(Some(f))
    }
  , None => Ok(None),
  }
}

/// "NaN" → None, null → None, empty string → None
pub(crate) fn de_opt_f64_nan<'de, D>(deserializer: D) 
-> Result<Option<f64>, D::Error>
where D: Deserializer<'de>,
{ let opt = Option::<Value>::deserialize(deserializer)?;
  let Some(val) = opt else { return Ok(None); };
  match val 
  { Value::Null => Ok(None),
    Value::Number(n) => n.as_f64().ok_or_else(|| serde::de::Error::custom("invalid number")).map(Some),
    Value::String(s) => 
    { let s = s.trim();
      if s.is_empty() || s.eq_ignore_ascii_case("NaN") 
      { Ok(None)
      } else 
      { s.parse::<f64>().map(Some).map_err(|_| serde::de::Error::custom("invalid numeric string"))
      }
    }
    other => Err(serde::de::Error::custom(format!("unexpected value for f64: {other:?}"))),
  }
}

/// Accept string or number → String
pub(crate) fn de_string_or_number<'de, D>(deserializer: D) 
-> Result<String, D::Error> where
D: Deserializer<'de>,
{ let val = Value::deserialize(deserializer)?;
  Ok
  ( match val 
    { Value::String(s) => s,
      Value::Number(n) => n.to_string(),
      _ => return Err(serde::de::Error::custom("expected string or number")),
    }
  )
}

pub(crate) fn de_opt_string_or_number<'de, D>(deserializer: D) 
-> Result<Option<String>, D::Error>
where D: Deserializer<'de>,
{ let opt = Option::<Value>::deserialize(deserializer)?;
  Ok
  ( opt.map
    ( |v| match v 
      { Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        _ => "".to_string(),
      }
    )
  )
}

// =============================================================================
// Internal Types (pub(crate))
// =============================================================================

use crate::dat::*;
use std::collections::HashMap;

// ---------- raw tuple structs for serde (positional arrays) ----------
// RawTrade maps to ["Trade", symbol, event_id, time, sequence, trade_day, exchange_code, price, change, size, day_id, day_volume, day_turnover, direction, is_eth]
#[derive(Debug, Deserialize)]
pub(crate) struct RawTrade
( String, // "Trade"
  String, // symbol
  #[serde(deserialize_with = "de_f64_flex")]
  f64,    // event_id
  #[serde(deserialize_with = "de_f64_flex")]
  f64,    // time
  #[serde(deserialize_with = "de_f64_flex")]
  f64,    // sequence
  #[serde(deserialize_with = "de_f64_flex")]
  f64,    // trade_day
  String, // exchange_code
  #[serde(deserialize_with = "de_f64_flex")]
  f64,    // price
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>, // change
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>, // size
  #[serde(deserialize_with = "de_f64_flex")]
  f64,    // day_id
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>, // day_volume
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>, // day_turnover
  String, // direction
  bool    // is_eth
);
// Convert RawTrade -> StreamTrade 
impl From<RawTrade> for StreamTrade 
{ fn from(r: RawTrade) -> Self 
  { StreamTrade 
    {   symbol:        r.1
      , event_id:      r.2
      , time:          r.3
      , sequence:      r.4
      , trade_day:     r.5
      , exchange_code: r.6
      , price:         r.7
      , change:        r.8
      , size:          r.9
      , day_id:        r.10
      , day_volume:    r.11
      , day_turnover:  r.12
      , direction:     r.13
      , is_eth:        r.14
      , extra: Default::default()
      ,   
    }
  }
}

// RawSummary -> ["Summary", symbol, event_id, day_id, open, high, low, close, session, prev_day_id, prev_day_close, close_type, volume, open_interest]
#[derive(Debug, Deserialize)]
pub(crate) struct RawSummary
(
  String,
  String,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  String,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  String,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
);
impl From<RawSummary> for StreamSummary 
{ fn from(r: RawSummary) -> Self 
  { StreamSummary 
    {   symbol:        r.1
      , event_id:      r.2
      , day_id:        r.3
      , day_open:      r.4
      , day_high:      r.5
      , day_low:       r.6
      , day_close:     r.7
      , session:       r.8
      , prev_day_id:   r.9
      , prev_day_close: r.10
      , close_type:    r.11
      , volume:        r.12
      , open_interest: r.13
      , extra: Default::default(),
    }
  }
}

// RawProfile -> ["Profile", symbol, event_id, description, market_status, status, halt_status, open_time, close_time, avg_volume, avg_volume_30d, high_52w, low_52w, div_amount, div_yield, div_freq, div_ex_date, beta, earnings_per_share, market_cap]
#[derive(Debug, Deserialize)]
pub(crate) struct RawProfile
(
  String,
  String,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  String,
  String,
  String,
  Option<Value>, // can be null
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_f64_flex")]
  f64,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  Option<f64>,
);

impl From<RawProfile> for StreamProfile 
{ fn from(r: RawProfile) -> Self 
  { StreamProfile 
    {
      symbol: r.1,
      event_id: r.2,
      description: r.3,
      market_status: r.4,
      status: r.5,
      halt_status: r.6.and_then(|v| v.as_f64()),
      open_time: r.7,
      close_time: r.8,
      avg_volume: r.9,
      avg_volume_30d: r.10,
      high_52w: r.11,
      low_52w: r.12,
      div_amount: r.13,
      div_yield: r.14,
      div_freq: r.15,
      div_ex_date: r.16,
      beta: r.17,
      earnings_per_share: r.18,
      market_cap: r.19,
      extra: Default::default(),
    }
  }
}

// ---------- Quote batch parsing helper ----------
// Might be slow compared to the single quote...
// dxFeed Quote messages come batched in an inner array. We parse the inner Vec<Value>
// by stepping through values, skipping repeated "Quote" markers, and extracting 5-field groups:
//   [ "Quote", sym, bid, ask, bid_size, ask_size, "Quote", sym2, ... ]
#[derive(Debug, Deserialize)]
pub(crate) struct RawQuoteBatch(pub String, pub Vec<Value>); // ("Quote", inner_array)
impl RawQuoteBatch 
{ /// Convert the inner batched array into Vec<StreamQuote> 
  pub fn into_quotes(self) -> Vec<StreamQuote> 
  {
    let mut out = Vec::new();
    let inner = self.1;
    let mut i = 0usize;
    while i < inner.len() 
    { // skip extra "Quote" markers
      if inner[i].as_str() == Some("Quote") 
      { i += 1;
        continue;
      }
      // need symbol + 4 numeric fields => 5 values
      if i + 4 >= inner.len() { break; }

      let symbol = inner[i].as_str().unwrap_or("").to_string();

      // helper to coerce Value -> Option<f64> treating "NaN" and null as None
      let to_opt_f64 = |v: &Value| -> Option<f64> 
      { match v 
        {   Value::Null => None
          , Value::Number(n) => n.as_f64()
          , Value::String(s) => 
            { let s = s.trim();
              if s.is_empty() || s == "NaN" { None } else { s.parse::<f64>().ok() }
            }
          , _ => None
        }
      };

      let bid      = to_opt_f64(&inner[i+1]);
      let ask      = to_opt_f64(&inner[i+2]);
      let bid_size = to_opt_f64(&inner[i+3]);
      let ask_size = to_opt_f64(&inner[i+4]);

      let q = StreamQuote 
      {
        event_type: "Quote".into(),
        symbol: symbol.clone(),
        bid_price: bid,
        ask_price: ask,
        bid_size,
        ask_size,
      };
      out.push(q);
      i += 5;
    }
    out
  }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct QuotesResponse 
{ pub data: QuotesData,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct QuotesData 
{ pub items: Vec<StreamQuote>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MarketMetricsResponse 
{ pub data: MarketMetricsData,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CustomerAccountsResponse 
{
  pub context: String,
  pub data: AccountsData,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PositionsResponse 
{ pub context: String,
  pub data: PositionsData,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BalancesResponse 
{ pub context: String,
  pub data: Balance,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TradingStatusResponse 
{ pub context: String,
  pub data: TradingStatus,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TransactionsResponse 
{ pub context: String,
  pub data: TransactionsData,
  pub pagination: Option<Pagination>,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Pagination 
{ #[serde(rename = "current-item-count")]
  pub current_item_count: u32,
  #[serde(rename = "item-offset")]
  pub item_offset: u32,
  #[serde(rename = "next-link")]
  pub next_link: Option<String>,
  #[serde(rename = "page-offset")]
  pub page_offset: u32,
  #[serde(rename = "per-page")]
  pub per_page: u32,
  #[serde(rename = "previous-link")]
  pub previous_link: Option<String>,
  #[serde(rename = "total-items")]
  pub total_items: u32,
  #[serde(rename = "total-pages")]
  pub total_pages: u32,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MarginRequirementsResponse 
{ pub data: MarginRequirements,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DryRunResponse 
{ pub data: DryRunData,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone)]
pub(crate) struct AccessToken 
{ pub token: String,
  pub expires_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct QuoteToken 
{ pub token: String,
  pub url: String,
  pub expires_at: std::time::Instant,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct QuoteTokenResp 
{ pub data: QuoteTokenData,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct QuoteTokenData 
{ pub token: String,
  #[serde(rename = "dxlink-url")]
  pub dxlink_url: String,
  pub level: String,
}

// Single-order / live-order responses
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrderSubmitResp 
{ pub data: OrderSubmitData,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrderSubmitData 
{ pub order: OrderResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SingleOrderResponse 
{ pub data: OrderResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LiveOrdersResponse 
{ pub data: LiveOrdersData,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LiveOrdersData 
{ pub items: Vec<OrderResponse>,
  #[serde(default)]
  pub context: Option<String>,
}