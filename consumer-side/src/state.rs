use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

/// Storage key for this contract's configuration.
pub static CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub secret_vrf_contract_address: String,
    pub secret_vrf_verification_key: String,
    pub secret_transfer_channel_id: String,
    pub chain_transfer_channel_id: String,
    pub secret_vrf_decoded: Vec<u8>,
}

pub static LAST: Item<String> = Item::new("last_random");
