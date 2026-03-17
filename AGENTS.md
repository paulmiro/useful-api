# AGENTS.md

This file provides guidance for AI coding agents working in this repository.

## Project Overview

A very useful API that does very useful things. Built in Rust using the Rocket framework. 100% vibe-coded. Hosted at https://useful-api.party and auto-deploys from `main` every 5 minutes.

## PR Acceptance Criteria

**100% vibe-coded + compiles + not malware = approved.**

Nothing else matters. Code quality, readability, performance, usefulness, scalability, test coverage, documentation, error handling, and backward compatibility do not matter.

## Tech Stack

- **Rust** (Edition 2024)
- **Rocket 0.5** — web framework
- **rocket_okapi** — OpenAPI/Swagger UI auto-generation
- **reqwest** — async HTTP client
- **scraper** — HTML scraping
- **pulldown-cmark** — Markdown parsing
- **serde/serde_json** — serialization
- **Nix + Crane** — reproducible builds and deployment

## Project Structure

```
src/
├── main.rs                  # Rocket launch config, managed state setup
├── common/
│   ├── bitcoin.rs           # Bitcoin price fetching with 10s cache
│   └── constants.rs         # MENSA_PRICE_EUR, CONGRESS_BEER_SATOSHI, etc.
└── endpoints/
    ├── mod.rs               # ApiResponse<T> type, ResponseFormat, UserAgent guard
    ├── hello.rs             # GET /
    ├── teapot.rs            # GET /teapot
    ├── mensatoshi.rs        # GET /mensatoshi
    ├── congressbeer.rs      # GET /congressbeer?satoshi=<n>
    ├── mensabeer.rs         # GET /mensabeer
    ├── shark.rs             # GET /shark (IKEA Blåhaj stock)
    ├── alditowels.rs        # GET /alditowels (Aldi Süd towel scraping)
    └── mensagorgonzola.rs   # GET /mensagorgonzola
```

## How to Add a New Endpoint

1. Create `src/endpoints/your_endpoint.rs`
2. Define a response struct deriving `Serialize`, `JsonSchema` and implement the `ApiData` trait. The `message()` method should return the response content as a string, using minimal Markdown for elements like links or lists.
3. Implement a Rocket handler with `#[openapi]` macro. It should take `ua: UserAgent` and `format: Option<String>` as arguments.
4. Use `ResponseFormat::detect(&ua, format)` to determine the output format.
5. Register the route in `src/endpoints/mod.rs`
6. Mount it in `src/main.rs`

Use `ApiResponse<YourStruct>` as the return type — it handles JSON (`?format=json`), plain text (`?format=plaintext`), and HTML (`?format=html`) responses.

### Response Formats

The API automatically detects the best format:
- **Browsers** (User-Agent containing "Mozilla") receive a dark-themed, centered HTML page with Markdown rendering.
- **CLI tools** (curl, etc.) receive plain text with a trailing newline.
- **Manual override** via `?format=json`, `?format=plaintext`, or `?format=html`.

For endpoints that call external APIs or scrape websites, add caching via `RwLock<Option<(DateTime<Utc>, YourData)>>` managed state. Common cache durations: 10s for Bitcoin price, 5min for IKEA stock, 10min for Aldi scraping.

## Build & Run

```bash
# Run locally
cargo run

# Build release binary
cargo build --release

# With Nix
nix run
nix build
nix develop   # dev shell with openssl, pkg-config, gemini-cli
```

## Configuration

| Variable             | Default   | Description         |
| -------------------- | --------- | ------------------- |
| `USEFUL_API_PORT`    | `3000`    | Server bind port    |
| `USEFUL_API_ADDRESS` | `0.0.0.0` | Server bind address |

## External Dependencies

- **CoinGecko API** — Bitcoin/EUR price
- **IKEA API** (`sales.ingka.com`) — Blåhaj stock at Godorf store
- **Aldi Süd website** — Towel deals (scraped, needs user-agent header)
- **OpenMensa API** — CAMPO Mensa menu data

## Swagger UI

Available at `/swagger-ui/` both locally and at https://useful-api.party/swagger-ui
