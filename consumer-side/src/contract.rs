use anybuf::Anybuf;
use base64::{Engine as _, engine::general_purpose};
use cosmwasm_std::{
    Binary, CosmosMsg::Stargate, Deps, DepsMut, entry_point, Env, MessageInfo, Response,
    StdError, StdResult, to_json_binary,
};
use sha2::{Digest, Sha256};

use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, LAST};

//For this demo, these values are provided as hardcoded consts.
//You can also store these values into the storage if needed
//const SECRET_VRF_CONTRACT_ADDRESS: &str = "secret1up0mymn4993hgn7zpzu4je34w0n5s7l0mem7rk";
//const SECRET_VRF_VERIFICATION_KEY: &str = "BClOY6gcGjBCqeaFskrg0VIzptmyftgfY329GcZOvr3/eH/C4pJ4nH6ch6W/gjog8UErnEpIbMUOmElayUOxDas=";

//Juno
//const SECRET_TRANSFER_CHANNEL_ID: &str = "channel-8";
//const CHAIN_TRANSFER_CHANNEL_ID: &str = "channel-48";

//Archway
//const SECRET_TRANSFER_CHANNEL_ID: &str = "channel-84";
//const CHAIN_TRANSFER_CHANNEL_ID: &str = "channel-21";

// Instantiate entry point
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let vrf_decoded = general_purpose::STANDARD
        .decode(msg.secret_vrf_verification_key.clone())
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    CONFIG.save(
        deps.storage,
        &Config {
            chain_transfer_channel_id: msg.chain_transfer_channel_id,
            secret_transfer_channel_id: msg.secret_transfer_channel_id,
            secret_vrf_contract_address: msg.secret_vrf_contract_address,
            secret_vrf_verification_key: msg.secret_vrf_verification_key,
            secret_vrf_decoded: vrf_decoded,
        },
    )?;
    LAST.save(deps.storage, &"None".to_string())?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RequestRandom { job_id } => request_random(deps, env, info, job_id),
        ExecuteMsg::ReceiveRandom {
            job_id,
            randomness,
            signature,
        } => receive_random(deps, env, job_id, randomness, signature),
    }
}

fn receive_random(
    deps: DepsMut,
    _env: Env,
    job_id: String,
    randomness: String,
    signature: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    //check if the randomness is correct and wasn't manipulated during transit
    //using deps.api is fine for this example here, but this might change with future versions of cosmwasm, careful.
    // Create a new Sha256 hasher instance to hash the input data for verfication
    let mut hasher = Sha256::new();
    hasher.update([job_id.clone(), randomness.clone()].concat().as_bytes());
    let hash_result = hasher.finalize();
    let signature_decoded = general_purpose::STANDARD
        .decode(signature)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let signature_correct = deps
        .api
        .secp256k1_verify(&hash_result, &signature_decoded, &config.secret_vrf_decoded)
        .map_err(|err| StdError::generic_err(err.to_string()))?;
    if !signature_correct {
        return Err(StdError::generic_err(
            "Could not verify Secret VRF signature",
        ));
    }

    //do whatever computation you need to do

    LAST.save(deps.storage, &randomness)?;
    Ok(Response::default()
        .add_attribute("random", "successfull")
        .add_attribute("randomness", randomness))
}

fn request_random(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    job_id: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    //do your preparation for requesting a random number here

    //create the IBC Hook memo that will be executed by Secret Network
    let ibc_callback_hook_memo = format!(
        "{{\"wasm\": {{\"contract\": \"{}\", \"msg\": {{\"request_random\": {{\"job_id\": \"{}\", \"num_words\": \"1\", \"callback_channel_id\": \"{}\", \"callback_to_address\": \"{}\", \"timeout_sec_from_now\": \"{}\"}}}}}}}}",
        config.secret_vrf_contract_address, // Secret VRF Contract address on Secret Network
        job_id,
        config.secret_transfer_channel_id, // IBC Channel on the Secret Network side to send it back
        env.contract.address,
        "900" //IBC callback timeout, here 900s = 15 min
    );

    // Construct a CosmosMsg::Stargate message with the serialized IBC Transfer Data
    let msg = Stargate {
        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
        value: Anybuf::new()
            .append_string(1, "transfer") // source port
            .append_string(2, config.chain_transfer_channel_id) // source channel (IBC Channel on your network side)
            .append_message(
                3,
                &Anybuf::new()
                    .append_string(1, info.funds[0].denom.clone())
                    .append_string(2, info.funds[0].amount.to_string()),
            ) // Token
            .append_string(4, env.contract.address) // sender
            .append_string(5, config.secret_vrf_contract_address) // receiver
            .append_message(6, &Anybuf::new().append_uint64(1, 0).append_uint64(2, 0)) // TimeoutHeight
            .append_uint64(7, env.block.time.plus_seconds(900).nanos()) // TimeoutTimestamp, here 900s = 15 min
            .append_string(8, ibc_callback_hook_memo)
            .into_vec()
            .into(),
    };

    // Return the response with the Secret VRF IBC message added to it
    Ok(Response::new().add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Last {} => to_json_binary(&query_last(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        secret_vrf_contract_address: config.secret_vrf_contract_address,
        secret_vrf_verification_key: config.secret_vrf_verification_key,
        secret_transfer_channel_id: config.secret_transfer_channel_id,
        chain_transfer_channel_id: config.chain_transfer_channel_id,
    })
}

pub fn query_last(deps: Deps) -> StdResult<String> {
    let last = LAST.load(deps.storage)?;
    Ok(last)
}
