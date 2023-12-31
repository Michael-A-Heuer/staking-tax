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
        csv::Writer::from_path(format!("Execution Rewards {}.csv", year)).unwrap();
    let mut consensus_writer =
        csv::Writer::from_path(format!("Consensus Rewards {}.csv", year)).unwrap();
    let mut fees_writer = csv::Writer::from_path(format!("Fees {}.csv", year)).unwrap();

    let header = ["Date", "Block", "Type", "ID", "ETH", "ETH_EUR_Price", "EUR"];
    execution_writer.write_record(header).unwrap();
    consensus_writer.write_record(header).unwrap();
    fees_writer.write_record(header).unwrap();

    let mut execution_rows = 1;
    let mut consensus_rows = 1;
    let mut fees_rows = 1;
    for r in filtered_events {
        match r {
            RewardEvent::Withdrawal { reward } => {
                write_reward(&mut consensus_writer, reward, "Withdrawal");
                consensus_rows += 1;
            }
            RewardEvent::ProducedBlock { reward } => {
                write_reward(&mut execution_writer, reward, "Block");
                execution_rows += 1;
            }
            RewardEvent::MevReward { reward } => {
                write_reward(&mut execution_writer, reward, "MevReward");
                execution_rows += 1;
            }
            RewardEvent::MevRewardInternal { reward } => {
                write_reward(&mut execution_writer, reward, "MevRewardInternal");
                execution_rows += 1;
            }
            RewardEvent::Outgoing { reward, fee } => {
                write_fee(&mut fees_writer, reward, *fee, "Fee");
                fees_rows += 1;
            }
        }
    }

    execution_writer
        .write_record(footer(execution_rows))
        .unwrap();
    consensus_writer
        .write_record(footer(consensus_rows))
        .unwrap();
    fees_writer.write_record(footer(fees_rows)).unwrap();

    execution_writer.flush().unwrap();
    consensus_writer.flush().unwrap();
    fees_writer.flush().unwrap();

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

fn footer(rows: i32) -> [&'static str; 7] {
    let fee_eth = format!("=SUM(E2:E{})", rows);
    let fee_eur = format!("=SUM(G2:G{})", rows);
    [
        "",
        "",
        "",
        "",
        Box::leak(fee_eth.into_boxed_str()),
        "",
        Box::leak(fee_eur.into_boxed_str()),
    ]
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
