use crate::constants::{self, LOCKED_TWO_YEAR};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg,
    RewardDistribution, StakerInfo, StakerListResponse, TestBalanceResponse,
    Token1ForToken2PriceResponse, Token2ForToken1PriceResponse,
};
use crate::state::{Config, CONFIG, RANKS, RANK_STAKERS, STAKERS};
use crate::util;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    QueryRequest, Response, StdResult, Storage, Uint128, WasmMsg, WasmQuery,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{
    BalanceResponse as CW20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg, Denom,
    TokenInfoResponse,
};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

// Version info, for migration info
const CONTRACT_NAME: &str = "incentive";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MULTIPLE: u128 = 100u128;
///////////////////////////////////////////////////////// this func is called for instantiating the contract //////////////////////////////////
///
///         input params: owner address
///                       stake token address
///                       reward token address
///                       reward interval
///                       charity wallet address for reward
///                       burn wallet address for reward
///                       artists wallet address for reward
///
///         
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        stake_token_address: msg.stake_token_address,
        reward_token_denom: msg.reward_token_denom,
        reward_interval: msg.reward_interval,
        reward_charity_address: msg.reward_charity_address,
        reward_burn_address: msg.reward_burn_address,
        reward_artists_address: msg.reward_artists_address,
        enabled: true,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { owner } => execute_update_owner(deps, info, owner),
        ExecuteMsg::UpdateEnabled { enabled } => execute_update_enabled(deps, info, enabled),
        ExecuteMsg::UpdateConstants { reward_interval } => {
            execute_update_constants(deps, info, reward_interval)
        }
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::WithdrawReward { amount } => execute_withdraw_reward(deps, env, info, amount),
        ExecuteMsg::WithdrawStake { amount } => execute_withdraw_stake(deps, env, info, amount),
        ExecuteMsg::ClaimReward { distribution } => {
            execute_claim_reward(deps, env, info, distribution)
        }
        ExecuteMsg::Unstake {} => execute_unstake(deps, env, info),
    }
}
///////////////////////////////////////////////////////// this func is called when user click stake button on the frontend //////////////////////////////////
///
///         input params: customer's wallet address
///                       lock_type for claim reward
///         
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    check_enabled(&deps, &info)?;
    let mut cfg = CONFIG.load(deps.storage)?;

    if wrapper.amount == Uint128::zero() {
        return Err(ContractError::InvalidInput {});
    }
    let user_addr = &deps.api.addr_validate(&wrapper.sender)?;

    if info.sender.clone() != cfg.stake_token_address {
        return Err(ContractError::UnacceptableToken {});
    }

    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    match msg {
        ReceiveMsg::Stake { lock_type } => {
            let mut list = STAKERS
                .load(deps.storage, user_addr.clone())
                .unwrap_or(vec![]);

            list.push(StakerInfo {
                address: user_addr.clone(),
                amount: wrapper.amount,
                reward: Uint128::zero(),
                last_time: env.block.time.seconds(),
                lock_type: match lock_type {
                    0 => constants::ONE_MONTH_SECONDS,
                    1 => constants::SIX_MONTH_SECONDS,
                    2 => constants::ONE_YEAR_SECONDS,
                    _ => constants::TWO_YEAR_SECONDS,
                },
            });

            STAKERS.save(deps.storage, user_addr.clone(), &list)?;

            return Ok(Response::new().add_attributes(vec![
                attr("action", "stake"),
                attr("address", user_addr.clone()),
                attr("amount", wrapper.amount),
            ]));
        }
    }
}
///////////////////////////////////////////////////////// this func is called for calculating the reward amount  //////////////////////////////////
///
///         
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn update_reward(
    storage: &mut dyn Storage,
    env: Env,
    address: Addr,
) -> Result<Uint128, ContractError> {
    let mut exists = STAKERS.load(storage, address.clone()).unwrap_or(vec![]);
    let cfg = CONFIG.load(storage)?;
    let mut total_reward = Uint128::zero();

    for i in 0..exists.len() {
        let staked_time = env.block.time.seconds() - exists[i].last_time;
        let mut reward_tot = Uint128::zero();

        if staked_time < exists[i].lock_type {
            reward_tot = Uint128::zero();
        } else {
            match exists[i].lock_type {
                constants::TWO_YEAR_SECONDS => {
                    if staked_time >= constants::TWO_YEAR_SECONDS {
                        // 100% for over 2 years
                        reward_tot = Uint128::from(exists[i].amount);
                    }
                }
                constants::ONE_YEAR_SECONDS => {
                    if staked_time >= constants::ONE_YEAR_SECONDS {
                        // 40% for over 1 years
                        reward_tot = exists[i].amount * Uint128::from(constants::ONE_YEAR_APY)
                            / Uint128::from(MULTIPLE);
                    }
                }
                constants::SIX_MONTH_SECONDS => {
                    if staked_time >= constants::SIX_MONTH_SECONDS {
                        // 20% for over 6 months
                        reward_tot = exists[i].amount * Uint128::from(constants::SIX_MONTH_APY)
                            / Uint128::from(MULTIPLE);
                    }
                }
                _ => {
                    if staked_time >= constants::ONE_MONTH_SECONDS {
                        // 10% for over 30 days
                        reward_tot = exists[i].amount * Uint128::from(constants::ONE_MONTH_APY)
                            / Uint128::from(MULTIPLE);
                    }
                }
            }
        };
        let reward = reward_tot * (Uint128::from(cfg.reward_interval))
            / (Uint128::from(constants::ONE_YEAR_SECONDS));
        exists[i].reward = reward; //* Uint128::from(staked_time) / Uint128::from(cfg.reward_interval); //for test
        total_reward += exists[i].reward;
    }

    STAKERS.save(storage, address.clone(), &exists).unwrap();

    return Ok(total_reward);
}
///////////////////////////////////////////////////////// this func is called when we click reward button on frontend//////////////////////////////////
///
///         input params: customer's wallet address
///                       juno reward flag(This is true when rank is bigger than 500, if this is true, customer can get juno reward, if false, cutomer can get only hole reward)
///                       artists wallet percent
///                       burn wallet percent
///                       charity wallet percent
///                       my wallet percent = 100 - artists_percent - burn_percent - charity_percent
///
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn execute_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reward_distribution: RewardDistribution,
) -> Result<Response, ContractError> {
    check_enabled(&deps, &info)?;
    let cfg = CONFIG.load(deps.storage)?;

    let cw20_reward = update_reward(deps.storage, env.clone(), info.sender.clone()).unwrap();

    let mut list = STAKERS
        .load(deps.storage, info.sender.clone())
        .unwrap_or(vec![]);

    // // neet to change reward to ujuno
    // // test for 1/1000 juno
    // // let juno_reward = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
    // //     contract_addr: pool_address.clone().into(),
    // //     msg: to_binary(&WasmswapQueryMsg::Token2ForToken1Price {
    // //         token2_amount: amount
    // //     })?,
    // // }))?;
    let juno_reward = cw20_reward; //for test
    
    let tot_reward_token = util::get_token_amount(
        deps.querier,
        Denom::Native(cfg.reward_token_denom.clone()),
        env.contract.address.clone(),
    )?;

    if tot_reward_token < juno_reward {
        return Err(ContractError::NotEnoughReward {});
    }

    let mut msgs: Vec<CosmosMsg> = vec![];

    for i in 0..list.len() {
        list[i].last_time = env.block.time.seconds();
    }
    STAKERS.save(deps.storage, info.sender.clone(), &list)?;

    if !cw20_reward.is_zero() {
        msgs.push(util::transfer_token_message(
            Denom::Cw20(cfg.stake_token_address.clone()),
            cw20_reward,
            info.sender.clone(),
        )?);
    }

    if reward_distribution.juno_reward {
        if reward_distribution.charity != 0u64 {
            let reward_charity =
                juno_reward * Uint128::from(reward_distribution.charity) / Uint128::from(100u64);
            if !reward_charity.is_zero() {
                msgs.push(util::transfer_token_message(
                    Denom::Native(cfg.reward_token_denom.clone()),
                    reward_charity,
                    cfg.reward_charity_address.clone(),
                )?);
            }
        }

        if reward_distribution.burn != 0u64 {
            let reward_burn =
                juno_reward * Uint128::from(reward_distribution.burn) / Uint128::from(100u64);
            if !reward_burn.is_zero() {
                msgs.push(util::transfer_token_message(
                    Denom::Native(cfg.reward_token_denom.clone()),
                    reward_burn,
                    cfg.reward_burn_address.clone(),
                )?);
            }
        }

        if reward_distribution.artists != 0u64 {
            let reward_artists =
                juno_reward * Uint128::from(reward_distribution.artists) / Uint128::from(100u64);
            if !reward_artists.is_zero() {
                msgs.push(util::transfer_token_message(
                    Denom::Native(cfg.reward_token_denom.clone()),
                    reward_artists,
                    cfg.reward_artists_address.clone(),
                )?);
            }
        }

        if (reward_distribution.charity + reward_distribution.burn + reward_distribution.artists)
            < 100
        {
            let reward_user_rate = 100
                - (reward_distribution.charity
                    + reward_distribution.burn
                    + reward_distribution.artists);
            let reward_user = juno_reward * Uint128::from(reward_user_rate) / Uint128::from(100u64);
            if !reward_user.is_zero() {
                msgs.push(util::transfer_token_message(
                    Denom::Native(cfg.reward_token_denom.clone()),
                    reward_user,
                    info.sender.clone(),
                )?);
            }
        }
    }

    // End

    return Ok(Response::new().add_messages(msgs).add_attributes(vec![
        attr("action", "claim_reward"),
        attr("address", info.sender.clone()),
        attr("reward_amount", Uint128::from(cw20_reward)),
    ]));
}
///////////////////////////////////////////////////////// this func is called when we click unstake button on frontend//////////////////////////////////
///
///         Using this function, we can unstake all staked token
///         input params: none
///         
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn execute_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    check_enabled(&deps, &info)?;
    let cfg = CONFIG.load(deps.storage)?;

    let mut list = STAKERS.load(deps.storage, info.sender.clone())?;

    // check if user can unstake this record
    // env.block.time.seconds(), record.stake_time
    let mut staked = Uint128::zero();

    for i in 0..list.len() {
        staked += list[i].amount;
    }

    let tot_staked = util::get_token_amount(
        deps.querier,
        Denom::Cw20(cfg.stake_token_address.clone()),
        env.contract.address.clone(),
    )?;

    if tot_staked < staked {
        return Err(ContractError::NotEnoughStake {});
    }
    for j in 0..list.len() {
        list.remove(0);
    }

    STAKERS.save(deps.storage, info.sender.clone(), &list)?;

    let msg = util::transfer_token_message(
        Denom::Cw20(cfg.stake_token_address.clone()),
        staked,
        info.sender.clone(),
    )?;

    return Ok(Response::new().add_message(msg).add_attributes(vec![
        attr("action", "unstake"),
        attr("address", info.sender.clone()),
        attr("staked_amount", Uint128::from(staked)),
    ]));
}

