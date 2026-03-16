use std::{env, str::FromStr, sync::Arc, time::Duration};

use anchor_lang::InstructionData;
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use dotenvy::dotenv;
use futures::StreamExt;
use serde::Serialize;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_response::RpcLogsResponse,
};
use solana_commitment_config::CommitmentConfig;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::{read_keypair_file, Keypair};
use solana_pubkey::Pubkey;
use solana_rpc_client_types::config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::Transaction;
use tokio::time::interval;
use tracing::{error, info, warn};

const DEFAULT_PRICE_POLL_INTERVAL_SEC: u64 = 600; // 10 minutes; live price from Binance when MOCK_PRICE is not set

#[derive(Clone)]
struct Config {
    rpc_http: String,
    rpc_ws: String,
    oracle_program_id: Pubkey,
    oracle_state: Pubkey,
    minter_program_id: Pubkey,
    backend_keypair_path: String,
    price_poll_interval: Duration,
    mock_price: Option<u64>,
    price_api_url: Option<String>,
}

impl Config {
    fn from_env() -> Result<Self> {
        let rpc_http =
            env::var("SOLANA_RPC_HTTP").context("SOLANA_RPC_HTTP env var is required")?;
        let rpc_ws = env::var("SOLANA_RPC_WS").context("SOLANA_RPC_WS env var is required")?;
        let oracle_program_id = Pubkey::from_str(
            &env::var("ORACLE_PROGRAM_ID").context("ORACLE_PROGRAM_ID is required")?,
        )?;
        let oracle_state =
            Pubkey::from_str(&env::var("ORACLE_STATE_PUBKEY").context("ORACLE_STATE_PUBKEY")?)?;
        let minter_program_id = Pubkey::from_str(
            &env::var("MINTER_PROGRAM_ID").context("MINTER_PROGRAM_ID is required")?,
        )?;
        let mut backend_keypair_path =
            env::var("BACKEND_KEYPAIR_PATH").context("BACKEND_KEYPAIR_PATH is required")?;
        if backend_keypair_path.starts_with("~/") {
            if let Some(home) = env::var_os("HOME") {
                backend_keypair_path =
                    format!("{}/{}", home.to_string_lossy(), &backend_keypair_path[2..]);
            }
        }
        let poll = env::var("PRICE_POLL_INTERVAL_SEC")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_PRICE_POLL_INTERVAL_SEC);
        let mock_price = env::var("MOCK_PRICE")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        let price_api_url = env::var("PRICE_API_URL").ok();

        Ok(Self {
            rpc_http,
            rpc_ws,
            oracle_program_id,
            oracle_state,
            minter_program_id,
            backend_keypair_path,
            price_poll_interval: Duration::from_secs(poll),
            mock_price,
            price_api_url,
        })
    }
}

#[derive(Clone)]
enum PriceSource {
    Mock(u64),
    Http { url: String },
}

impl PriceSource {
    fn from_config(cfg: &Config) -> Self {
        if let Some(mock) = cfg.mock_price {
            PriceSource::Mock(mock)
        } else if let Some(url) = cfg.price_api_url.clone() {
            PriceSource::Http { url }
        } else {
            PriceSource::Http {
                url: "https://api.binance.com/api/v3/ticker/price?symbol=SOLUSDT".to_string(),
            }
        }
    }

