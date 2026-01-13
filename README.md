# A very useful API that does very useful things (100% vibe-coded)

## 🚀 Overview

This is a very useful API that does very useful things. Built with Rust and Axum, it provides endpoints for calculating satoshi-related conversions with a focus on real-world applications like Mensa meals and Congressbeers.

## 📡 Endpoints

### 🏠 Root Endpoint

**GET** `/`

Returns a simple greeting.

**Response:**
```
Hello, World!
```

---

### 🍲 Mensa Satoshi Endpoint

**GET** `/mensatoshi`

Calculates how many satoshi a Mensa meal costs based on current Bitcoin prices. The price is cached for 10 seconds to avoid rate limiting.

**Response:**
```
Der Mensa-Eintopf kostet aktuell 1234 Satoshi.
```

**Features:**
- 💰 Fetches real-time Bitcoin prices from CoinGecko API
- ⚡ Caches results for 10 seconds to optimize performance
- 🍽️ Uses a fixed Mensa price of €1.20 (TODO: fetch dynamically)

---

### 🍺 Congressbeer Endpoint

**GET** `/congressbeer?satoshi=<amount>`

Calculates how many Congressbeers a given amount of satoshi could have been, where **1 Congressbeer = 69 satoshi**. The result is rounded down to the nearest integer (floor division).

**Query Parameters:**
- `satoshi` (required): The amount of satoshi to convert to Congressbeers (must be a number)

**Examples:**

Request: `GET /congressbeer?satoshi=690`
```
690 Satoshi entspricht 10 Congressbeers.
```

Request: `GET /congressbeer?satoshi=200`
```
200 Satoshi entspricht 2 Congressbeers.
```
*(200 ÷ 69 = 2.89... → floors to 2)*

Request: `GET /congressbeer?satoshi=138`
```
138 Satoshi entspricht 2 Congressbeers.
```
*(138 ÷ 69 = 2.0 → exactly 2)*

Request: `GET /congressbeer?satoshi=50`
```
50 Satoshi entspricht 0 Congressbeers.
```
*(50 ÷ 69 = 0.72... → floors to 0)*

**Features:**
- 🍺 Calculates Congressbeers based on the legendary 69 satoshi per beer rate
- 🔢 Returns integer values only (rounded down)
- 📊 Simple and efficient calculation

---

## 🛠️ Technical Details

- **Framework:** Axum (Rust)
- **Runtime:** Tokio
- **Port:** 19190
- **Host:** 0.0.0.0 (listens on all interfaces)

## 🚦 Running the Server

```bash
cargo run
```

The server will start on `http://0.0.0.0:19190`

## 📝 Notes

- The API uses lazy static caching for Bitcoin price data
- CoinGecko API rate limits are handled gracefully
- All endpoints return plain text responses