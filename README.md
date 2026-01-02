# TTRS: Tastytrade Rust SDK

A comprehensive, high-performance Rust client library for the Tastytrade API. Built from scratch with async-first design, robust connection management, and targeting complete API coverage.

**Current Status**: 0.8.1 (Integration Testing) → 0.9.0 (Stabilization) → 1.0.0 (Production Ready)

Change Status

0.8.1: fixed a bug that prevented proper ticker subscriptions when using options streamer symbols

0.8.2: TODO: will check rate settings for streaming

---

## What is TTRS?

TTRS is a native Rust implementation of the Tastytrade trading API, replacing the need for Python wrappers or unofficial SDKs. It's designed for traders and developers who need:

- **Fast, non-blocking async I/O** via tokio
- **Reliable WebSocket streaming** for real-time quotes and account updates
- **Resilient connection management** with automatic token refresh and reconnection
- **Type-safe API bindings** with serde deserialization
- **Production-grade error handling** and observability
---

## History

Originally, Tastytrade didn't offer a native Rust SDK. I previously built **[ttai](https://github.com/yourname/ttai)** — a Rust wrapper around the unofficial Python API. When Tastytrade deprecated non-OAuth connections, the effort of maintaining ttai became equivalent to maintaining a full API implementation anyway.

**TTRS** is that full implementation: purpose-built for Rust, following Tastytrade's official OAuth2 API.

---

## Design Principles

### 1. **Full API Coverage**
Every public Tastytrade endpoint is supported:
- ✅ Account management (info, trading status, positions, balances)
- ✅ Order management (place, cancel, replace, dry-run, complex orders)
- ✅ Real-time quotes (equity, futures, crypto, options)
- ✅ Market data (option chains, instruments, market metrics)
- ✅ Account streaming (real-time position, order, balance updates)

### 2. **Lock-Free Async-First Architecture**
Built on **tokio** with **mpsc channels**:
- No mutexes on the hot path
- Actor-pattern message passing (Thand = user-facing hand-off, Tfoot = core running process)
- Concurrent REST + WebSocket streams
- Clean separation of concerns

### 3. **Robust Connection Management**
State machines handle the messy real-world:
- **Token refresh**: Exponential backoff on OAuth failures
- **WebSocket resilience**: Auto-reconnect on disconnect
- **Subscription tracking**: Routes incoming data to correct user channels
- **Error propagation**: All failures flow to a single error channel

### 4. **Unified Data Models (QoL)**
- **Unified instrument lookup**: One function for equity/future/crypto/option symbols
- **Ticker data enum**: Quote | Trade | Summary | Profile in a single `StreamData` enum type
- **No manual transformations**: Unlike some SDKs, positions are not synthetically negated or mangled

### 5. **API Observability**
- **Comprehensive logging**: DEBUG for function entry/exit with GOOD/FAIL indicators, TRACE for data flow, WARN for unexpected paths.
- **Unknown field detection**: Serialization catches API changes early
- **Full test coverage**: 15+ integration tests spanning REST endpoints + WebSocket streams

### 6. **Self-Documenting Code**
- Consistent naming (`tell_tastytrade_to_*` for public methods)
- Idealized Formatted code with strategic comments
- Trace & other logs generally replace the need for comments.
- Type system guides implementation
---

## Quick Start

### Prerequisites

- Rust 1.70+
- Tokio runtime
- Valid Tastytrade OAuth credentials (see below)

### Installation

Add to `Cargo.toml`:
```toml
[dependencies]
ttrs = { git = "https://github.com/YaSteven/ttrs", rev = "main" }
tokio = { version = "1.0", features = ["full"] }
```

### Get Credentials

