use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub secret_vrf_contract_address: String,
    pub secret_vrf_verification_key: String,
    pub secret_transfer_channel_id: String,
    pub chain_transfer_channel_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RequestRandom {
        job_id: String,
    },
    ReceiveRandom {
        job_id: String,
        randomness: String,
        signature: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Last {}
}

#[cw_serde]
pub struct ConfigResponse {
    pub secret_vrf_contract_address: String,
    pub secret_vrf_verification_key: String,
    pub secret_transfer_channel_id: String,
    pub chain_transfer_channel_id: String,
}
