use crate::conversion::fetch_ethereum_price;
use async_throttle::RateLimiter;
use chrono::NaiveDateTime;
use ethers::types::{U256, U64};
use ethers::utils::format_ether;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RewardEvent {
    Withdrawal { reward: Reward },
    ProducedBlock { reward: Reward },
    MevReward { reward: Reward },
    MevRewardInternal { reward: Reward },
    Outgoing { reward: Reward, fee: U256 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reward {
    pub block: U64,
    pub id: String,
    pub date: NaiveDateTime,
    pub amount: U256,
    pub price: f64,
    pub fiat: f64,
}
impl Eq for Reward {}

impl Reward {
    pub async fn new(
        block: U64,
        id: String,
        timestamp: String,
        amount: U256,
        limiter: Arc<RateLimiter>,
    ) -> Self {
        let unix_time = timestamp.parse::<i64>().unwrap();
        let date = NaiveDateTime::from_timestamp_opt(unix_time, 0).unwrap();
        let price = fetch_ethereum_price(&date, limiter.clone()).await.unwrap();

        Reward {
            block,
            id,
            date,
            amount,
            price,
            fiat: format_ether(amount).parse::<f64>().unwrap() * price,
        }
    }
}

impl Ord for RewardEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let date = match self {
            RewardEvent::ProducedBlock { reward } => reward.date,
            RewardEvent::Withdrawal { reward } => reward.date,
            RewardEvent::MevReward { reward } => reward.date,
            RewardEvent::MevRewardInternal { reward } => reward.date,
            RewardEvent::Outgoing { reward, fee: _gas } => reward.date,
        };

        let other_date = match other {
            RewardEvent::ProducedBlock { reward } => reward.date,
            RewardEvent::Withdrawal { reward } => reward.date,
            RewardEvent::MevReward { reward } => reward.date,
            RewardEvent::MevRewardInternal { reward } => reward.date,
            RewardEvent::Outgoing { reward, fee: _gas } => reward.date,
        };

        return date.cmp(&other_date);
    }
}

impl PartialOrd for RewardEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
