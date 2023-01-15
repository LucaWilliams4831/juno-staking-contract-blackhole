use cosmwasm_std::{Addr, Uint128};

use cw20::{Cw20ReceiveMsg, Denom};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub stake_token_address: Addr,
    pub reward_charity_address: Addr,
    pub reward_burn_address: Addr,
    pub reward_artists_address: Addr,
    pub reward_token_denom: String,
    pub reward_interval: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub address: Addr,
    pub amount: Uint128,
    pub reward: Uint128,
    pub last_time: u64,
    pub lock_type: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardDistribution {
    pub juno_reward: bool,
    pub charity: u64,
    pub burn: u64,
    pub artists: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner { owner: Addr },
    UpdateEnabled { enabled: bool },
    UpdateConstants { reward_interval: u64 },
    Receive(Cw20ReceiveMsg),
    WithdrawReward { amount: Uint128 },
    WithdrawStake { amount: Uint128 },
    ClaimReward { distribution: RewardDistribution },
    Unstake {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Stake { lock_type: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Staker { address: Addr },
    ListStakers { start_after: Option<String> },
    GetHoleAmount { address: Addr },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
    pub stake_token_address: Addr,
    pub reward_charity_address: Addr,
    pub reward_burn_address: Addr,
    pub reward_artists_address: Addr,
    pub reward_token_denom: String,
    pub reward_interval: u64,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StakerListResponse {
    pub stakers: Vec<Vec<StakerInfo>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token1ForToken2PriceResponse {
    pub token2_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token2ForToken1PriceResponse {
    pub token1_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TestBalanceResponse {
    pub balance: Uint128,
}
