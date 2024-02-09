use cosmwasm_schema::{cw_serde};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RequestRandom, { job_id ; num_words }
    ReceiveRandom, { job_id; result; signature} 
}