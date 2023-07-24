use cosmwasm_std::{Addr, StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Serialize,Deserialize,Clone,PartialEq,JsonSchema)]
pub struct InitialBalance {
    pub address:Addr,
    pub amount:Uint128,
}

#[derive(Serialize,Deserialize,JsonSchema)]
pub struct InitMsg {
    pub name: String,
    pub symbol:String,
    pub decimals:u8,
    pub initial_balances:Vec<InitialBalance>,
    pub minter:Option<Addr>,
    pub cap:Option<Uint128>

}

impl InitMsg {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.minter.as_ref().and_then(|v| v.cap)
    }

    pub fn validate(&self) -> StdResult<()> {
        if self.decimals > 18 {
            return Err(StdError::generic_err("Decimals must not exceed 18"));
        }
        Ok(())
    }

}