#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types

mod addresses;
mod conversion;
mod etherscan;
mod processing;
mod types;

use crate::processing::{
    current_balance, process_transactions, range_filter, total_earnings, unliquidated,
};
use crate::types::RewardEvent;
use ethers::utils::format_ether;

async fn proc() {
    let mut events = process_transactions().await;
    events.sort();

    let year = 2023;

    let filtered_events: Vec<&RewardEvent> = events
        .iter()
        .filter(|event| match event {
            RewardEvent::Withdrawal { reward } => range_filter(&reward.date, year),
            RewardEvent::ProducedBlock { reward } => range_filter(&reward.date, year),
            RewardEvent::MevReward { reward } => range_filter(&reward.date, year),
            RewardEvent::MevRewardInternal { reward, .. } => range_filter(&reward.date, year),
            RewardEvent::Outgoing { reward, .. } => range_filter(&reward.date, year),
        })
        .collect();

    for r in filtered_events {
        match r {
            RewardEvent::ProducedBlock { reward } => {
                println!(
                    "{}: {:.6} ETH, {:.2} EUR (block)",
                    reward.date,
                    format_ether(reward.amount),
                    reward.fiat
                )
            }
            RewardEvent::Withdrawal { reward } => {
                println!(
                    "{}: {:.6} ETH, {:.2} EUR (withdrawal)",
                    reward.date,
                    format_ether(reward.amount),
                    reward.fiat
                )
            }
            RewardEvent::MevReward { reward } => {
                println!(
                    "{}: {:.6} ETH, {:.2} EUR (mev)",
                    reward.date,
                    format_ether(reward.amount),
                    reward.fiat
                )
            }
            RewardEvent::MevRewardInternal { reward } => {
                println!(
                    "{}: {:.6} ETH, {:.2} EUR (mev internal)",
                    reward.date,
                    format_ether(reward.amount),
                    reward.fiat
                )
            }
            RewardEvent::Outgoing { reward, gas: _gas } => {
                println!(
                    "{}: {:.6}, {:.2} EUR (outgoing)",
                    reward.date,
                    format_ether(reward.amount),
                    reward.fiat,
                )
            }
        }
    }

    println!(
        "Current Balance: {} ETH",
        format_ether(current_balance(&events))
    );

    println!(
        "Sum: {} EUR, unliquidated: {} EUR",
        total_earnings(&events),
        unliquidated(&events),
    );
}

#[tokio::main]
async fn main() {
    proc().await;
}