    async fn fetch_price(&self) -> Result<u64> {
        match self {
            PriceSource::Mock(val) => Ok(*val),
            PriceSource::Http { url } => {
                #[derive(serde::Deserialize)]
                struct Resp {
                    price: String,
                }
                let resp: Resp = reqwest::get(url).await?.json().await?;
                to_fixed_6(&resp.price)
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct TokenCreatedLog {
    creator: String,
    mint: String,
    decimals: u8,
    initial_supply: u64,
    fee_lamports: u64,
    sol_usd_price: u64,
    slot: u64,
    signature: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::from_env()?;
    let price_source = PriceSource::from_config(&cfg);
    let admin = Arc::new(
        read_keypair_file(&cfg.backend_keypair_path)
            .map_err(|e| anyhow!(e.to_string()))
            .context("read backend keypair")?,
    );

    let price_task = tokio::spawn(run_price_updater(cfg.clone(), price_source, admin.clone()));
    let listener_task = tokio::spawn(run_event_listener(cfg.clone()));

    let (price_res, listener_res) = tokio::try_join!(price_task, listener_task)?;
    price_res?;
    listener_res?;
    Ok(())
}

async fn run_price_updater(
    cfg: Config,
    price_source: PriceSource,
    admin: Arc<Keypair>,
) -> Result<()> {
    let client = RpcClient::new(cfg.rpc_http.clone());
    let mut ticker = interval(cfg.price_poll_interval);

    // Run one update immediately on startup
    try_update_price(&client, &cfg, &price_source, admin.clone(), "initial").await;

    loop {
        ticker.tick().await;
        try_update_price(&client, &cfg, &price_source, admin.clone(), "scheduled").await;
    }
}

async fn try_update_price(
    client: &RpcClient,
    cfg: &Config,
    price_source: &PriceSource,
    admin: Arc<Keypair>,
    kind: &'static str,
) {
    match price_source.fetch_price().await {
        Ok(price) => {
            if price == 0 {
                warn!(
                    "Skipped {} price update because fetched price is zero",
                    kind
                );
                return;
            }
            match submit_price(client, cfg, price, admin).await {
                Ok(sig) => info!(%sig, price, "oracle price updated ({})", kind),
                Err(err) => error!(?err, "failed to submit {} price", kind),
            }
        }
        Err(err) => error!(?err, "failed to fetch {} price", kind),
    }
}

async fn submit_price(
    client: &RpcClient,
    cfg: &Config,
    new_price: u64,
    admin: Arc<Keypair>,
) -> Result<Signature> {
    use sol_usd_oracle::instruction;

    let ix_data = instruction::UpdatePrice { new_price }.data();
    let accounts = vec![
        AccountMeta::new(cfg.oracle_state, false),
        AccountMeta::new(admin.pubkey(), true),
    ];

    let ix = Instruction {
        program_id: cfg.oracle_program_id,
        accounts,
        data: ix_data,
    };

    let bh = client
        .get_latest_blockhash()
        .await
        .context("fetch blockhash")?;

    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&admin.pubkey()), &[admin.as_ref()], bh);

    let sig = client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::confirmed(),
        )
        .await
        .context("send price tx")?;

    Ok(sig)
}

async fn run_event_listener(cfg: Config) -> Result<()> {
    let client = PubsubClient::new(&cfg.rpc_ws)
        .await
        .context("connect pubsub ws")?;

    let (mut stream, _unsub) = client
        .logs_subscribe(
            RpcTransactionLogsFilter::Mentions(vec![cfg.minter_program_id.to_string()]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::confirmed()),
            },
        )
        .await
        .context("subscribe to logs")?;

    while let Some(value) = stream.next().await {
        if let Some(parsed) = parse_token_created(&value.value, cfg.minter_program_id) {
            info!(
                target: "token_created",
                "creator={} mint={} decimals={} supply={} fee_lamports={} price={} slot={} sig={}",
                parsed.creator,
                parsed.mint,
                parsed.decimals,
                parsed.initial_supply,
                parsed.fee_lamports,
                parsed.sol_usd_price,
                parsed.slot,
                parsed.signature
            );
            if let Ok(json) = serde_json::to_string(&parsed) {
                println!("{json}");
            }
        }
    }
    Ok(())
}

/// Anchor event discriminator for `TokenCreated`: sha256("event:TokenCreated")[..8]
const TOKEN_CREATED_DISC: [u8; 8] = [0xec, 0x13, 0x29, 0xff, 0x82, 0x4e, 0x93, 0xac];

fn parse_token_created(logs: &RpcLogsResponse, _program_id: Pubkey) -> Option<TokenCreatedLog> {
    for log in &logs.logs {
        let data_b64 = match log.strip_prefix("Program data: ") {
            Some(d) => d,
            None => continue,
        };
        let data = match BASE64.decode(data_b64) {
            Ok(d) => d,
            Err(_) => continue,
        };
        // 8 disc + 32 creator + 32 mint + 1 decimals + 8 supply + 8 fee + 8 price + 8 slot = 105
        if data.len() < 105 || data[..8] != TOKEN_CREATED_DISC {
            continue;
        }

        let creator = Pubkey::new_from_array(<[u8; 32]>::try_from(&data[8..40]).ok()?);
        let mint = Pubkey::new_from_array(<[u8; 32]>::try_from(&data[40..72]).ok()?);
        let decimals = data[72];
        let initial_supply = u64::from_le_bytes(<[u8; 8]>::try_from(&data[73..81]).ok()?);
        let fee_lamports = u64::from_le_bytes(<[u8; 8]>::try_from(&data[81..89]).ok()?);
        let sol_usd_price = u64::from_le_bytes(<[u8; 8]>::try_from(&data[89..97]).ok()?);
        let slot = u64::from_le_bytes(<[u8; 8]>::try_from(&data[97..105]).ok()?);

        return Some(TokenCreatedLog {
            creator: creator.to_string(),
            mint: mint.to_string(),
            decimals,
            initial_supply,
            fee_lamports,
            sol_usd_price,
            slot,
            signature: logs.signature.clone(),
        });
    }
    None
}

