use dotenv::dotenv;
use std::env;

pub fn execution_rewards_address() -> ethers::types::Address {
    dotenv().ok();

    env::var("EXECUTION_REWARDS_ADDRESS")
        .expect("EXECUTION_REWARDS_ADDRESS not found in .env")
        .parse()
        .unwrap()
}

pub fn consensus_rewards_address() -> ethers::types::Address {
    dotenv().ok();

    env::var("CONSENSUS_REWARDS_ADDRESS")
        .expect("CONSENSUS_REWARDS_ADDRESS not found in .env")
        .parse()
        .unwrap()
}
