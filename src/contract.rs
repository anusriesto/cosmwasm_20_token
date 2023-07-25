#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use cw2::set_contract_version;
use cw20::{
    BalanceResponse, Cw20Coin, Cw20ReceiveMsg, DownloadLogoResponse, EmbeddedLogo, Logo, LogoInfo,
    MarketingInfoResponse, MinterResponse, TokenInfoResponse,
};



use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    MinterData, TokenInfo, ALLOWANCES, ALLOWANCES_SPENDER, BALANCES, LOGO, MARKETING_INFO,
    TOKEN_INFO,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const LOGO_SIZE_CAP: usize = 5 * 1024;






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
    // validate_accounts(accounts)?;

    let mut total_supply = Uint128::zero();
    for row in accounts {
        let address = deps.api.addr_validate(&row.address)?;
        BALANCES.save(deps.storage, &address, &row.amount)?;
        total_supply += row.amount;
    }

    Ok(total_supply)
}

// pub fn validate_accounts(accounts: &[Cw20Coin]) -> Result<(),StdError> {
//     let mut addresses = accounts.iter().map(|c| &c.address).collect::<Vec<_>>();
//     addresses.sort();
//     addresses.dedup();

// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
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

    let res = Response::new()
        .add_attribute("action", "mint")
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
    use cosmwasm_std::{coins,Addr,from_binary,CosmosMsg,StdError,WasmMsg,Uint128};
    fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
        query_balance(deps, address.into()).unwrap().balance}

    fn do_instantiate_with_minter(
        deps: DepsMut,
        addr: &str,
        amount: Uint128,
        minter: &str,
        cap: Option<Uint128>,
    ) -> TokenInfoResponse {
        _do_instantiate(
            deps,
            addr,
            amount,
            Some(MinterResponse {
                minter: minter.to_string(),
                cap,
            }),
        )
    }

    // this will set up the instantiation for other tests
    fn do_instantiate(deps: DepsMut, addr: &str, amount: Uint128) -> TokenInfoResponse {
        _do_instantiate(deps, addr, amount, None)
    }

    // this will set up the instantiation for other tests
    fn _do_instantiate(
        mut deps: DepsMut,
        addr: &str,
        amount: Uint128,
        mint: Option<MinterResponse>,
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

    const PNG_HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];



    #[test]
    fn minting(){
        let mut deps = mock_dependencies();
            let amount = Uint128::new(11223344);
            let minter = String::from("asmodat");
            let limit = Uint128::new(511223344);
            let instantiate_msg = InstantiateMsg {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
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
                    name: "Cash Token".to_string(),
                    symbol: "CASH".to_string(),
                    decimals: 9,
                    total_supply: amount,
                }
            );
            assert_eq!(
                get_balance(deps.as_ref(), "addr0000"),
                Uint128::new(11223344)
            );
        
        

    }
}