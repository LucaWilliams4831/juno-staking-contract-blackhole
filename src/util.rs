use crate::error::ContractError;
use crate::state::CONFIG;
use cosmwasm_std::{
    to_binary, Addr, BalanceResponse as NativeBalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg,
    QuerierWrapper, QueryRequest, Response, StdResult, Storage, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, BalanceResponse as CW20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Denom};
// use stockpool::msg::{ExecuteMsg as WasmswapExecuteMsg, QueryMsg as WasmswapQueryMsg, Token1ForToken2PriceResponse, Token2ForToken1PriceResponse, InfoResponse as WasmswapInfoResponse, TokenSelect};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const NORMAL_DECIMAL: u128 = 1000000u128;
pub const THRESHOLD: u128 = 3000000u128;

pub fn transfer_native_token_message(
    denom: Denom,
    amount: Uint128,
    receiver: Addr,
) -> Result<CosmosMsg, ContractError> {
    match denom.clone() {
        Denom::Native(native_str) => {
            return Ok(BankMsg::Send {
                to_address: receiver.clone().into(),
                amount: vec![Coin{
                    denom: native_str,
                    amount
                }]
            }.into());
        },
        Denom::Cw20(cw20_address) => {
            return Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_address.clone().into(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: receiver.clone().into(),
                    amount
                })?,
            }));
        }
    }
}

pub fn transfer_token_message(
    denom: Denom,
    amount: Uint128,
    receiver: Addr,
) -> Result<CosmosMsg, ContractError> {
    match denom.clone() {
        Denom::Native(native_str) => {
            return Ok(BankMsg::Send {
                to_address: receiver.clone().into(),
                amount: vec![Coin {
                    denom: native_str,
                    amount,
                }],
            }
            .into());
        }
        Denom::Cw20(cw20_address) => {
            return Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_address.clone().into(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: receiver.clone().into(),
                    amount,
                })?,
            }));
        }
    }
}

pub fn get_token_amount(
    querier: QuerierWrapper,
    denom: Denom,
    contract_addr: Addr,
) -> Result<Uint128, ContractError> {
    match denom.clone() {
        Denom::Native(native_str) => {
            let native_response: NativeBalanceResponse =
                querier.query(&QueryRequest::Bank(BankQuery::Balance {
                    address: contract_addr.clone().into(),
                    denom: native_str,
                }))?;
            return Ok(native_response.amount.amount);
        }
        Denom::Cw20(cw20_address) => {
            let balance_response: CW20BalanceResponse =
                querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: cw20_address.clone().into(),
                    msg: to_binary(&Cw20QueryMsg::Balance {
                        address: contract_addr.clone().into(),
                    })?,
                }))?;
            return Ok(balance_response.balance);
        }
    }
}
