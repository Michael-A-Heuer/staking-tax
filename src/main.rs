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
use crate::types::{Reward, RewardEvent};
use csv::Writer;
use ethers::types::U256;
use ethers::utils::format_ether;
use std::fs::File;

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

    let mut execution_writer =
        csv::Writer::from_path(format!("report-execution-{}.csv", year)).unwrap();
    let mut consensus_writer =
        csv::Writer::from_path(format!("report-consensus-{}.csv", year)).unwrap();
    let mut fees_writer = csv::Writer::from_path(format!("report-fees-{}.csv", year)).unwrap();

    let header = ["Date", "Block", "Type", "ID", "ETH", "ETH_EUR_Price", "EUR"];
    execution_writer.write_record(header).unwrap();
    consensus_writer.write_record(header).unwrap();
    fees_writer.write_record(header).unwrap();

    for r in filtered_events {
        match r {
            RewardEvent::Withdrawal { reward } => {
                write_reward(&mut consensus_writer, reward, "Withdrawal");
            }
            RewardEvent::ProducedBlock { reward } => {
                write_reward(&mut execution_writer, reward, "Block");
            }
            RewardEvent::MevReward { reward } => {
                write_reward(&mut execution_writer, reward, "MevReward");
            }
            RewardEvent::MevRewardInternal { reward } => {
                write_reward(&mut execution_writer, reward, "MevRewardInternal");
            }
            RewardEvent::Outgoing { reward, fee } => {
                write_fee(&mut fees_writer, reward, *fee, "Fee");
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

fn write_fee(fees_writer: &mut Writer<File>, reward: &Reward, fee: U256, type_name: &str) {
    let price = reward.price;
    let fee = format_ether(fee).parse::<f64>().unwrap();
    fees_writer
        .write_record([
            reward.date.to_string(),
            reward.block.to_string(),
            type_name.to_string(),
            reward.id.to_string(),
            format!("{:.8}", fee),
            price.to_string(),
            (fee * price).to_string(),
        ])
        .unwrap();
}

fn write_reward(writer: &mut Writer<File>, reward: &Reward, type_name: &str) {
    writer
        .write_record([
            reward.date.to_string(),
            reward.block.to_string(),
            type_name.to_string(),
            reward.id.to_string(),
            format!("{:.8}", format_ether(reward.amount)),
            reward.price.to_string(),
            reward.fiat.to_string(),
        ])
        .unwrap();
}

#[tokio::main]
async fn main() {
    proc().await;
}
