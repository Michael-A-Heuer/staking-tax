// https://api.coingecko.com/api/v3/coins/ethereum/history?date=30-12-2022

use chrono::NaiveDateTime;
use reqwest;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use dotenv::dotenv;
use std::error::Error;
use std::string::ToString;
use std::sync::Arc;

use async_throttle::RateLimiter;

#[derive(Debug, Deserialize)]
struct CoinGeckoResponse {
    //id: String,
    //symbol: String,
    //name: String,
    market_data: MarketData,
}

#[derive(Debug, Deserialize)]
struct MarketData {
    current_price: CurrentPrice,
}

#[derive(Debug, Deserialize)]
struct CurrentPrice {
    eur: f64,
}

const FILE_PATH: &str = "historic_prices.json";

pub async fn fetch_ethereum_price(
    datetime: &NaiveDateTime,
    limiter: Arc<RateLimiter>,
) -> Result<f64, Box<dyn Error>> {
    let date = datetime.format("%d-%m-%Y").to_string();

    match read_ethereum_price(&date).await {
        Ok(val) => match val {
            Some(val) => Ok(val),
            None => {
                let queried_price = query_ethereum_price_throttled(&date, limiter).await?;
                store_ethereum_price(&date, queried_price).await?;

                Ok(queried_price)
            }
        },
        Err(error) => Err(error),
    }
}

pub async fn store_ethereum_price(date: &str, price: f64) -> std::io::Result<()> {
    let mut historic_prices: HashMap<String, f64> = match File::open(FILE_PATH) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        }
        Err(file_error) => match file_error.kind() {
            std::io::ErrorKind::NotFound => {
                File::create(FILE_PATH)?;
                HashMap::new()
            }
            _ => return Err(file_error.into()),
        },
    };

    historic_prices.insert(date.to_string(), price);

    let serialized_data = serde_json::to_string_pretty(&historic_prices)?;

    File::create(FILE_PATH)?.write_all(serialized_data.as_bytes())
}

async fn read_ethereum_price(date: &str) -> Result<Option<f64>, Box<dyn Error>> {
    //println!("Try to read from file...");

    match File::open(FILE_PATH) {
        Ok(mut file) => {
            let mut contents = String::new();
            let _ = file.read_to_string(&mut contents);
            let historic_prices: HashMap<String, f64> = serde_json::from_str(&contents)?;
            Ok(historic_prices.get(date).cloned())
        }
        Err(file_error) => match file_error.kind() {
            std::io::ErrorKind::NotFound => {
                let mut new_file = File::create(FILE_PATH)?;
                new_file.write_all("{}".as_bytes())?;

                Ok(None)
            }
            _ => Err(file_error.into()),
        },
    }
}

async fn query_ethereum_price_throttled(
    date: &str,
    limiter: Arc<RateLimiter>,
) -> Result<f64, Box<dyn Error>> {
    match limiter.throttle(|| query_ethereum_price(&date)).await {
        Ok(val) => Ok(val),
        Err(e) => Err(e),
    }
}
async fn query_ethereum_price(date: &str) -> Result<f64, Box<dyn Error>> {
    dotenv().ok();

    let url = match dotenv::var("COINGECKO_API_KEY") {
        Ok(coingeck_api_key) => {
            format!(
                "https://api.coingecko.com/api/v3/coins/ethereum/history?date={}&x_cg_api_key={}",
                date, coingeck_api_key,
            )
        }
        Err(_) => {
            println!("No CoinGecko API Key provided. Fetching historical prices will take longer.");
            format!(
                "https://api.coingecko.com/api/v3/coins/ethereum/history?date={}",
                date,
            )
        }
    };

    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Err(response.error_for_status().err().unwrap().into());
    }

    let coin_gecko_data: CoinGeckoResponse = response.json().await?;
    Ok(coin_gecko_data.market_data.current_price.eur)
}

pub fn coingecko_rate_limiter() -> RateLimiter {
    let duration = match dotenv::var("COINGECKO_API_KEY") {
        Ok(_) => std::time::Duration::from_secs(4),
        Err(_) => std::time::Duration::from_secs(12),
    };
    println!(
        "CoinGecko Rate-limit: {} calls / min",
        60 / duration.as_secs()
    );

    RateLimiter::new(duration)
}

#[tokio::test]
async fn test_fetch_ethereum_price_on_date() {
    let limiter = coingecko_rate_limiter();
    let date = NaiveDateTime::from_timestamp_opt(1650000000, 0).unwrap();
    let price = fetch_ethereum_price(&date, Arc::new(limiter))
        .await
        .unwrap();
    assert_eq!(price, 2794.538482111171);
}
