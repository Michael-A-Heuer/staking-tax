use dotenv::dotenv;
use ethers::core::types::Chain;
use ethers::etherscan::account::{
    BeaconWithdrawalTransaction, BlockType, InternalTransaction, InternalTxQueryOption::ByAddress,
    MinedBlock, NormalTransaction,
};
use ethers::etherscan::{errors::EtherscanError, Client};

use std::env;

pub fn etherscan_client() -> Result<Client, EtherscanError> {
    dotenv().ok();

    let etherscan_api_key =
        env::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY not found in .env");

    Client::new(Chain::Mainnet, etherscan_api_key)
}

pub async fn internal_transactions(
    client: &Client,
    address: ethers::types::Address,
) -> Vec<InternalTransaction> {
    println!("Querying internal txns for address {}", address);

    client
        .get_internal_transactions(ByAddress(address), None)
        .await
        .unwrap()
}

pub async fn transactions(
    client: &Client,
    address: ethers::types::Address,
) -> Vec<NormalTransaction> {
    println!("Querying txns for address {}", address);

    client.get_transactions(&address, None).await.unwrap()
}

pub async fn produced_blocks(client: &Client, address: ethers::types::Address) -> Vec<MinedBlock> {
    println!("Querying produced blocks for address {}", address);

    client
        .get_mined_blocks(&address, Some(BlockType::CanonicalBlocks), None)
        .await
        .unwrap()
}

pub async fn beacon_withdrawal_transactions(
    client: &Client,
    address: ethers::types::Address,
) -> Vec<BeaconWithdrawalTransaction> {
    println!("Querying beacon withdrawals for address {}", address);

    client
        .get_beacon_withdrawal_transactions(&address, None)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_beacon_withdrawal_transactions() {
    let txs = beacon_withdrawal_transactions(
        &etherscan_client().unwrap(),
        crate::addresses::consensus_rewards_address(),
    )
    .await;

    for tx in txs.iter() {
        println!("Withdrawal index: {}", tx.withdrawal_index);
        println!("Block number: {}", tx.block_number);
        println!("Timestamp: {}", tx.timestamp);
        println!("Address: {}", tx.address);
        println!("Amount: {}", tx.amount);
    }

    println!("Blocks: {}", txs.len())
}

#[tokio::test]
async fn test_produced_blocks() {
    let txs = produced_blocks(
        &etherscan_client().unwrap(),
        crate::addresses::execution_rewards_address(),
    )
    .await;

    for tx in txs.iter() {
        println!("Block reward: {}", tx.block_reward);
        println!("Block number: {}", tx.block_number);
        println!("Timestamp: {}", tx.time_stamp);
        println!();
    }

    println!("Blocks: {}", txs.len())
}

#[tokio::test]
async fn test_transactions() {
    let txs = transactions(
        &etherscan_client().unwrap(),
        crate::addresses::consensus_rewards_address(),
    )
    .await;

    for tx in txs.iter() {
        println!("Transaction Hash: {:?}", tx.hash);
        println!("From: {:?}", tx.from);
        println!("To: {:?}", tx.to);
        println!("Value: {:?}", tx.value);
        println!("Gas Price: {:?}", tx.gas);
        println!("Gas Limit: {:?}", tx.gas_used);
        println!("Block Number: {:?}", tx.block_number);
        println!("Timestamp: {:?}", tx.time_stamp);
        println!();
    }
}

#[tokio::test]
async fn test_internal_transactions() {
    let internal_transactions = internal_transactions(
        &etherscan_client().unwrap(),
        crate::addresses::execution_rewards_address(),
    )
    .await;

    for tx in internal_transactions.iter() {
        println!("Transaction Hash: {:?}", tx.hash);
        println!("From: {:?}", tx.from);
        println!("To: {:?}", tx.to);
        println!("Value: {:?}", tx.value);
        println!("Gas Price: {:?}", tx.gas);
        println!("Gas Limit: {:?}", tx.gas_used);
        println!("Block Number: {:?}", tx.block_number);
        println!("Timestamp: {:?}", tx.time_stamp);
        println!();
    }
}