///////////////////////////////////////////////////////// this func is called for checking ownership//////////////////////////////////
///
///         Owner is set when contract is instantiated.
///         Using this function, we can authorize the ownership
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn check_owner(deps: &DepsMut, info: &MessageInfo) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(Response::new().add_attribute("action", "check_owner"))
}
///////////////////////////////////////////////////////// this func is called for checking enable state//////////////////////////////////
///
///         Enable state is set when contract is instantiated.
///         The default vale is true.
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn check_enabled(deps: &DepsMut, info: &MessageInfo) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if !cfg.enabled {
        return Err(ContractError::Disabled {});
    }
    Ok(Response::new().add_attribute("action", "check_enabled"))
}
///////////////////////////////////////////////////////// this func is called for updating the ownership//////////////////////////////////
///
///         Owner is set when contract is instantiated.
///         if changing ownership is needed, we can use this function.
///         input params: new owner(new walletaddress)
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    owner: Addr,
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.owner = owner;
        Ok(exists)
    })?;
    Ok(Response::new().add_attribute("action", "update_owner"))
}
///////////////////////////////////////////////////////// this func is called for updating the enable state //////////////////////////////////
///
///         If we need changing the enable state of the contract, this function is used.
///         input params: new state(BOOL)
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn execute_update_enabled(
    deps: DepsMut,
    info: MessageInfo,
    enabled: bool,
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.enabled = enabled;
        Ok(exists)
    })?;
    Ok(Response::new().add_attribute("action", "update_enabled"))
}
///////////////////////////////////////////////////////// this func is called for updating reward interval //////////////////////////////////
///
///         If we need changing reward interval, this function is used.
///         input params: new reward_interval(u64)
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn execute_update_constants(
    deps: DepsMut,
    info: MessageInfo,
    reward_interval: u64,
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.reward_interval = reward_interval;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_constants"))
}
///////////////////////////////////////////////////////// this func is called for withdrawing reward //////////////////////////////////
///
///         If withdrawing the reward tokens is needed, this function is used.
///         Only owner can call this function
///         input pararms: the reward token amount of withdrawing
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn execute_withdraw_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    check_owner(&deps, &info)?;

    let mut cfg = CONFIG.load(deps.storage)?;

    let tot = util::get_token_amount(
        deps.querier,
        Denom::Native(cfg.reward_token_denom.clone()),
        env.contract.address.clone(),
    )?;

    if tot < amount {
        return Err(ContractError::NotEnoughReward {});
    }

    let msg = util::transfer_token_message(
        Denom::Native(cfg.reward_token_denom.clone()),
        amount,
        info.sender.clone(),
    )?;

    return Ok(Response::new().add_message(msg).add_attributes(vec![
        attr("action", "withdraw_reward"),
        attr("address", info.sender.clone()),
        attr("amount", amount),
    ]));
}
///////////////////////////////////////////////////////// this func is called for withdrawing the staked token //////////////////////////////////
///
///         Only owner can call this function
///         input pararms: the withdraw amount
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn execute_withdraw_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    check_owner(&deps, &info)?;

    let mut cfg = CONFIG.load(deps.storage)?;

    let tot = util::get_token_amount(
        deps.querier,
        Denom::Cw20(cfg.stake_token_address.clone()),
        env.contract.address.clone(),
    )?;

    if tot < amount {
        return Err(ContractError::NotEnoughStake {});
    }

    let msg = util::transfer_token_message(
        Denom::Cw20(cfg.stake_token_address.clone()),
        amount,
        info.sender.clone(),
    )?;

    return Ok(Response::new().add_message(msg).add_attributes(vec![
        attr("action", "withdraw_stake"),
        attr("address", info.sender.clone()),
        attr("amount", amount),
    ]));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Staker { address } => to_binary(&query_staker(deps, address)?),
        QueryMsg::ListStakers { start_after } => to_binary(&query_list_stakers(deps, start_after)?),
        QueryMsg::GetHoleAmount { address } => to_binary(&query_get_hole_amount(deps, address)?),
    }
}
///////////////////////////////////////////////////////// this func is called for getting the state of the contract  //////////////////////////////////
///
///         
///         Using this function, we can get the contract informatios such as owner, reward token denom, stake token address,
///         reward interval, artists, burn, charity address for reward, enable state.
///          
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner,
        reward_token_denom: cfg.reward_token_denom.into(),
        stake_token_address: cfg.stake_token_address.into(),
        reward_interval: cfg.reward_interval,
        reward_artists_address: cfg.reward_artists_address.into(),
        reward_burn_address: cfg.reward_burn_address.into(),
        reward_charity_address: cfg.reward_charity_address.into(),
        enabled: cfg.enabled,
    })
}
///////////////////////////////////////////////////////// this func is called for getting the hole token amout  //////////////////////////////////
///
///         
///         Using this function, we can get the whole amout of the hole token in the contract.
///         input params: contract address or wallet address
///     
/// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn query_get_hole_amount(deps: Deps, address: Addr) -> StdResult<TestBalanceResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    let total = util::get_token_amount(
        deps.querier,
        Denom::Cw20(cfg.stake_token_address.clone()),
        address.clone(),
    )
    .unwrap();
    Ok(TestBalanceResponse { balance: total })
}
///////////////////////////////////////////////////////// this func is called for getting the informations of stakers  //////////////////////////////////
///
///         
///         Using this function, we can get anybody's all staking informations.
///         input params: contract address or wallet address
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn query_staker(deps: Deps, address: Addr) -> StdResult<Vec<StakerInfo>> {
    let list = STAKERS
        .load(deps.storage, address.clone())
        .unwrap_or(vec![]);
    Ok(list)
}

fn map_staker(item: StdResult<(Addr, Vec<StakerInfo>)>) -> StdResult<Vec<StakerInfo>> {
    item.map(|(_id, record)| record)
}
///////////////////////////////////////////////////////// this func is called for getting the informations of all stakers  //////////////////////////////////
///
///         
///         Using this function, we can get all staking informations for all stakers.
///         input params: start wallet address for getting the list of stakers.
///     
/// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
fn query_list_stakers(deps: Deps, start_after: Option<String>) -> StdResult<StakerListResponse> {
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.map(|addr| Bound::exclusive(addr.clone()));

    let stakers: StdResult<Vec<Vec<_>>> = STAKERS
        .range(deps.storage, start, None, Order::Ascending)
        .map(|item| map_staker(item))
        .collect();

    Ok(StakerListResponse { stakers: stakers? })
}
///////////////////////////////////////////////////////// this func is called for migration of the contract  //////////////////////////////////
///
///         
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
