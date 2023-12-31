extern crate chrono;

use crate::addresses::{consensus_rewards_address, execution_rewards_address};
use crate::conversion::coingecko_rate_limiter;
use crate::etherscan::{
    beacon_withdrawal_transactions, etherscan_client, internal_transactions, produced_blocks,
    transactions,
};
use crate::processing::chrono::Datelike;
use crate::types::{Reward, RewardEvent};
use chrono::NaiveDateTime;
use ethers::types::U256;
use std::ops::Mul;
use std::sync::Arc;
use RewardEvent::Outgoing;

pub async fn process_transactions() -> Vec<RewardEvent> {
    let client = etherscan_client().unwrap();
    let execution_addr = execution_rewards_address();
    let consensus_addr = consensus_rewards_address();

    let limiter = Arc::new(coingecko_rate_limiter());

    let mut rewards: Vec<RewardEvent> = vec![];

    // Produced blocks
    for block in produced_blocks(&client, execution_addr).await {
        let event = RewardEvent::ProducedBlock {
            reward: Reward::new(
                block.block_number.as_number().unwrap(),
                String::from(""),
                block.time_stamp,
                U256::from_dec_str(block.block_reward.as_str()).unwrap(),
                limiter.clone(),
            )
            .await,
        };
        rewards.push(event);
    }

    // Transactions
    for tx in transactions(&client, execution_addr).await {
        let event = if tx.to.unwrap() == execution_addr {
            RewardEvent::MevReward {
                reward: Reward::new(
                    tx.block_number.as_number().unwrap(),
                    tx.hash.value().unwrap().to_string(),
                    tx.time_stamp,
                    tx.value,
                    limiter.clone(),
                )
                .await,
            }
        } else {
            Outgoing {
                reward: Reward::new(
                    tx.block_number.as_number().unwrap(),
                    tx.hash.value().unwrap().to_string(),
                    tx.time_stamp,
                    tx.value,
                    limiter.clone(),
                )
                .await,
                fee: tx.gas_used.mul(tx.gas_price.unwrap()),
            }
        };
        rewards.push(event);
    }

    // Transactions
    for tx in transactions(&client, consensus_addr).await {
        if tx.from.value().unwrap().eq(&consensus_addr) {
            let event = Outgoing {
                reward: Reward::new(
                    tx.block_number.as_number().unwrap(),
                    tx.hash.value().unwrap().to_string(),
                    tx.time_stamp,
                    tx.value,
                    limiter.clone(),
                )
                .await,
                fee: tx.gas_used.mul(tx.gas_price.unwrap()),
            };
            rewards.push(event);
        }
    }

    // Internal Transactions TODO
    for tx in internal_transactions(&client, execution_addr).await {
        let event = RewardEvent::MevRewardInternal {
            reward: Reward::new(
                tx.block_number.as_number().unwrap(),
                tx.hash.to_string(),
                tx.time_stamp,
                tx.value,
                limiter.clone(),
            )
            .await,
        };
        rewards.push(event);
    }

    for tx in beacon_withdrawal_transactions(&client, consensus_addr).await {
        let event = RewardEvent::Withdrawal {
            reward: Reward::new(
                tx.block_number.as_number().unwrap(),
                tx.validator_index.to_string(),
                tx.timestamp,
                U256::exp10(9) * U256::from_dec_str(tx.amount.as_str()).unwrap(),
                limiter.clone(),
            )
            .await,
        };
        rewards.push(event);
    }

    return rewards;
}

pub fn range_filter(date: &NaiveDateTime, year: i32) -> bool {
    date.year() == year
}

pub fn current_balance(events: &Vec<RewardEvent>) -> U256 {
    let mut sum = U256::from(0);

    for e in events {
        match e {
            RewardEvent::ProducedBlock { reward, .. } => sum += reward.amount,
            RewardEvent::Withdrawal { reward, .. } => sum += reward.amount,
            RewardEvent::MevReward { reward, .. } => sum += reward.amount,
            RewardEvent::MevRewardInternal { reward, .. } => sum += reward.amount,
            Outgoing { reward, fee: gas } => sum -= reward.amount + gas,
        }
    }

    return sum;
}

pub fn total_earnings(events: &Vec<RewardEvent>) -> f64 {
    let mut sum = 0.0;

    for e in events {
        match e {
            RewardEvent::ProducedBlock { reward, .. } => sum += reward.fiat,
            RewardEvent::Withdrawal { reward, .. } => sum += reward.fiat,
            RewardEvent::MevReward { reward, .. } => sum += reward.fiat,
            RewardEvent::MevRewardInternal { reward, .. } => sum += reward.fiat,
            Outgoing { .. } => {}
        }
    }
    return sum;
}

pub fn unliquidated(events: &Vec<RewardEvent>) -> f64 {
    let mut sum = 0.0;

    for e in events {
        match e {
            RewardEvent::ProducedBlock { reward, .. } => sum += reward.fiat,
            RewardEvent::Withdrawal { reward, .. } => sum += reward.fiat,
            RewardEvent::MevReward { reward, .. } => sum += reward.fiat,
            RewardEvent::MevRewardInternal { reward, .. } => sum += reward.fiat,
            Outgoing { reward, .. } => sum -= reward.fiat,
        }
    }
    return sum;
}
