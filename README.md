# O'Mara Investment System

A robust system for collecting and storing market data from cryptocurrency exchanges, built in Rust.

## Overview

This project consists of two main components:

1. **quantlib**: A Rust library providing core functionality for market data collection and processing
   - WebSocket client for Kraken's v2 API
   - Order book management
   - Market data stream handling

2. **data-collection**: A service that uses quantlib to collect and store market data
   - Configurable subscriptions for multiple symbols
   - Raw JSON data storage for maximum flexibility in the future
   - Real-time statistics tracking

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/declanomara/Investments
cd Investments
```

2. Build the project:
```bash
cargo build --release
```

### Configuration

Create a `config.toml` file with your desired settings:

```toml
subscriptions = { 
    book = ["BTC/USD", "ETH/USD"],
    ticker = ["BTC/USD", "ETH/USD"]
}

data_dir = "data"
monitor_socket = "/tmp/collector.sock"
```

### Running the Collector

```bash
cargo run --release --bin data-collection
```

## Project Structure

```
.
├── quantlib/           # Core library
│   ├── src/
│   │   ├── kraken/    # Kraken API implementation
│   │   ├── objects.rs # Data structures
│   │   └── util.rs    # Utility functions
│   └── Cargo.toml
├── data-collection/   # Data collection service
│   ├── src/
│   │   ├── main.rs    # Main service entry point
│   │   ├── config.rs  # Configuration handling
│   │   └── stats.rs   # Statistics tracking
│   └── Cargo.toml
├── data/             # Data storage directory
└── data_collection_config.toml
```

## Development Roadmap

### Short Term
- [ ] Implement graceful shutdown mechanism
- [ ] Add Unix domain socket for monitoring and control
- [ ] Add data validation and error recovery
- [ ] Improve monitoring capabilities

### Long Term
- [ ] Implement data processing pipeline
- [ ] Add support for more exchanges
- [ ] Develop analysis tools
- [ ] Create visualization dashboard

## License

This project is proprietary software. All rights reserved. See LICENSE file for details.

Unauthorized copying, modification, distribution, public display, or public performance of this software is strictly prohibited.
