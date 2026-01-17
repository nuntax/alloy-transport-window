# Example App

A Dioxus web application demonstrating `alloy-transport-window` usage with browser wallets.

## Structure

```
example/
├── src/
│   ├── main.rs              # App entry point and routing
│   └── examples/
│       ├── send_transaction.rs  # Send ETH transactions
│       ├── block_explorer.rs    # Query blockchain data
│       └── aave_pool.rs         # Call smart contracts
├── assets/                  # Static assets
├── sol_interface/           # Contract ABIs
└── Cargo.toml
```

## Running

Install Dioxus CLI:
```sh
curl -sSL http://dioxus.dev/install.sh | sh
```

Run the app:
```sh
dx serve
```

Opens at `http://localhost:8080`

## Requirements

- A browser wallet extension (MetaMask, Rabby, etc.)
- Rust 1.88+
