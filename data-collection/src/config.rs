use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use clap::Parser;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // Map of channel to symbols for that channel
    pub subscriptions: HashMap<String, Vec<String>>,
    pub data_dir: PathBuf,
    pub monitor_socket: PathBuf,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Override subscriptions from config file
    /// Format: channel:symbol1,symbol2,...
    /// Example: book:BTC/USD,ETH/USD ticker:BTC/USD
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    pub subscriptions: Option<Vec<String>>,

    /// Override data directory from config file
    #[arg(short, long)]
    pub data_dir: Option<PathBuf>,

    /// Override monitor socket path from config file
    #[arg(short, long)]
    pub monitor_socket: Option<PathBuf>,
}

impl Config {
    pub fn load(args: CliArgs) -> Result<Self, Box<dyn std::error::Error>> {
        // Load base config from file
        let mut config = if args.config.exists() {
            let contents = std::fs::read_to_string(&args.config)?;
            toml::from_str(&contents)?
        } else {
            // Default configuration
            let mut subscriptions = HashMap::new();
            subscriptions.insert("book".to_string(), vec!["BTC/USD".to_string(), "ETH/USD".to_string()]);
            subscriptions.insert("ticker".to_string(), vec!["BTC/USD".to_string(), "ETH/USD".to_string()]);
            
            Config {
                subscriptions,
                data_dir: PathBuf::from("data"),
                monitor_socket: PathBuf::from("/tmp/collector.sock"),
            }
        };

        // Override with command line arguments
        if let Some(subscription_args) = args.subscriptions {
            let mut new_subscriptions = HashMap::new();
            for arg in subscription_args {
                let parts: Vec<&str> = arg.split(':').collect();
                if parts.len() != 2 {
                    return Err("Invalid subscription format. Use channel:symbol1,symbol2,...".into());
                }
                let channel = parts[0].to_string();
                let symbols: Vec<String> = parts[1].split(',').map(|s| s.to_string()).collect();
                new_subscriptions.insert(channel, symbols);
            }
            config.subscriptions = new_subscriptions;
        }
        if let Some(data_dir) = args.data_dir {
            config.data_dir = data_dir;
        }
        if let Some(monitor_socket) = args.monitor_socket {
            config.monitor_socket = monitor_socket;
        }

        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&config.data_dir)?;

        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::load(CliArgs::parse_from(&["program"])).unwrap();
        assert_eq!(config.subscriptions.get("book").unwrap(), &vec!["BTC/USD", "ETH/USD"]);
        assert_eq!(config.subscriptions.get("ticker").unwrap(), &vec!["BTC/USD", "ETH/USD"]);
        assert_eq!(config.data_dir, PathBuf::from("data"));
        assert_eq!(config.monitor_socket, PathBuf::from("/tmp/collector.sock"));
    }

    #[test]
    fn test_cli_override() {
        let args = CliArgs::parse_from(&[
            "program",
            "--subscriptions", "book:BTC/USD,ETH/USD ticker:BTC/USD",
            "--data-dir", "/custom/data",
            "--monitor-socket", "/custom/sock"
        ]);
        
        let config = Config::load(args).unwrap();
        assert_eq!(config.subscriptions.get("book").unwrap(), &vec!["BTC/USD", "ETH/USD"]);
        assert_eq!(config.subscriptions.get("ticker").unwrap(), &vec!["BTC/USD"]);
        assert_eq!(config.data_dir, PathBuf::from("/custom/data"));
        assert_eq!(config.monitor_socket, PathBuf::from("/custom/sock"));
    }
} 