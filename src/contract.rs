#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use cw2::set_contract_version;
use cw20::{
    BalanceResponse, Cw20Coin,AllowanceResponse, Cw20ReceiveMsg, Expiration,
     MinterResponse, TokenInfoResponse,
};



use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    MinterData, TokenInfo, ALLOWANCES, ALLOWANCES_SPENDER, BALANCES,
    TOKEN_INFO,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");






//Initialise the account
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // check valid token info
    msg.validate()?;
    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg.initial_balances)?;
    
    //checking whether the initial supply is greater than market cap
    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;
    

    Ok(Response::default())
}
pub fn create_accounts(
    deps: &mut DepsMut,
    accounts: &[Cw20Coin],
) -> Result<Uint128, ContractError> {
    

    let mut total_supply = Uint128::zero();
    for row in accounts {
        let address = deps.api.addr_validate(&row.address)?;
        BALANCES.save(deps.storage, &address, &row.amount)?;
        total_supply += row.amount;
    }

    Ok(total_supply)
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
    //expires:Option<Expiration>,
) -> Result<Response, ContractError> {
    let mut config = TOKEN_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if config
        .mint
        .as_ref()
        .ok_or(ContractError::Unauthorized {})?
        .minter
        != info.sender
    {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    config.total_supply += amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &config)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    // let update_fn = |allow: Option<AllowanceResponse>| -> Result<_, _> {
    //     let mut val = allow.unwrap_or_default();
    //     if let Some(exp) = expires {
    //         if exp.is_expired(&_env.block) {
    //             return Err(ContractError::InvalidExpiration {});
    //         }
    //         val.expires = exp;
    //     }
    //     val.allowance =amount;
    //     Ok(val)
    // };
    // ALLOWANCES.update(deps.storage, (&rcpt_addr,&rcpt_addr), update_fn)?;
    // ALLOWANCES_SPENDER.update(deps.storage, (&rcpt_addr, &rcpt_addr), update_fn)?;

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("action", "expires")
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let info = TOKEN_INFO.load(deps.storage)?;
    let res = TokenInfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
    };
    Ok(res)
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}



#[cfg(test)]
mod tests{
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies,mock_env,mock_info};
    use cw20::{Expiration};
    use cosmwasm_std::{coins,Addr,from_binary,CosmosMsg,StdError,WasmMsg,Uint128};
    fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
        query_balance(deps, address.into()).unwrap().balance}

    fn do_instantiate_with_minter(
        deps: DepsMut,
        addr: &str,
        amount: Uint128,
        minter: &str,
        cap: Option<Uint128>,
        expires:Option<Expiration>
    ) -> TokenInfoResponse {
        _do_instantiate(
            deps,
            addr,
            amount,
            Some(MinterResponse {
                minter: minter.to_string(),
                cap,
            }),
        //expires,
    
        )
    }

    // this will set up the instantiation for other tests
    fn do_instantiate(deps: DepsMut, addr: &str, amount: Uint128,expires:Option<Expiration>) -> TokenInfoResponse {
        _do_instantiate(deps, addr, amount, None)
    }

    // this will set up the instantiation for other tests
    fn _do_instantiate(
        mut deps: DepsMut,
        addr: &str,
        amount: Uint128,
        mint: Option<MinterResponse>,
        //expires:Option<Expiration>,
    ) -> TokenInfoResponse {
        let instantiate_msg = InstantiateMsg {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: addr.to_string(),
                amount,
        
            }],
            mint: mint.clone(),
            marketing: None,
            get_cap:Some(amount),
            //expires:expires.clone(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let meta = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(
            meta,
            TokenInfoResponse {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                total_supply: amount,
            }
        );
        assert_eq!(get_balance(deps.as_ref(), addr), amount);
        meta
    }

    



    // #[test]
    // fn minting(){
    //     let mut deps = mock_dependencies();
    //     let info = mock_info("creator", &[]);
    //     let env = mock_env();
    //         let amount = Uint128::new(1000000);
    //         let minter = String::from("anuj");
    //         let limit = Uint128::new(10000000000);
    //         let expired = Expiration::AtTime(env.block.time.plus_seconds(86400));
    //         let instantiate_msg = InstantiateMsg {
    //             name: "Mettalex".to_string(),
    //             symbol: "W".to_string(),
    //             decimals: 9,
    //             initial_balances: vec![Cw20Coin {
    //                 address: "addr000".into(),
    //                 amount,
    //             }],
    //             mint: Some(MinterResponse {
    //                 minter: minter.clone(),
    //                 cap: Some(limit),
    //             }),
    //             marketing: None,
    //             get_cap:Some(limit),
    //             //expires:Some(expired),
    //         };
            
    //         let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    //         //assert_eq!(0, res.messages.len());

    //         assert_eq!(
    //             query_token_info(deps.as_ref()).unwrap(),
    //             TokenInfoResponse {
    //                 name: "Metallex".to_string(),
    //                 symbol: "W".to_string(),
    //                 decimals: 9,
    //                 total_supply: amount,
    //             }
    //         );
    //         assert_eq!(
    //             get_balance(deps.as_ref(), "addr000"),
    //             Uint128::new(1000000)
    //         );
        
        

    // }


    #[test]
    fn minting(){
        let mut deps = mock_dependencies();
            let amount = Uint128::new(1000000);
            let minter = String::from("asmodat");
            let limit = Uint128::new(1000000000);
            let instantiate_msg = InstantiateMsg {
                name: "Metallex Token".to_string(),
                symbol: "META".to_string(),
                decimals: 9,
                initial_balances: vec![Cw20Coin {
                    address: "addr0000".into(),
                    amount,
                }],
                mint: Some(MinterResponse {
                    minter: minter.clone(),
                    cap: Some(limit),
                }),
                marketing: None,
                get_cap:Some(limit)
            };
            let info = mock_info("creator", &[]);
            let env = mock_env();
            let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
            assert_eq!(0, res.messages.len());

            assert_eq!(
                query_token_info(deps.as_ref()).unwrap(),
                TokenInfoResponse {
                    name: "Metallex Token".to_string(),
                    symbol: "META".to_string(),
                    decimals: 9,
                    total_supply: amount,
                }
            );
            assert_eq!(
                get_balance(deps.as_ref(), "addr0000"),
                Uint128::new(1000000)
            );
        
        

    }
}