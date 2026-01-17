# Alloy Window Transport Example

This example demonstrates how to integrate Web3 wallet functionality into a Dioxus web application using the `alloy-transport-window` library.

## Features

This example showcases:

- **Wallet Connection**: Connect to browser-based Web3 wallets (MetaMask, Rabby, Coinbase Wallet, etc.)
- **Blockchain Data Fetching**: Retrieve chain ID, current block number, and account balance
- **Message Signing**: Sign arbitrary messages using the connected wallet
- **Reactive UI**: Real-time updates using Dioxus 0.7's signal system
- **Error Handling**: Comprehensive error display and user feedback

## Prerequisites

1. **Dioxus CLI**: Install the Dioxus CLI tool
   ```sh
   curl -sSL http://dioxus.dev/install.sh | sh
   ```

2. **Web3 Wallet**: Install a browser wallet extension:
   - [MetaMask](https://metamask.io/)
   - [Rabby](https://rabby.io/)
   - [Coinbase Wallet](https://www.coinbase.com/wallet)
   - Or any other EIP-1193 compatible wallet

3. **Rust**: Ensure you have Rust installed (MSRV 1.88)

## Running the Example

From the `example/` directory:

```sh
# Start the development server with hot reload
dx serve

# Or explicitly for web platform
dx serve --platform web
```

The application will open in your default browser at `http://localhost:8080`.

## Usage Guide

### 1. Connect Your Wallet

Click the "Connect Wallet" button. Your browser wallet will prompt you to:
- Approve the connection request
- Select which account(s) to connect

### 2. View Wallet Information

Once connected, the app displays:
- **Address**: Your wallet's Ethereum address
- **Chain ID**: The network you're connected to (1 = Ethereum Mainnet, 11155111 = Sepolia, etc.)
- **Current Block**: The latest block number on the blockchain
- **Balance**: Your account balance in wei (1 ETH = 10^18 wei)

Click "Refresh Data" to update this information.

### 3. Sign a Message

- Enter a custom message in the text input
- Click "Sign Message"
- Approve the signing request in your wallet
- The signature will be displayed in hexadecimal format

## Code Structure

### Main Components

The example is contained in [src/main.rs](src/main.rs):

- **Home Component**: Main UI component demonstrating wallet integration
- **State Management**: Uses Dioxus 0.7's `use_signal()` for reactive state
- **Async Operations**: Spawns async tasks for wallet interactions

### Key Implementation Details

**Connecting to Wallet**:
```rust
let signer = WindowSigner::new().await?;
let address = signer.address();
```

**Creating a Provider**:
```rust
let transport = WindowTransport::new()?;
let client = RpcClient::new(transport, false);
let provider = ProviderBuilder::new().connect_client(client);
```

**Fetching Blockchain Data**:
```rust
let chain_id = provider.get_chain_id().await?;
let block_number = provider.get_block_number().await?;
let balance = provider.get_balance(address).await?;
```

**Signing Messages**:
```rust
let signature = signer.sign_message(b"Hello!").await?;
```

## Architecture

The example uses:

- **Dioxus 0.7**: Modern Rust UI framework with signals and reactive state
- **alloy-transport-window**: Browser wallet transport implementation
- **Alloy Provider**: High-level API for blockchain interactions
- **Tailwind CSS**: Utility-first CSS framework for styling

## Troubleshooting

### Wallet Not Detected

Ensure your wallet extension is:
- Installed and enabled
- Unlocked
- Not already connected to another dapp in the same tab

### Connection Fails

- Check browser console for detailed error messages
- Try refreshing the page
- Make sure your wallet is connected to a supported network

### Build Errors

Ensure you're running the web platform:
```sh
dx serve --platform web
```

The `alloy-transport-window` crate only works in browser environments (WASM).

## Next Steps

Extend this example by:

- Adding transaction sending functionality
- Implementing contract interactions
- Adding ENS name resolution
- Supporting multiple networks with chain switching
- Adding wallet disconnection

## Resources

- [Alloy Documentation](https://alloy.rs)
- [Dioxus Documentation](https://dioxuslabs.com)
- [EIP-1193: Ethereum Provider JavaScript API](https://eips.ethereum.org/EIPS/eip-1193)

