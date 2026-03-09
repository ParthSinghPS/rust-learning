use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    BlockNumber,
    GetBalance { address: String },
}

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<String>,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u64>,   // allow null from server
    result: Option<String>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let rpc_url = "https://ethereum.publicnode.com";

    let request = match cli.command {
        Commands::BlockNumber => JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_blockNumber".to_string(),
            params: vec![],
        },

        Commands::GetBalance { address } => JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_getBalance".to_string(),
            params: vec![address, "latest".to_string()],
        },
    };

    let client = Client::new();

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await?
        .error_for_status()?;

    let rpc_response: JsonRpcResponse = response.json().await?;

    match rpc_response.result {
        Some(result) => {
            if result.starts_with("0x") {
                let value = u64::from_str_radix(&result[2..], 16)?;
                println!("Parsed result: {}", value);
            } else {
                println!("Result: {}", result);
            }
        }
        None => {
            if let Some(err) = rpc_response.error {
                println!("RPC Error {}: {}", err.code, err.message);
            }
        }
    }

    Ok(())
}