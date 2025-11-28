// src/dat.rs
// =============================================================================
// ttrs/src/dat.rs - Tastytrade API Types (Nov 2025)
// =============================================================================
  use serde::{Deserialize, Serialize};
  use serde_json::Value;
  use std::collections::HashMap;
  pub(crate) use crate::ser::*; // ← brings all deserializers into scope cleanly

// =================================================================
// Public Types
// =================================================================

// -----------------------------------------------------------------
// Connection
// -----------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct ConnectionInfo 
{ pub base_url: String,
  pub oauth: OauthInfo,
}
pub type OauthInfo = 
( String // client_id
, String // client_secret
, String // refresh_token
); 

// -----------------------------------------------------------------------------
// Account & Trading Status
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountInfo 
{ #[serde(rename = "account-number")]
  pub account_number: String ,
  #[serde(rename = "account-type-name")]
  pub account_type_name: String,
  #[serde(rename = "created-at")]
  pub created_at: String,
  #[serde(rename = "day-trader-status")]
  pub day_trader_status: bool,
  #[serde(rename = "ext-crm-id")]
  pub ext_crm_id: Option<String>,
  #[serde(rename = "external-id")]
  pub external_id: Option<String>,
  #[serde(rename = "funding-date")]
  pub funding_date: Option<String>,
  #[serde(rename = "futures-account-purpose")]
  pub futures_account_purpose: Option<String>,
  #[serde(rename = "investment-objective")]
  pub investment_objective: Option<String>,
  #[serde(rename = "is-closed")]
  pub is_closed: bool,
  #[serde(rename = "is-firm-error")]
  pub is_firm_error: bool,
  #[serde(rename = "is-firm-proprietary")]
  pub is_firm_proprietary: bool,
  #[serde(rename = "is-foreign")]
  pub is_foreign: bool,
  #[serde(rename = "is-futures-approved")]
  pub is_futures_approved: bool,
  #[serde(rename = "margin-or-cash")]
  pub margin_or_cash: String,
  pub nickname: String,
  #[serde(rename = "opened-at")]
  pub opened_at: String,
  #[serde(rename = "regulatory-domain")]
  pub regulatory_domain: Option<String>,
  #[serde(rename = "suitable-options-level")]
  pub suitable_options_level: String,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountsData 
{ pub items: Vec<AccountItem>,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountItem 
{ pub account: AccountInfo,
  #[serde(rename = "authority-level")]
  pub authority_level: String,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingStatus 
{ pub id: u64,
  #[serde(rename = "account-number")]
  pub account_number: String,
  #[serde(rename = "day-trade-count")]
  pub day_trade_count: u32,
  #[serde(rename = "are-deep-itm-carry-options-enabled")]
  pub are_deep_itm_carry_options_enabled: bool,
  #[serde(rename = "are-far-otm-net-options-restricted")]
  pub are_far_otm_net_options_restricted: bool,
  #[serde(rename = "are-options-values-restricted-to-nlv")]
  pub are_options_values_restricted_to_nlv: bool,
  #[serde(rename = "are-single-tick-expiring-hedges-ignored")]
  pub are_single_tick_expiring_hedges_ignored: bool,
  #[serde(rename = "enhanced-fraud-safeguards-enabled-at")]
  pub enhanced_fraud_safeguards_enabled_at: Option<String>,
  #[serde(rename = "equities-margin-calculation-type")]
  pub equities_margin_calculation_type: String,
  #[serde(rename = "ext-crm-id")]
  pub ext_crm_id: Option<String>,
  #[serde(rename = "fee-schedule-name")]
  pub fee_schedule_name: String,
  #[serde(rename = "futures-margin-rate-multiplier", deserialize_with = "de_f64_flex")]
  pub futures_margin_rate_multiplier: f64,
  #[serde(rename = "has-intraday-equities-margin")]
  pub has_intraday_equities_margin: bool,
  #[serde(rename = "is-aggregated-at-clearing")]
  pub is_aggregated_at_clearing: bool,
  #[serde(rename = "is-closed")]
  pub is_closed: bool,
  #[serde(rename = "is-closing-only")]
  pub is_closing_only: bool,
  #[serde(rename = "is-cryptocurrency-closing-only")]
  pub is_cryptocurrency_closing_only: bool,
  #[serde(rename = "is-cryptocurrency-enabled")]
  pub is_cryptocurrency_enabled: bool,
  #[serde(rename = "is-equity-offering-closing-only")]
  pub is_equity_offering_closing_only: bool,
  #[serde(rename = "is-equity-offering-enabled")]
  pub is_equity_offering_enabled: bool,
  #[serde(rename = "is-frozen")]
  pub is_frozen: bool,
  #[serde(rename = "is-full-equity-margin-required")]
  pub is_full_equity_margin_required: bool,
  #[serde(rename = "is-futures-closing-only")]
  pub is_futures_closing_only: bool,
  #[serde(rename = "is-futures-enabled")]
  pub is_futures_enabled: bool,
  #[serde(rename = "is-futures-intra-day-enabled")]
  pub is_futures_intra_day_enabled: bool,
  #[serde(rename = "is-in-day-trade-equity-maintenance-call")]
  pub is_in_day_trade_equity_maintenance_call: bool,
  #[serde(rename = "is-in-margin-call")]
  pub is_in_margin_call: bool,
  #[serde(rename = "is-pattern-day-trader")]
  pub is_pattern_day_trader: bool,
  #[serde(rename = "is-roll-the-day-forward-enabled")]
  pub is_roll_the_day_forward_enabled: bool,
  #[serde(rename = "is-small-notional-futures-intra-day-enabled")]
  pub is_small_notional_futures_intra_day_enabled: bool,
  #[serde(rename = "options-level")]
  pub options_level: String,
  #[serde(rename = "pdt-reset-on")]
  pub pdt_reset_on: Option<String>,
  #[serde(rename = "short-calls-enabled")]
  pub short_calls_enabled: bool,
  #[serde(rename = "small-notional-futures-margin-rate-multiplier", deserialize_with = "de_f64_flex")]
  pub small_notional_futures_margin_rate_multiplier: f64,
  #[serde(rename = "updated-at")]
  pub updated_at: String,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

// -----------------------------------------------------------------------------
// Positions & Balances & Transactions
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Position 
{
  #[serde(rename = "account-number")]
  pub account_number: String,
  #[serde(rename = "instrument-type")]
  pub instrument_type: String,
  #[serde(rename = "streamer-symbol")]
  pub streamer_symbol: String,
  pub symbol: String,
  #[serde(rename = "underlying-symbol")]
  pub underlying_symbol: String,
  #[serde(deserialize_with = "de_f64_flex")]  
  pub quantity: f64,
  #[serde(rename = "quantity-direction")]
  pub quantity_direction: String,
  #[serde(rename = "close-price", deserialize_with = "de_f64_flex")]
  pub close_price: f64,
  #[serde(rename = "average-open-price", deserialize_with = "de_f64_flex")]
  pub average_open_price: f64,
  #[serde(rename = "average-yearly-market-close-price", deserialize_with = "de_f64_flex")]
  pub average_yearly_market_close_price: f64,
  #[serde(rename = "average-daily-market-close-price", deserialize_with = "de_f64_flex")]
  pub average_daily_market_close_price: f64,
  #[serde(deserialize_with = "de_f64_flex")]
  pub multiplier: f64,
  #[serde(rename = "cost-effect")]
  pub cost_effect: String,
  #[serde(rename = "is-suppressed")]
  pub is_suppressed: bool,
  #[serde(rename = "is-frozen")]
  pub is_frozen: bool,
  #[serde(rename = "restricted-quantity", deserialize_with = "de_f64_flex")]  
  pub restricted_quantity: f64,
  #[serde(rename = "realized-day-gain", deserialize_with = "de_f64_flex")]
  pub realized_day_gain: f64,
  #[serde(rename = "realized-day-gain-effect")]
  pub realized_day_gain_effect: String,
  #[serde(rename = "realized-day-gain-date")]
  pub realized_day_gain_date: String,
  #[serde(rename = "realized-today", deserialize_with = "de_f64_flex")]
  pub realized_today: f64,
  #[serde(rename = "realized-today-effect")]
  pub realized_today_effect: String,
  #[serde(rename = "realized-today-date")]
  pub realized_today_date: String,
  #[serde(rename = "created-at")]
  pub created_at: String,
  #[serde(rename = "updated-at")]
  pub updated_at: String,
  #[serde(default, rename = "expires-at")]
  pub expires_at: Option<String>,
  #[serde(default, rename = "fixing-price", deserialize_with = "de_opt_f64_flex")]
  pub fixing_price: Option<f64>,
  #[serde(default, rename = "deliverable-type")]
  pub deliverable_type: Option<String>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PositionsData 
{ pub items: Vec<Position>,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Balance 
{
  #[serde(rename = "account-number")]
  pub account_number: String,
  #[serde(rename = "cash-balance", deserialize_with = "de_f64_flex")]
  pub cash_balance: f64,
  #[serde(rename = "cash-available-to-withdraw", deserialize_with = "de_f64_flex")]
  pub cash_available_to_withdraw: f64,
  #[serde(rename = "net-liquidating-value", deserialize_with = "de_f64_flex")]
  pub net_liquidating_value: f64,
  #[serde(rename = "equity-buying-power", deserialize_with = "de_f64_flex")]
  pub equity_buying_power: f64,
  #[serde(rename = "derivative-buying-power", deserialize_with = "de_f64_flex")]
  pub derivative_buying_power: f64,
  #[serde(rename = "day-trading-buying-power", deserialize_with = "de_f64_flex")]
  pub day_trading_buying_power: f64,
  #[serde(rename = "futures-margin-requirement", deserialize_with = "de_f64_flex")]
  pub futures_margin_requirement: f64,
  #[serde(rename = "available-trading-funds", deserialize_with = "de_f64_flex")]
  pub available_trading_funds: f64,
  #[serde(rename = "maintenance-requirement", deserialize_with = "de_f64_flex")]
  pub maintenance_requirement: f64,
  #[serde(rename = "maintenance-call-value", deserialize_with = "de_f64_flex")]
  pub maintenance_call_value: f64,
  #[serde(rename = "reg-t-call-value", deserialize_with = "de_f64_flex")]
  pub reg_t_call_value: f64,
  #[serde(rename = "reg-t-margin-requirement", deserialize_with = "de_f64_flex")]
  pub reg_t_margin_requirement: f64,
  #[serde(rename = "day-trade-excess", deserialize_with = "de_f64_flex")]
  pub day_trade_excess: f64,
  #[serde(rename = "day-trading-call-value", deserialize_with = "de_f64_flex")]
  pub day_trading_call_value: f64,
  #[serde(rename = "maintenance-excess", deserialize_with = "de_f64_flex")]
  pub maintenance_excess: f64,
  #[serde(rename = "pending-cash", deserialize_with = "de_f64_flex")]
  pub pending_cash: f64,
  #[serde(rename = "pending-cash-effect")]
  pub pending_cash_effect: String,
  #[serde(rename = "long-derivative-value", deserialize_with = "de_f64_flex")]
  pub long_derivative_value: f64,
  #[serde(rename = "short-derivative-value", deserialize_with = "de_f64_flex")]
  pub short_derivative_value: f64,
  #[serde(rename = "long-equity-value", deserialize_with = "de_f64_flex")]
  pub long_equity_value: f64,
  #[serde(rename = "short-equity-value", deserialize_with = "de_f64_flex")]
  pub short_equity_value: f64,
  #[serde(rename = "long-cryptocurrency-value", deserialize_with = "de_f64_flex")]
  pub long_cryptocurrency_value: f64,
  #[serde(rename = "short-cryptocurrency-value", deserialize_with = "de_f64_flex")]
  pub short_cryptocurrency_value: f64,
  #[serde(rename = "cryptocurrency-margin-requirement", deserialize_with = "de_f64_flex")]
  pub cryptocurrency_margin_requirement: f64,
  #[serde(rename = "snapshot-date")]
  pub snapshot_date: String,
  #[serde(rename = "updated-at")]
  pub updated_at: String,
  pub currency: String,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction 
{
  pub id: u64,
  #[serde(rename = "account-number")]
  pub account_number: String,
  #[serde(default, rename = "symbol")]
  pub symbol: Option<String>,
  #[serde(default, rename = "instrument-type")]
  pub instrument_type: Option<String>,
  #[serde(default, rename = "underlying-symbol")]
  pub underlying_symbol: Option<String>,
  #[serde(rename = "transaction-type")]
  pub transaction_type: String,
  #[serde(rename = "transaction-sub-type")]
  pub transaction_sub_type: String,
  pub description: String,
  #[serde(default)]
  pub action: Option<String>,
  #[serde(default, rename = "quantity", deserialize_with = "de_opt_f64_flex")]
  pub quantity: Option<f64>,
  #[serde(default, rename = "price", deserialize_with = "de_opt_f64_flex")]
  pub price: Option<f64>,
  #[serde(rename = "executed-at")]
  pub executed_at: String,
  #[serde(rename = "transaction-date")]
  pub transaction_date: String,
  #[serde(rename = "value", deserialize_with = "de_f64_flex")]
  pub value: f64,
  #[serde(rename = "value-effect")]
  pub value_effect: String,
  #[serde(rename = "net-value", deserialize_with = "de_f64_flex")]
  pub net_value: f64,
  #[serde(rename = "net-value-effect")]
  pub net_value_effect: String,
  #[serde(rename = "is-estimated-fee")]
  pub is_estimated_fee: bool,
  #[serde(default, rename = "order-id")]
  pub order_id: Option<u64>,
  #[serde(default, rename = "commission", deserialize_with = "de_opt_f64_flex")]
  pub commission: Option<f64>,
  #[serde(default, rename = "commission-effect")]
  pub commission_effect: Option<String>,
  #[serde(default, rename = "clearing-fees", deserialize_with = "de_opt_f64_flex")]
  pub clearing_fees: Option<f64>,
  #[serde(default, rename = "clearing-fees-effect")]
  pub clearing_fees_effect: Option<String>,
  #[serde(default, rename = "regulatory-fees", deserialize_with = "de_opt_f64_flex")]
  pub regulatory_fees: Option<f64>,
  #[serde(default, rename = "regulatory-fees-effect")]
  pub regulatory_fees_effect: Option<String>,
  #[serde(default, rename = "proprietary-index-option-fees", deserialize_with = "de_opt_f64_flex")]
  pub proprietary_index_option_fees: Option<f64>,
  #[serde(default, rename = "proprietary-index-option-fees-effect")]
  pub proprietary_index_option_fees_effect: Option<String>,
  #[serde(default, rename = "exec-id")]
  pub exec_id: Option<String>,
  #[serde(default, rename = "destination-venue")]
  pub destination_venue: Option<String>,
  pub currency: String,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransactionsData 
{ pub items: Vec<Transaction>,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}


// -----------------------------------------------------------------------------
// Streaming Data
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamData 
{ Quote(StreamQuote),
  Trade(StreamTrade),
  Summary(StreamSummary),
  Profile(StreamProfile),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamQuote 
{
  #[serde(rename = "eventType")]
  pub event_type: String,
  #[serde(rename = "eventSymbol")]
  pub symbol: String,
  #[serde(rename = "bidPrice", deserialize_with = "de_opt_f64_flex")]
  pub bid_price: Option<f64>,
  #[serde(rename = "askPrice", deserialize_with = "de_opt_f64_flex")]
  pub ask_price: Option<f64>,
  #[serde(rename = "bidSize", deserialize_with = "de_opt_f64_flex")]
  pub bid_size: Option<f64>,
  #[serde(rename = "askSize", deserialize_with = "de_opt_f64_flex")]
  pub ask_size: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamTrade 
{
  pub symbol: String,
  #[serde(deserialize_with = "de_f64_flex")]
  pub event_id: f64, // Often 0; dxFeed: event sequence or ID
  #[serde(deserialize_with = "de_f64_flex")]
  pub time: f64, // Timestamp in millis
  #[serde(deserialize_with = "de_f64_flex")]
  pub sequence: f64, // Often 0; dxFeed sequence number
  #[serde(deserialize_with = "de_f64_flex")]
  pub trade_day: f64, // dxFeed: trade day code
  pub exchange_code: String, // e.g., "Q" (NASDAQ), "" for crypto
  #[serde(deserialize_with = "de_f64_flex")]
  pub price: f64,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub change: Option<f64>, // Price change; "NaN" -> None
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub size: Option<f64>, // Trade size; "NaN" -> None
  #[serde(deserialize_with = "de_f64_flex")]
  pub day_id: f64, // Day ID code
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub day_volume: Option<f64>, // Cumulative volume; "NaN" -> None
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub day_turnover: Option<f64>, // Cumulative turnover; "NaN" -> None
  pub direction: String, // dxFeed tick direction: "ZERO_DOWN", "UNDEFINED"
  pub is_eth: bool, // Extended trading hours (ETH)
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
  // Parsing logic: Inner array is ["Trade", symbol, event_id, time, sequence, trade_day, exchange_code, price, change, size, day_id, day_volume, day_turnover, direction, is_eth]
  // Handle batches if multiple (similar to Quote, but rare in logs). Use Option for "NaN".
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamProfile 
{
  pub symbol: String,
  #[serde(deserialize_with = "de_f64_flex")]
  pub event_id: f64, // Often 0
  pub description: String,
  pub market_status: String, // e.g., "UNDEFINED" (dxFeed: short sale restriction or similar?)
  pub status: String, // e.g., "ACTIVE" (trading status)
  #[serde(deserialize_with = "de_opt_f64_flex")]
  pub halt_status: Option<f64>, // null in logs; dxFeed halt start time?
  #[serde(deserialize_with = "de_f64_flex")]
  pub open_time: f64, // Open time timestamp
  #[serde(deserialize_with = "de_f64_flex")]
  pub close_time: f64, // Close time timestamp
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub avg_volume: Option<f64>, // "NaN" -> None; avg daily volume
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub avg_volume_30d: Option<f64>, // "NaN" -> None
  #[serde(deserialize_with = "de_f64_flex")]
  pub high_52w: f64,
  #[serde(deserialize_with = "de_f64_flex")]
  pub low_52w: f64,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub div_amount: Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub div_yield: Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub div_freq: Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub div_ex_date: Option<f64>,
  #[serde(deserialize_with = "de_f64_flex")]
  pub beta: f64, // Often 0
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub earnings_per_share: Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub market_cap: Option<f64>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
  // Parsing logic: Inner array is ["Profile", symbol, event_id, description, market_status, status, halt_status, open_time, close_time, avg_volume, avg_volume_30d, high_52w, low_52w, div_amount, div_yield, div_freq, div_ex_date, beta, earnings_per_share, market_cap]
  // Single per message; map "NaN" to None for Options.
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamSummary 
{
  pub symbol: String,
  #[serde(deserialize_with = "de_f64_flex")]
  pub event_id: f64, // Often 0
  #[serde(deserialize_with = "de_f64_flex")]
  pub day_id: f64,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub day_open: Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub day_high: Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub day_low: Option<f64>,
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub day_close: Option<f64>, // Prev close in some contexts
  pub session: String, // e.g., "REGULAR"
  #[serde(deserialize_with = "de_f64_flex")]
  pub prev_day_id: f64,
  #[serde(deserialize_with = "de_f64_flex")]
  pub prev_day_close: f64,
  pub close_type: String, // e.g., "FINAL", "REGULAR"
  #[serde(deserialize_with = "de_opt_f64_nan")]
  pub volume: Option<f64>, // Day volume
  #[serde(deserialize_with = "de_f64_flex")]
  pub open_interest: f64, // Often 0
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
  // Parsing logic: Inner array is ["Summary", symbol, event_id, day_id, day_open, day_high, day_low, day_close, session, prev_day_id, prev_day_close, close_type, volume, open_interest]
  // Single per message; map "NaN" to None.
}

// -----------------------------------------------------------------------------
// Universal Instrument (Equity, Future(?), Crypto, FutureOption)
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Deserialize)]
pub struct Instrument 
{ pub symbol: String,
  #[serde(rename = "instrument-type")]
  pub instrument_type: String,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default, rename = "root-symbol")]
  pub root_symbol: Option<String>,
  #[serde(default, rename = "streamer-symbol")]
  pub streamer_symbol: Option<String>,
  #[serde(default, rename = "listing-date")]
  pub listing_date: Option<String>,
  #[serde(default, rename = "expiration-date")]
  pub expiration_date: Option<String>,
  #[serde(default, rename = "tick-size", deserialize_with = "de_opt_f64_flex")]
  pub tick_size: Option<f64>,
  #[serde(default, deserialize_with = "de_opt_f64_flex")]
  pub multiplier: Option<f64>,
  #[serde(default, rename = "is-closing-only")]
  pub is_closing_only: Option<bool>,
  #[serde(default, rename = "deliverable-type")]
  pub deliverable_type: Option<String>,
  #[serde(default)]
  pub active: Option<bool>,
  #[serde(default, rename = "short-description")]
  pub short_description: Option<String>,
  #[serde(default)]
  pub id: Option<u64>,
  // Futures-specific fields (all optional)
  #[serde(default, rename = "future-product")]
  pub future_product: Option<FutureProduct>,
  #[serde(default, rename = "future-etf-equivalent")]
  pub future_etf_equivalent: Option<FutureEtfEquivalent>,
  #[serde(default, rename = "exchange-data")]
  pub exchange_data: Option<ExchangeData>,
  // Additional fields (all optional)
  #[serde(default, rename = "product-group")]
  pub product_group: Option<String>,
  #[serde(default, rename = "is-tradeable")]
  pub is_tradeable: Option<bool>,
  #[serde(default, rename = "main-fraction", deserialize_with = "de_opt_f64_flex")]
  pub main_fraction: Option<f64>,
  #[serde(default, rename = "next-active-month")]
  pub next_active_month: Option<bool>,
  #[serde(default, rename = "contract-size", deserialize_with = "de_opt_f64_flex")]
  pub contract_size: Option<f64>,
  #[serde(default, rename = "option-tick-sizes")]
  pub option_tick_sizes: Option<Vec<OptionTickSize>>,
  #[serde(default, rename = "spread-tick-sizes")]
  pub spread_tick_sizes: Option<Vec<SpreadTickSize>>,
  #[serde(default, rename = "stops-trading-at")]
  pub stops_trading_at: Option<String>,
  #[serde(default, rename = "expires-at")]
  pub expires_at: Option<String>,
  #[serde(default, rename = "closing-only-date")]
  pub closing_only_date: Option<String>,
  #[serde(default, rename = "last-trade-date")]
  pub last_trade_date: Option<String>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionTickSize 
{
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub threshold: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub value: Option<String>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpreadTickSize 
{
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub symbol: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub value: Option<String>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct FutureProduct 
{
  #[serde(rename = "active-months")]
  pub active_months: Option<Vec<String>>,
  #[serde(rename = "back-month-first-calendar-symbol")]
  pub back_month_first_calendar_symbol: Option<bool>,
  #[serde(rename = "cash-settled")]
  pub cash_settled: Option<bool>,
  #[serde(rename = "clearing-code")]
  pub clearing_code: Option<String>,
  #[serde(rename = "clearing-exchange-code")]
  pub clearing_exchange_code: Option<String>,
  #[serde(rename = "clearport-code")]
  pub clearport_code: Option<String>,
  #[serde(rename = "code")]
  pub code: Option<String>,
  #[serde(rename = "description")]
  pub description: Option<String>,
  #[serde(rename = "display-factor")]
  pub display_factor: Option<String>,
  #[serde(rename = "exchange")]
  pub exchange: Option<String>,
  #[serde(rename = "first-notice")]
  pub first_notice: Option<bool>,
  #[serde(rename = "legacy-code")]
  pub legacy_code: Option<String>,
  #[serde(rename = "legacy-exchange-code")]
  pub legacy_exchange_code: Option<String>,
  #[serde(rename = "listed-months")]
  pub listed_months: Option<Vec<String>>,
  #[serde(rename = "market-sector")]
  pub market_sector: Option<String>,
  #[serde(rename = "notional-multiplier")]
  pub notional_multiplier: Option<String>,
  #[serde(rename = "product-type")]
  pub product_type: Option<String>,
  #[serde(rename = "security-group")]
  pub security_group: Option<String>,
  #[serde(rename = "small-notional")]
  pub small_notional: Option<bool>,
  #[serde(rename = "streamer-exchange-code")]
  pub streamer_exchange_code: Option<String>,
  #[serde(rename = "supported")]
  pub supported: Option<bool>,
  #[serde(rename = "root-symbol")]
  pub root_symbol: Option<String>,
  #[serde(rename = "tick-size")]
  pub tick_size: Option<String>,
  #[serde(rename = "underlying-description")]
  pub underlying_description: Option<String>,
  #[serde(rename = "underlying-identifier")]
  pub underlying_identifier: Option<String>,
  #[serde(default)]
  pub roll: Option<Roll>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FutureEtfEquivalent 
{
  #[serde(rename = "share-quantity")]
  pub share_quantity: Option<u32>,
  pub symbol: Option<String>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeData 
{
  #[serde(rename = "security_id")]
  pub security_id: Option<String>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Roll 
{
  #[serde(rename = "active-count")]
  pub active_count: Option<u32>,
  #[serde(rename = "business-days-offset")]
  pub business_days_offset: Option<u32>,
  #[serde(rename = "cash-settled")]
  pub cash_settled: Option<bool>,
  #[serde(rename = "first-notice")]
  pub first_notice: Option<bool>,
  pub name: Option<String>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstrumentResp 
{ pub data: Instrument,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Deliverable 
{
  pub amount: String,
  #[serde(rename = "deliverable-type")]
  pub deliverable_type: String,
  pub description: String,
  pub id: u64,
  #[serde(rename = "instrument-type")]
  pub instrument_type: String,
  pub percent: String,
  #[serde(rename = "root-symbol")]
  pub root_symbol: String,
  pub symbol: String,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionExpiration 
{
  #[serde(rename = "days-to-expiration")]
  pub days_to_expiration: u32,
  #[serde(rename = "expiration-date")]
  pub expiration_date: String,
  #[serde(rename = "expiration-type")]
  pub expiration_type: String,
  #[serde(rename = "settlement-type")]
  pub settlement_type: String,
  pub strikes: Vec<Strike>,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Strike 
{
  #[serde(rename = "call")]
  pub call: String,
  #[serde(rename = "call-streamer-symbol")]
  pub call_streamer_symbol: String,
  #[serde(rename = "put")]
  pub put: String,
  #[serde(rename = "put-streamer-symbol")]
  pub put_streamer_symbol: String,
  #[serde(rename = "strike-price")]
  pub strike_price: String,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickSize 
{ pub threshold: Option<String>,
  pub value: String,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

// -----------------------------------------------------------------------------
// Option Chains
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OptionChainResponse 
{ pub context: String,
  pub data: OptionChainData,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionChainData 
{ pub items: Vec<OptionChainRoot>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionChainRoot 
{ pub deliverables: Vec<Deliverable>,
  pub expirations: Vec<OptionExpiration>,
  #[serde(rename = "option-chain-type")]
  pub option_chain_type: String,
  #[serde(rename = "root-symbol")]
  pub root_symbol: String,
  #[serde(rename = "shares-per-contract")]
  pub shares_per_contract: u32,
  #[serde(rename = "tick-sizes")]
  pub tick_sizes: Vec<TickSize>,
  #[serde(rename = "underlying-symbol")]
  pub underlying_symbol: String,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

// -----------------------------------------------------------------------------
// Market Metrics (Greeks, Risk, P&L)
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Deserialize)]
pub struct MarketMetric 
{
  #[serde(rename = "metric-type")]
  pub metric_type: String,
  #[serde(deserialize_with = "de_f64_flex")]
  pub value: f64,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default)]
  pub currency: Option<String>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketMetricsData 
{ pub items: Vec<MarketMetric>,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

// -----------------------------------------------------------------------------
// Dry Run / Order types
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest 
{ #[serde(rename = "time-in-force")]
  pub time_in_force: String,
  #[serde(rename = "order-type")]
  pub order_type: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub price: Option<f64>,
  #[serde(rename = "price-effect", skip_serializing_if = "Option::is_none")]
  pub price_effect: Option<String>,
  pub legs: Vec<OrderLeg>,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLeg 
{ pub symbol: String,
  pub action: String,
  pub quantity: f64,
  #[serde(rename = "instrument-type")]
  pub instrument_type: String,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderLegResponse 
{ pub symbol: String,
  pub action: String,
  #[serde(deserialize_with = "de_f64_flex")]
  pub quantity: f64,
  #[serde(rename = "remaining-quantity", deserialize_with = "de_f64_flex")]
  pub remaining_quantity: f64,
  #[serde(rename = "instrument-type")]
  pub instrument_type: String,
  pub fills: Vec<OrderFill>,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderFill 
{ 
  #[serde(deserialize_with = "de_f64_flex")]
  pub quantity: f64,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderResponse 
{
  pub id: u64,
  pub status: String,
  #[serde(rename = "account-number")]
  pub account_number: String,
  #[serde(rename = "time-in-force")]
  pub time_in_force: String,
  #[serde(rename = "order-type")]
  pub order_type: String,
  #[serde(deserialize_with = "de_string_or_number")]
  pub size: String,
  #[serde(rename = "underlying-symbol")]
  pub underlying_symbol: Option<String>,
  #[serde(rename = "underlying-instrument-type")]
  pub underlying_instrument_type: Option<String>,
  #[serde(deserialize_with = "de_opt_f64_flex")]
  pub price: Option<f64>,
  #[serde(rename = "price-effect")]
  #[serde(default, deserialize_with = "de_opt_string_or_number")]
  pub price_effect: Option<String>,
  pub cancellable: bool,
  pub editable: bool,
  pub edited: bool,
  #[serde(rename = "received-at")]
  pub received_at: Option<String>,
  #[serde(rename = "updated-at")]
  pub updated_at: u64,
  pub legs: Vec<OrderLegResponse>,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}


#[derive(Debug, Clone, Deserialize)]
pub struct DryRunData 
{ pub order: OrderResponse,
  pub warnings: Vec<String>,
  #[serde(rename = "buying-power-effect")]
  pub buying_power_effect: BuyingPowerEffect,
  #[serde(rename = "fee-calculation")]
  pub fee_calculation: FeeCalculation,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuyingPowerEffect 
{ 
  #[serde(rename = "change-in-buying-power", deserialize_with = "de_f64_flex")]
  pub change_in_buying_power: f64,
  #[serde(rename = "change-in-buying-power-effect")]
  pub change_in_buying_power_effect: String,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeeCalculation 
{
  #[serde(rename = "total-fees", deserialize_with = "de_f64_flex")]
  pub total_fees: f64,
  #[serde(rename = "total-fees-effect")]
  pub total_fees_effect: String,
  #[serde(flatten)]
  pub extra : HashMap<String,Value>
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarginRequirements 
{
  #[serde(rename = "initial-requirement", deserialize_with = "de_f64_flex")]
  pub initial_requirement: f64,
  #[serde(rename = "maintenance-requirement", deserialize_with = "de_f64_flex")]
  pub maintenance_requirement: f64,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,  
}

// Complex Orders (just aliases)
pub type ComplexOrderRequest = OrderRequest;
pub type ComplexOrderResponse = OrderResponse;
