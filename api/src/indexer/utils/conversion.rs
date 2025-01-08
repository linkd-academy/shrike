use crate::indexer::rpc::client::Client;

use crate::shared::neo::{
    address_to_hash160, base64_to_address, base64_to_hex, base64_to_script_hash, hex_decode,
    hex_to_base64, neo3_disassemble,
};
use serde_json::to_string;

use crate::block::models::Block;
use crate::history::models::DailyAddressBalance;
use crate::indexer::rpc::models::{
    BlockAppLogResult, BlockResult, ClientError, Contract, TransactionAppLogResult,
    TransactionResult,
};
use crate::transaction::models::{Notification, Transaction};

pub fn convert_block_result(r: BlockResult, a: &BlockAppLogResult) -> Block {
    let block_reward = &a.executions[1].notifications[0].state.value[2].value;
    let block_receiver = &a.executions[1].notifications[0].state.value[1].value;

    let reward_string = block_reward.clone().unwrap().as_str().unwrap().to_string();
    let reward = reward_string.parse::<u64>().unwrap();
    let reward_as_float = reward as f64 / 100_000_000_f64;

    let receiver = serde_json::to_string(block_receiver).unwrap();
    let stripped = &receiver[1..29];
    let address = base64_to_address(stripped);

    Block {
        index: r.index,
        hash: r.hash,
        size: r.size,
        version: r.version,
        merkle_root: r.merkleroot,
        time: r.time,
        nonce: r.nonce,
        speaker: r.primary,
        next_consensus: r.nextconsensus,
        reward: reward_as_float,
        reward_receiver: address,
        witnesses: r.witnesses,
    }
}

pub fn convert_transaction_result(
    t: TransactionResult,
    a: &TransactionAppLogResult,
    block_height: u64,
) -> Transaction {
    let state = &a.executions[0].vmstate;
    let stack = &a.executions[0].stack;
    let notifs = &a.executions[0].notifications;

    Transaction {
        index: 0,
        hash: t.hash,
        block_index: block_height,
        timestamp: t.timestamp,
        vm_state: state.to_string(),
        size: t.size,
        version: t.version,
        nonce: t.nonce,
        sender: t.sender,
        sysfee: t.sysfee,
        netfee: t.netfee,
        valid_until: t.validuntilblock,
        signers: t.signers,
        script: base64_to_hex(&t.script),
        witnesses: t.witnesses,
        stack_result: to_string(&stack).unwrap(),
        notifications: notifs.clone(),
    }
}

pub fn convert_contract_result(
    script: String,
    notifications: Vec<Notification>,
    block_height: u64,
) -> Vec<Contract> {
    let mut contracts = Vec::new();

    for notification in notifications {
        if notification.eventname == "Deploy"
            && notification.contract == "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd"
        {
            let full_disassembled_script = neo3_disassemble(&hex_to_base64(&script));
            let disassembled_script: Vec<&str> = full_disassembled_script.split("\n").collect();

            let mut contract_supported_standard: String = "[]".to_string();

            if let Some(data) = disassembled_script
                .iter()
                .find(|&s| s.contains("PUSHDATA2"))
            {
                let parts: Vec<&str> = data.split_whitespace().collect();
                let metadata_hex = parts.get(1).unwrap_or(&"");
                let metadata_hex_decoded = hex_decode(metadata_hex);
                let metadata = String::from_utf8(metadata_hex_decoded).unwrap();

                if metadata.starts_with("{") {
                    let metadata_json: serde_json::Value = serde_json::from_str(&metadata).unwrap();

                    contract_supported_standard = metadata_json["supportedstandards"].to_string();
                }
            }

            let contract_hash_base64 = notification.state.value[0]
                .value
                .as_ref()
                .unwrap()
                .as_str()
                .expect("Value is not a string");

            let contract_script_hash = base64_to_script_hash(contract_hash_base64);

            contracts.push(Contract {
                block_index: block_height,
                hash: contract_script_hash,
                contract_type: contract_supported_standard,
            });
        }
    }

    return contracts;
}

pub async fn convert_address_result(
    notifications: Vec<Notification>,
    block_height: u64,
    timestamp: u64,
    client: &Client,
) -> Result<Vec<DailyAddressBalance>, ClientError> {
    let mut addresses = Vec::new();

    for notification in notifications {
        if notification.eventname == "Transfer" {
            let state = notification.state.clone();
            let token = notification.contract.as_str();

            let sender_type = state.value[0]._type.as_str();
            let recipient_type = state.value[1]._type.as_str();

            if sender_type == "ByteString" && recipient_type == "ByteString" {
                let sender_base64 = notification.state.value[0]
                    .value
                    .as_ref()
                    .unwrap()
                    .as_str()
                    .expect("Value is not a string");

                let recipient_base64 = notification.state.value[1]
                    .value
                    .as_ref()
                    .unwrap()
                    .as_str()
                    .expect("Value is not a string");

                let sender_address = base64_to_address(sender_base64);
                let recipient_address = base64_to_address(recipient_base64);

                let sender_hash160 = address_to_hash160(&sender_address);
                let sender_response = client
                    .get_balance_of_historic(block_height, token, &sender_hash160)
                    .await?;
                let sender_balance: i64 = sender_response
                    .stack
                    .get(0) // Obtém o primeiro elemento do stack
                    .and_then(|entry| entry.value.as_ref()) // Acessa o Option<serde_json::Value>
                    .and_then(|val| {
                        val.as_str()
                            .and_then(|s| s.parse::<i64>().ok()) // Tenta converter string para i64
                            .or_else(|| val.as_i64()) // Ou usa diretamente como i64, se aplicável
                    })
                    .unwrap_or(0); // Valor padrão caso falhe

                let receiver_hash160 = address_to_hash160(&recipient_address);
                let receiver_response = client
                    .get_balance_of_historic(block_height, token, &receiver_hash160)
                    .await?;
                let receiver_balance: i64 = receiver_response
                    .stack
                    .get(0) // Obtém o primeiro elemento do stack
                    .and_then(|entry| entry.value.as_ref()) // Acessa o Option<serde_json::Value>
                    .and_then(|val| {
                        val.as_str()
                            .and_then(|s| s.parse::<i64>().ok()) // Tenta converter string para i64
                            .or_else(|| val.as_i64()) // Ou usa diretamente como i64, se aplicável
                    })
                    .unwrap_or(0); // Valor padrão caso falhe

                addresses.push(DailyAddressBalance {
                    block_index: block_height,
                    timestamp: timestamp,
                    date: "".to_string(),
                    address: sender_address.clone(),
                    token_contract: token.to_string(),
                    balance: sender_balance,
                });

                addresses.push(DailyAddressBalance {
                    block_index: block_height,
                    timestamp: timestamp,
                    date: "".to_string(),
                    address: recipient_address.clone(),
                    token_contract: token.to_string(),
                    balance: receiver_balance,
                });
            }
        }
    }

    Ok(addresses)
}
