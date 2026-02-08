// src/lib.rs
// ==================================================================
// ttrs/src/lib.rs - Public API with flat re-exports (Nov 2025)
// ==================================================================
pub mod dat;
pub mod bot;
pub mod ser;

// ------------------------------------------------------------------
// Public re-exports – flat, ergonomic API
// Users can do: use ttrs::{Thand, StreamData, Instrument, ...};
// ------------------------------------------------------------------

// Core bot & config
pub use bot::
{   CoreConfig
  , CoreError
  , fn_run_core
  , make_core_api
  , Thand
  , Tfoot
};

// Data types – everything a user might want to touch
pub use dat::
{ // ===== AUTH / CONNECTION =====
    ConnectionInfo
  , OauthInfo
  // ===== STREAMING DATA TYPES (NEW) ===== 
  , StreamData
  , StreamQuote
  , StreamTrade
  , StreamSummary
  , StreamProfile
  // ===== CORE ACCOUNT & POSITION TYPES ===== 
  , AccountInfo
  , Balance
  , Position
  , TradingStatus
  , Transaction
  // ===== QUOTES & MARKET DATA ===== 
  , MarketMetric
  , MarketMetricsData
  , Instrument
  , InstrumentResp
  // ===== OPTION CHAINS ===== 
  , OptionChainData
  , OptionChainRoot
  , OptionExpiration
  , Strike
  , Deliverable
  // ===== ORDERS & DRY-RUN ===== 
  , OrderRequest
  , ComplexOrderRequest
  , OrderResponse
  , ComplexOrderResponse
  , OrderLeg
  , OrderLegResponse
  , OrderFill
  , DryRunData
  , BuyingPowerEffect
  , FeeCalculation
  , MarginRequirements
  // ==== MISC. COMPONENETS
  , TickSize
  , OptionTickSize
  , SpreadTickSize

};

// Keep for the tests.rs  
#[cfg(test)]
pub mod tests;

// ------------------------------------------------------------------
// Optional: Type aliases for ergonomic user code
// ------------------------------------------------------------------

/// Convenience type for streaming channel: `Sender<StreamData>`
pub type StreamTx = tokio::sync::mpsc::Sender<dat::StreamData>;

/// Convenience type for streaming receiver: `Receiver<StreamData>`
pub type StreamRx = tokio::sync::mpsc::Receiver<dat::StreamData>;

// ------------------------------------------------------------------
// Optional: Re-export common Tokio/utility types for users
// ------------------------------------------------------------------
pub use tokio::sync::mpsc;
