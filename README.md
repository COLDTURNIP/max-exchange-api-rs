# maicoin_max - MAX (Maicoin Assets eXchange) Rust SDK

[![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/workflow/status/COLDTURNIP/max-exchange-api-rs/CI/main
[actions]: https://github.com/COLDTURNIP/max-exchange-api-rs/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/maicoin_max.svg
[crates.io]: https://crates.io/crates/maicoin_max

[crates.io](https://crates.io/crates/maicoin_max) |
[docs.rs](https://docs.rs/maicoin_max/latest/maicoin_max/index.html)

[Latest REST API Introduction](https://max.maicoin.com/documents/api_v2) |
[Latest REST API Endpoints](https://max.maicoin.com/documents/api_list/v2) |
[Latest Websocket API Documentation](https://maicoin.github.io/max-websocket-docs/)

## Table of Contents

* [Introduction](#introduction)
  * [Install](#install)
  * [Usage](#usage)
* [Supporting API](#supporting-api)
  * [REST API v2](#rest-api-v2)
  * [Websocket API v2](#websocket-api-v2)
* [Contributing](#contributing)


## Introduction

- Implements MaiCoin MAX v2 API
- Support both REST and websocket API
- Asynchronous programming that not limit to specific runtime

### Install

In `Cargo.toml`:

```toml
[dependencies]
maicoin_max = "2.0"
```

No feature flag required.

- To use REST API, a [http_types](https://docs.rs/http-types) compatible client (e.g [Surf](https://crates.io/crates/surf)) is required.
- To use websocket API, [serde_json](https://docs.serde.rs/serde_json/) is required to serialize/deserialize messages,
  and a websocket client (e.g [Tungstenite](https://crates.io/crates/tungstenite)) is also required.

### Usage

- Get list of exchange supported currencies via public REST API: [examples/get_currencies.rs](examples/get_currencies.rs)
- Private REST API authentication: [examples/rest_auth.rs](examples/rest_auth.rs)
- Receiving tickers from public websocket API: [examples/ws_client.rs](examples/ws_client.rs)
- Websocket authentication and channel filtering: [examples/ws_auth.rs](examples/ws_auth.rs)


## Supporting API

- [v2 REST API Introduction](https://max.maicoin.com/documents/api_v2)
- [v2 REST API Endpoints](https://max.maicoin.com/documents/api_list/v2)
- [v2 Websocket API](https://maicoin.github.io/max-websocket-docs/)

### REST API v2

- private
  - [x] `GET /api/v2/trades/my/of_order`
  - [x] `GET /api/v2/trades/my`
  - [x] `GET /api/v2/withdrawal`
  - [x] `POST /api/v2/withdrawal`
  - [x] `GET /api/v2/members/profile`
  - [x] `GET /api/v2/members/vip_level`
  - [x] `GET /api/v2/members/me`
  - [x] `GET /api/v2/members/accounts/{path_currency}`
  - [x] `GET /api/v2/members/accounts`
  - [x] `GET /api/v2/deposits`
  - [x] `GET /api/v2/deposit`
  - [ ] `GET /api/v2/deposit_address` (officially deprecated)
  - [x] `GET /api/v2/deposit_addresses`
  - [x] `POST /api/v2/deposit_addresses`
  - [x] `GET /api/v2/withdraw_addresses`
  - [x] `GET /api/v2/internal_transfers`
  - [x] `GET /api/v2/internal_transfer`
  - [x] `GET /api/v2/rewards/{path_reward_type}`
  - [x] `GET /api/v2/rewards`
  - [x] `GET /api/v2/yields`
  - [x] `GET /api/v2/max_rewards/yesterday`
  - [x] `POST /api/v2/orders/clear`
  - [x] `GET /api/v2/orders`
  - [x] `POST /api/v2/orders`
  - [ ] `POST /api/v2/orders/multi/onebyone`
  - [x] `POST /api/v2/order/delete`
  - [x] `GET /api/v2/order`
  - [x] `GET /api/v2/withdrawals`
- public
  - [x] `GET /api/v2/vip_levels`
  - [x] `GET /api/v2/vip_levels/{level}`
  - [x] `GET /api/v2/currencies`
  - [x] `GET /api/v2/k`
  - [x] `GET /api/v2/depth`
  - [x] `GET /api/v2/trades`
  - [x] `GET /api/v2/markets`
  - [x] `GET /api/v2/summary`
  - [x] `GET /api/v2/tickers/{path_market}`
  - [x] `GET /api/v2/tickers`
  - [x] `GET /api/v2/timestamp`
  - [x] `GET /api/v2/vip_levels`
  - [x] `GET /api/v2/vip_levels/{level}`
  - [x] `GET /api/v2/withdrawal/constraint`

### Websocket API v2

- Public Channels
  - [x] Subscribe
  - [x] UnSubscribe
  - [x] Orderbook feeds
  - [x] Trade feeds
  - [x] Ticker feeds
- Private Channels
  - [x] Authentication & Subscribe
  - [x] Order feeds
  - [x] Trade feeds
  - [x] Account feeds


## Contributing

Patches and pull requests are welcome. For major features or breaking changes,
please open a ticket or start a discussion first so we can discuss what you
would like to do.
