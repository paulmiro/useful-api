# A very useful API that does very useful things (100% vibe-coded)

# [useful-api.party](https://useful-api.party)

## 🚀 Overview

This is a very useful API that does very useful things. Built with Rust and Rocket, it provides endpoints for calculating satoshi-related conversions with a focus on real-world applications like Mensa meals and Congress-Beers, as well as checking stock for our favorite sharks.

Interactive API documentation is available via Swagger UI at [`/swagger-ui/`](https://useful-api.party/swagger-ui).

The server at [useful-api.party](https://useful-api.party) automatically updates from the `main` branch every 5 minutes.

## 🎉 Contributions

**✨ *We want as many Pull Requests as possible!* ✨**

**EVERY** PR will be merged, *as long as it adheres to these guidelines (in order of importance)*:

- ✅ It is 100% vibe-coded.
- 🛠️ It compiles.
- 🛡️ It is not malware.

Things that do not matter (in no particular order):

- 🚫 Code quality *(as long as it compiles)*
- 📚 Code readability *(no human will ever have to read it)*
- 💨 Performance *(it's written in rust, so bad performance is impossible)*
- 🤔 Usefulness to the user *(who could even decide what counts as "useful"?)*
- 🚀 Scalability *(it runs on my machine, so it will probably scale to a billion users)*
- 🧪 Test coverage *(untested code is the purest code)*
- 📖 Documentation *(the code is self-documenting, if you can read it)*
- 🐛 Error handling *(errors are just unexpected features)*
- ⏪ Backward compatibility *(if it breaks, they shouldn't have used it)*
- 🔮 Future-proofing *(tomorrow's problems are for tomorrow's agents)*

## 📡 Endpoints

### 🏠 [Root Endpoint](https://useful-api.party/)

**GET** `/`

Returns a simple greeting.

---

### 🫖 [Teapot Endpoint](https://useful-api.party/teapot)

**GET** `/teapot`

Returns a `418 I'm a teapot` status code.

---

### 🍲 [Mensa Satoshi Endpoint](https://useful-api.party/mensatoshi)

**GET** `/mensatoshi`

Calculates how many satoshi a Mensa meal costs based on current Bitcoin prices. The price is cached for 10 seconds to avoid rate limiting.

**Features:**
- 💰 Fetches real-time Bitcoin prices from CoinGecko API
- ⚡ Caches results for 10 seconds to optimize performance
- 🍽️ Uses a fixed Mensa price of €1.20 (TODO: fetch dynamically)

---

### 🦈 [Shark Endpoint](https://useful-api.party/shark)

**GET** `/shark`

Checks the stock of "beeghaj" (large Blåhaj), "smolhaj" (small Blåhaj), and "whale" (Blåvingad Whale) at IKEA Godorf. The stock data is cached for 5 minutes.

**Features:**
- 🦈 Fetches real-time stock data from IKEA API
- ⚡ Caches results for 5 minutes to optimize performance
- 📦 Tracks multiple plushie varieties from the IKEA ocean collection

---

### 🍺 [Congress-Beer Endpoint](https://useful-api.party/congressbeer)

**GET** `/congressbeer?satoshi=<amount>`

Calculates how many Congress-Beers a given amount of satoshi could have been, where **1 Congress-Beer = 69 satoshi**. The result is rounded down to the nearest integer.



**Features:**
- 🍺 Calculates Congress-Beers based on the legendary 69 satoshi per beer rate
- 🔢 Returns integer values only (rounded down)
- 📊 Simple and efficient calculation

---

### 🥘🍺 [Mensa-Beer Endpoint](https://useful-api.party/mensabeer)

**GET** `/mensabeer`

Calculates how many Congress-Beers you could buy for the price of one Mensa meal, based on the current Bitcoin exchange rate.

**Features:**
- 🔄 Combines real-time Bitcoin data with Mensa and Congress metrics.
- 🍺 Shows the purchasing power of a Mensa meal in the most important currency: Congress-Beer.
- 🍽️ Uses the same fixed Mensa price (€1.20) as the Mensa Satoshi endpoint.

---

## ⚙️ Configuration

The application can be configured using environment variables:

| Variable             | Description                     | Default   |
| -------------------- | ------------------------------- | --------- |
| `USEFUL_API_PORT`    | The port the server listens on  | `3000`    |
| `USEFUL_API_ADDRESS` | The address the server binds to | `0.0.0.0` |

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
- **API Documentation:** [OpenAPI / Swagger UI](https://swagger.io/) via [rocket_okapi](https://github.com/GREsau/okapi)
- **Schema Generation:** [schemars](https://github.com/GREsau/schemars)
- **HTTP Client:** [reqwest](https://docs.rs/reqwest/) (v0.13)
- **Serialization:** [serde](https://serde.rs/) & [serde_json](https://docs.rs/serde_json/)
- **Runtime:** Tokio
- **Build System:** Cargo & Nix (via [Crane](https://github.com/ipetkov/crane))

## 📝 Notes

- Interactive API documentation is served at [`/swagger-ui/`](https://useful-api.party/swagger-ui/).
- The API uses Rocket's managed state and `RwLock` for caching data.
- CoinGecko API rate limits are handled gracefully.
- Endpoints support both plain text and JSON responses (via `?format=json`).
- The server's bind address is configurable via the `USEFUL_API_ADDRESS` environment variable. Defaults to `0.0.0.0`.
- The server's port is configurable via the `USEFUL_API_PORT` environment variable. Defaults to `3000`.

## ⚠️ Disclaimer

This API involves cryptocurrency data which is highly volatile. The conversions provided are for entertainment purposes only and should not be used for financial decisions.