fn to_fixed_6(txt: &str) -> Result<u64> {
    let (int_str, frac_str) = match txt.split_once('.') {
        Some((i, f)) => (i, f),
        None => (txt, ""),
    };

    let int_part: u64 = int_str
        .parse()
        .map_err(|_| anyhow!("invalid integer part: {}", int_str))?;

    let frac_truncated = if frac_str.len() > 6 {
        &frac_str[..6]
    } else {
        frac_str
    };

    let frac_padded = format!("{:0<6}", frac_truncated);
    let frac_part: u64 = frac_padded
        .parse()
        .map_err(|_| anyhow!("invalid fractional part: {}", frac_str))?;

    Ok(int_part * 1_000_000 + frac_part)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::rpc_response::RpcLogsResponse;

    fn sample_cfg(mock_price: Option<u64>, price_api_url: Option<&str>) -> Config {
        Config {
            rpc_http: "http://127.0.0.1:8899".to_string(),
            rpc_ws: "ws://127.0.0.1:8900".to_string(),
            oracle_program_id: Pubkey::new_unique(),
            oracle_state: Pubkey::new_unique(),
            minter_program_id: Pubkey::new_unique(),
            backend_keypair_path: "/tmp/id.json".to_string(),
            price_poll_interval: Duration::from_secs(60),
            mock_price,
            price_api_url: price_api_url.map(ToString::to_string),
        }
    }

    #[test]
    fn to_fixed_6_parses_integer_and_fractional_part() {
        assert_eq!(to_fixed_6("120").unwrap(), 120_000_000);
        assert_eq!(to_fixed_6("120.12").unwrap(), 120_120_000);
        assert_eq!(to_fixed_6("0.000001").unwrap(), 1);
    }

    #[test]
    fn to_fixed_6_truncates_fraction_to_six_digits() {
        assert_eq!(to_fixed_6("1.1234569").unwrap(), 1_123_456);
    }

    #[test]
    fn to_fixed_6_rejects_invalid_input() {
        assert!(to_fixed_6("abc").is_err());
    }

    #[test]
    fn parse_token_created_reads_expected_fields() {
        // Real Anchor event: base64(discriminator + borsh-serialized TokenCreated)
        // creator=[1;32], mint=[2;32], decimals=6, supply=1_000_000,
        // fee=41_666_666, price=120_000_000, slot=77
        let logs = RpcLogsResponse {
            signature: "5Yf8k3w2J3k9R8B9Q2".to_string(),
            err: None,
            logs: vec![
                "Program xyz log".to_string(),
                "Program data: 7BMp/4JOk6wBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBkBCDwAAAAAAash7AgAAAAAADicHAAAAAE0AAAAAAAAA".to_string(),
            ],
        };

        let parsed = parse_token_created(&logs, Pubkey::new_unique()).expect("event should parse");
        assert_eq!(parsed.creator, "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi");
        assert_eq!(parsed.mint, "8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR");
        assert_eq!(parsed.decimals, 6);
        assert_eq!(parsed.initial_supply, 1_000_000);
        assert_eq!(parsed.fee_lamports, 41_666_666);
        assert_eq!(parsed.sol_usd_price, 120_000_000);
        assert_eq!(parsed.slot, 77);
        assert_eq!(parsed.signature, logs.signature);
    }

    #[test]
    fn parse_token_created_returns_none_for_unrelated_logs() {
        let logs = RpcLogsResponse {
            signature: "4m4u3z".to_string(),
            err: None,
            logs: vec!["Program log: some other event".to_string()],
        };
        assert!(parse_token_created(&logs, Pubkey::new_unique()).is_none());
    }

    #[test]
    fn price_source_prefers_mock_over_url() {
        let cfg = sample_cfg(Some(123), Some("https://example.com/price"));
        let source = PriceSource::from_config(&cfg);
        match source {
            PriceSource::Mock(v) => assert_eq!(v, 123),
            PriceSource::Http { .. } => panic!("expected mock source"),
        }
    }

    #[test]
    fn price_source_uses_default_url_when_no_override() {
        let cfg = sample_cfg(None, None);
        let source = PriceSource::from_config(&cfg);
        match source {
            PriceSource::Mock(_) => panic!("expected http source"),
            PriceSource::Http { url } => {
                assert_eq!(
                    url,
                    "https://api.binance.com/api/v3/ticker/price?symbol=SOLUSDT"
                )
            }
        }
    }
}