1. Go to [Tastytrade Developer](https://developer.tastyworks.com/)
2. Create an OAuth application
3. Save your `client_id`, `client_secret`, and `refresh_token`
4. Create `tests/test-creds.json`:
```json
{
  "client_id": "your_client_id",
  "client_secret": "your_client_secret",
  "refresh_token": "your_refresh_token",
  "account": "your_account_number"
}
```

### Basic Example

```rust
use ttrs::bot::{make_core_api, fn_run_core, CoreConfig, ConnectionInfo};
use ttrs::dat::StreamData;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> 
{ let conn_info = ConnectionInfo 
  { base_url: "https://api.tastyworks.com".to_string()
  , oauth: 
    ( format!("your_client_id"),
    , format!("your_client_secret"),
    , format!("your_refresh_token"),
    )
  };

  // Create core API (hand for sending requests, foot for receiving)
  let (thand, tfoot) = make_core_api(10);
    
  // Spawn core event loop
  let (error_tx, mut error_rx) = mpsc::unbounded_channel();
  let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
  let config = CoreConfig { error_tx, shutdown_rx };
    
  let core_handle = tokio::spawn
  ( fn_run_core(tfoot, conn_info, config)
  );

  // Give core time to initialize
  tokio::time::sleep(std::time::Duration::from_millis(500)).await;

  // Get account info
  let accounts = thand.tell_tastytrade_to_get_accounts_info().await?;
  println!("Accounts: {:#?}", accounts);

  let actnum: String = accounts.items.get(0).cloned()
    .unwrap_or_else(|| "ACC12345".to_string());

  // Get positions
  let positions = thand.tell_tastytrade_to_get_positions("ACC12345".to_string()).await?;
  for pos in positions 
  { println!("{}: {} shares @ ${}", pos.symbol, pos.quantity, pos.close_price);
  }

  // Subscribe to live quotes
  let (quote_tx, mut quote_rx) = mpsc::channel(100);
  
  thand.tell_tastytrade_to_get_ticker_stream
  ( format!("SPY")
  , quote_tx
  ).await;

  // Receive quotes
  while let Some(data) = quote_rx.recv().await 
  { match data 
    { StreamData::Quote(q) => 
      { println!
        ( "{}: bid={:?} ask={:?}"
        , q.symbol, q.bid_price, q.ask_price
        );
      },
      StreamData::Trade(t) => 
      { println!
        ( "{}: price={} size={:?}", t.symbol, t.price, t.size
        );
      },
      _ => {}
    }
  }

  // Shutdown gracefully
  shutdown_tx.send(format!("shutdown")).await?;
  core_handle.await?;

  Ok(())
}
```

Keep an eye out on [main.rs](./src/main.rs) or tests.rs for complete examples in the future
, including order placement, dry runs, and complex orders.

---

## API Overview

### REST Endpoints (Async Functions)

Thand structure allows you to either directly call send on the mpsc 
Transmitters while passing the arbitary Result callback receiever,
or use a simplified helper function that awaits on the callbacks if
you prefer a slightly less performant, sync-like design:

**Account Management**
```rust
thand.tell_tastytrade_to_get_accounts_info() → Result<AccountsData>
thand.tell_tastytrade_to_get_trading_status(acct: String) → Result<TradingStatus>
thand.tell_tastytrade_to_get_positions(acct: String) → Result<Vec<Position>>
thand.tell_tastytrade_to_get_balances(acct: String) → Result<Balance>
thand.tell_tastytrade_to_get_transactions(acct: String) → Result<Vec<Transaction>>
```

**Orders**
```rust
thand.tell_tastytrade_to_place_order(acct: String, order: OrderRequest) → Result<OrderResponse>
thand.tell_tastytrade_to_get_live_orders(acct: String) → Result<Vec<OrderResponse>>
thand.tell_tastytrade_to_get_order_by_id(acct: String, order_id: u64) → Result<OrderResponse>
thand.tell_tastytrade_to_cancel_order(acct: String, order_id: u64) → Result<OrderResponse>
thand.tell_tastytrade_to_dry_run_order(acct: String, order: OrderRequest) → Result<DryRunResponse>
```

**Market Data**
```rust
thand.tell_tastytrade_to_get_option_chain(symbol: String) → Result<OptionChainData>
thand.tell_tastytrade_to_get_instrument(symbol: String) → Result<Instrument>
thand.tell_tastytrade_to_get_bulk_quotes(symbols: Vec<String>) → Result<Vec<StreamQuote>>
thand.tell_tastytrade_to_get_market_metrics(acct: String) → Result<Vec<MarketMetric>>
```

### WebSocket Streams (Fire-and-Forget)

```rust
// Ticker stream: Quote | Trade | Summary | Profile
thand.tell_tastytrade_to_get_ticker_stream(symbol: String, tx: mpsc::Sender<StreamData>).await

// Account stream: position | order | account-balance updates
thand.tell_tasytrade_to_get_accounts_stream(acct: String, tx: mpsc::Sender<Value>).await
```
---

## Testing

### Run All Tests

```bash
cargo test -- --test-threads=1 --nocapture
```

**Requires**: Valid `tests/test-creds.json` with real Tastytrade credentials and account.

### Test Coverage

- ✅ OAuth token refresh + resilience
- ✅ REST endpoints (accounts, positions, balances, transactions, orders)
- ✅ Ticker streaming (equity, crypto, futures)
- ✅ Account streaming (real-time updates)
- ❌ Order lifecycle (place, dry-run, cancel)
- ✅ Option chains + market data
- ✅ Log sanitization (removes tokens/credentials from test logs)
---

## Architecture

### Actor Pattern

```
User Code
   ↓
Thand (sender side)
   ↓ mpsc channels
fn_run_core (tokio::spawn)
   ├─ Handles user requests via Tfoot (receiver side)
   ├─ Handles REST requests via api_req! macro
   ├─ Dispatches ticker subscriptions
   ├─ Manages account stream connections
   └─ Routes results back via oneshot channels
   ↓
WebSocket Loops
   ├─ ticker_loop_resilient (dxFeed quotes)
   ├─ account_stream_loop (real-time updates)
   └─ Auto-reconnect on failure
```

### Key Files

- **bot.rs** (<1500 lines): Core event loop, API handlers, WebSocket state machines
- **dat.rs** (<1000 lines): Type definitions for all API objects
- **ser.rs** (<500 lines):  Private types and Custom deserializers (handles "NaN", flexible float parsing)
- **tests.rs** (1800 lines): Comprehensive integration test suite

---

## Performance Characteristics

- TODO
- **REST latency**: ... 
- **WebSocket latency**: ...
- **Memory**: ... 
- **Concurrency**: ...

---

## Error Handling

All errors flow to a single `error_tx` channel:

```rust
pub enum CoreError 
{
  OAuth(String),           // Token refresh failed
  WebSocket(String),       // Connection/parsing error
  ParseFailure(String),    // JSON deserialization error
  ParseWarning(String),    // Unexpected field (API changed?)
  Unrecoverable(String),   // Task panic or fatal condition
}

// Spawn core with error channel
let (error_tx, mut error_rx) = mpsc::unbounded_channel();
// ...
while let Some(err) = error_rx.recv().await 
{ eprintln!("TTRS error: {:?}", err);
}
```

---

## Logging

Enable detailed logging for debugging:

```bash
RUST_LOG=ttrs=debug cargo run # verbose function Entry/Exit
RUST_LOG=ttrs=trace cargo run # Very verbose, data-level logging
```
Key log patterns (subject to changes):
- `ENTRY! function_name()` — Function entry point
- `TTRS_TL:` — Ticker loop messages
- `WARN` — Unexpected data or API changes
---

## Roadmap

### 0.8.0 (Past)
- ✅ Full OAuth2 + REST API coverage
- ✅ WebSocket streaming (quotes + account updates)
- ✅ Comprehensive test suite
- ✅ Production integration testing

### 0.8.x (Next)
- 📋 Fix any issues not caught by current tests (e.g., I may have bugs with Futures at the moment)
- 📋 Testing Account Streaming & Orders on live system

### 0.9.0 (After Next)
- 📋 Mock testing infrastructure
- 📋 Tighter token refresh logic
- 📋 Expanded README + examples
- 📋 Renaming the ugly stuff (e.g., Thand & Tfoot, tell_tastytrade_...)

### 1.0.0 (Final)
- 📋 Stabilized public API
- 📋 crates.io release
- 📋 Performance benchmarks
- 📋 Production support guarantee

---

## Contributing

Found a bug? Missing endpoint? Have a suggestion?

1. Check [issues](https://github.com/yourname/ttrs/issues)
2. Submit a PR with tests

---

## Support

TTRS is developed independently. If you find this useful, consider supporting ongoing development:

- **Bitcoin**: `1KujcRGWjwJRzVMTx1bbdzBB79bnVFm9dP` (add your BTC address)
- **GitHub Sponsors**: [Enable sponsorships](https://github.com/sponsors/yourname)
- **Issues/PRs**: Community contributions are welcome!

---

## License

???

---

## Disclaimer

TTRS is an unofficial SDK. Use at your own risk. Not affiliated with Tastytrade. 

Always test thoroughly before trading with real money. Unless you're me and have no money.

---

## Credits

- **Author**: Steven E. Elliott
- **Built with**: Tokio, Serde, Reqwest, Tokio-Tungstenite
- **Inspired by**: Official Tastytrade API docs, community needs

---

## FAQ

### Q: Why Rust?
It's the best systems computer programming language.

### Q: Can I use this in production?
After 0.9.0 stabilization and a week of integration testing, yes.
Currently, I consider 0.8.0 "release candidate" stage.

### Q: Does it support paper trading?
Yes — use Tastytrade's cert environment by setting `base_url` to `https://api.cert.tastyworks.com`.
But note, the paper trading has had weird rules in the past and present about how it processes orders, 

### Q: What about options trading?
Full support: get chains, dry-run spreads, place complex multi-leg orders.

### Q: How do I handle reconnects?
Automatic. WebSocket streams reconnect on failure. Check the `error_tx` channel for notifications.

### Q: Can I trade multiple accounts?
Yes. Spawn multiple `fn_run_core` instances with different credentials, or use the same core and route by account number.

---

**Happy ~~gambling~~ trading! 📈**