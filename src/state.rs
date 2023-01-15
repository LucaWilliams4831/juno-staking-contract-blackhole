use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw20::Denom;
use cw_storage_plus::{Item, Map};
use crate::msg::{ StakerInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub stake_token_address: Addr,
    pub reward_charity_address: Addr,
    pub reward_burn_address: Addr,
    pub reward_artists_address: Addr,
    pub reward_token_denom: String,
    pub reward_interval: u64,
    pub enabled: bool
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const STAKERS_KEY: &str = "stakers";
pub const STAKERS: Map<Addr, Vec<StakerInfo>> = Map::new(STAKERS_KEY);

pub const RANK_STAKERS_KEY: &str = "rank_stakers";
pub const RANK_STAKERS: Map<u8, (Addr, Uint128)> = Map::new(RANK_STAKERS_KEY);

pub const RANKS_KEY: &str = "ranks";
pub const RANKS: Map<Addr, Uint128> = Map::new(RANKS_KEY);