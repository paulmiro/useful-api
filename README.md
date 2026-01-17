# A very useful API that does very useful things (100% vibe-coded)

## 🚀 Overview

This is a very useful API that does very useful things. Built with Rust and Rocket, it provides endpoints for calculating satoshi-related conversions with a focus on real-world applications like Mensa meals and Congress-Beers, as well as checking stock for our favorite sharks.

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

**Query Parameters:**
- `format` (optional): Set to `json` to get a JSON response.

**Response (Plain Text):**
```
Der Mensa-Eintopf kostet aktuell 1234 Satoshi.
```

**Response (JSON):**
```json
{
  "satoshi": 1234.0,
  "message": "Der Mensa-Eintopf kostet aktuell 1234 Satoshi."
}
```

**Features:**
- 💰 Fetches real-time Bitcoin prices from CoinGecko API
- ⚡ Caches results for 10 seconds to optimize performance
- 🍽️ Uses a fixed Mensa price of €1.20 (TODO: fetch dynamically)

---

### 🦈 Shark Endpoint

**GET** `/shark`

Checks the stock of "beeghaj" (large Blåhaj), "smolhaj" (small Blåhaj), and "whale" (Blåvingad Whale) at IKEA Godorf. The stock data is cached for 5 minutes.

**Query Parameters:**
- `format` (optional): Set to `json` to get a JSON response.

**Response (Plain Text):**
```
Der IKEA Godorf hat aktuell 42 beeghajs, 69 smolhajs und 7 whales auf Lager :D
```

**Response (JSON):**
```json
{
  "beeghaj": 42,
  "smolhaj": 69,
  "whale": 7,
  "message": "Der IKEA Godorf hat aktuell 42 beeghajs, 69 smolhajs und 7 whales auf Lager :D"
}
```

**Features:**
- 🦈 Fetches real-time stock data from IKEA API
- ⚡ Caches results for 5 minutes to optimize performance
- 📦 Tracks multiple plushie varieties from the IKEA ocean collection

---

### 🍺 Congress-Beer Endpoint

**GET** `/congressbeer?satoshi=<amount>`

Calculates how many Congress-Beers a given amount of satoshi could have been, where **1 Congress-Beer = 69 satoshi**. The result is rounded down to the nearest integer (floor division).

**Query Parameters:**
- `satoshi` (required): The amount of satoshi to convert to Congress-Beers (must be a number)

**Examples:**

Request: `GET /congressbeer?satoshi=690`
```
690 Satoshi entspricht 10 Bier auf dem Congress.
```

Request: `GET /congressbeer?satoshi=200`
```
200 Satoshi entspricht 2 Bier auf dem Congress.
```
*(200 ÷ 69 = 2.89... → floors to 2)*

Request: `GET /congressbeer?satoshi=50`
```
50 Satoshi entspricht 0 Bier auf dem Congress.
```
*(50 ÷ 69 = 0.72... → floors to 0)*

Request: `GET /congressbeer`
```
69 Satoshi entspricht 1 Bier auf dem Congress.
```
*defaults to the price of one beer*


**Features:**
- 🍺 Calculates Congress-Beers based on the legendary 69 satoshi per beer rate
- 🔢 Returns integer values only (rounded down)
- 📊 Simple and efficient calculation

---

### 🥘🍺 Mensa-Beer Endpoint

**GET** `/mensabeer`

Calculates how many Congress-Beers you could buy for the price of one Mensa meal, based on the current Bitcoin exchange rate.

**Query Parameters:**
- `format` (optional): Set to `json` to get a JSON response.

**Response (Plain Text):**
```
Für den Preis eines Mensa-Eintopfs bekommt man aktuell 123 Bier auf dem Congress.
```

**Response (JSON):**
```json
{
  "beers": 123.0,
  "message": "Für den Preis eines Mensa-Eintopfs bekommt man aktuell 123 Bier auf dem Congress."
}
```

**Features:**
- 🔄 Combines real-time Bitcoin data with Mensa and Congress metrics.
- 🍺 Shows the purchasing power of a Mensa meal in the most important currency: Congress-Beer.
- 🍽️ Uses the same fixed Mensa price (€1.20) as the Mensa Satoshi endpoint.

---

## ⚙️ Configuration

The application can be configured using environment variables:

| Variable          | Description                    | Default |
| ----------------- | ------------------------------ | ------- |
| `USEFUL_API_PORT` | The port the server listens on | `3000`  |

---

## 🛠️ Development

### Prerequisites

- **Rust** (latest stable) & **Cargo**
- OR: **Nix** (with flake support enabled)

### Running locally

**Using Cargo:**
```bash
cargo run
```

**Using Nix:**
```bash
nix run
```

### Development Environment

If you are using Nix, you can enter a reproducible development shell with all dependencies (including `pkg-config`, `openssl`, and `gemini-cli`) pre-configured:

```bash
nix develop
```

---

## 📦 Building

**Using Cargo:**
```bash
cargo build --release
```
The binary will be located at `target/release/useful-api`.

**Using Nix:**
```bash
nix build
```
The binary will be available in `./result/bin/useful-api`.

---

## 🧩 Technical Stack

- **Framework:** [Rocket](https://rocket.rs/) (v0.5.1)
- **HTTP Client:** [reqwest](https://docs.rs/reqwest/) (v0.13)
- **Serialization:** [serde](https://serde.rs/) & [serde_json](https://docs.rs/serde_json/)
- **Runtime:** Tokio
- **Build System:** Cargo & Nix (via [Crane](https://github.com/ipetkov/crane))

## 📝 Notes

- The API uses Rocket's managed state and `RwLock` for caching data.
- CoinGecko API rate limits are handled gracefully.
- Endpoints support both plain text and JSON responses (via `?format=json`).
- `0.0.0.0` is used as the bind address, making the server accessible externally.

## ⚠️ Disclaimer

This API involves cryptocurrency data which is highly volatile. The conversions provided are for entertainment purposes only and should not be used for financial decisions.
