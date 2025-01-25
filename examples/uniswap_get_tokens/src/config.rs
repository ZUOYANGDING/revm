use clap::Parser;
use serde_derive::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// [CLIArguments] defines command-line arguments for Transaction Courier service.
#[derive(Parser, Debug, Clone)]
pub(crate) struct CLIArguments {
    /// Path to the configuration file.
    #[clap(long, value_parser)]
    pub(crate) config_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SwapPool {
    pub(crate) name: String,
    pub(crate) address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UniSwapConfig {
    pub(crate) db_path: PathBuf,
    pub(crate) swap_pools: Vec<SwapPool>,
}

pub(crate) fn load_config(config_path: &str) -> Result<UniSwapConfig, String> {
    let file_string = fs::read_to_string(config_path).map_err(|e| {
        format!(
            "Error: Config file (config.toml) is not found.
            Please ensure that the configuration directory \"{}\" exists.
            ERROR: {:?}",
            config_path, e
        )
    })?;

    toml::from_str::<UniSwapConfig>(&file_string)
        .map_err(|e| format!("fail to parse config file: {:?}", e))
}

#[cfg(test)]
mod test {
    use super::load_config;

    #[test]
    fn config_parse_test() {
        let mut cur_dir = std::env::current_dir().unwrap();
        cur_dir.push("example_config_file/config.toml");
        match load_config(cur_dir.to_str().unwrap()) {
            Ok(result) => {
                for pool in result.swap_pools {
                    println!("{:?}, {:?}", pool.address, pool.name);
                }
            }
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }
}
