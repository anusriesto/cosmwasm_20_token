use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr,Storage,Uint128, CanonicalAddr};
use cw_storage_plus::{Item};
use cosmwasm_storage::{Bucket,bucket,bucket_read,singleton_read,singleton,ReadonlyBucket,ReadonlySingleton,Singleton};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Meta {
    pub name: String,
    pub symbol:String,
    pub decimals:u8,
    pub total_supply:Uint128,
    pub minter:Option<CanonicalAddr>,
    pub cap:Option<Uint128>
}


const META_KEY:&[u8]=b"meta";
pub const CONFIG: Item<Config> = Item::new("config");
pub const MINTED_TOKENS: Item<Vec<String>> = Item::new("minted_tokens");

