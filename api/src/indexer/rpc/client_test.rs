use crate::indexer::rpc::client::Client;
use crate::indexer::rpc::models::{BlockAppLogResult, TransactionAppLogResult, TransactionResult};
use crate::shared::neo::address_to_hash160;
use tokio::runtime::Runtime;

#[test]
fn test_get_current_height_integration() {
    let client = Client::new();
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(client.get_current_height()).unwrap();
    assert!(result > 0);
}

#[test]
fn test_get_block_integration() {
    let client = Client::new();
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(client.get_block(1000)).unwrap();

    assert_eq!(
        result.hash,
        "0xe31ad93809a2ac112b066e50a72ad4883cf9f94a155a7dea2f05e69417b2b9aa"
    );
    assert_eq!(result.index, 1000);
    assert_eq!(result.size, 697);
    assert_eq!(result.version, 0);
    assert_eq!(
        result.merkleroot,
        "0x0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert_eq!(
        result.previousblockhash,
        "0xd340930d329ba06ae3982631fdd6aeaaa25b8b27d50f9283d42397212a88747e"
    );
}

#[test]
fn test_fetch_full_block_integration() {
    let client = Client::new();
    let rt = Runtime::new().unwrap();
    let (_, app_log): (_, BlockAppLogResult) = rt.block_on(client.fetch_full_block(1000)).unwrap();

    assert_eq!(
        app_log.blockhash,
        "0xe31ad93809a2ac112b066e50a72ad4883cf9f94a155a7dea2f05e69417b2b9aa"
    );
    assert_eq!(app_log.executions.len(), 2);

    // Verificar primeira execução (OnPersist)
    let on_persist = &app_log.executions[0];
    assert_eq!(on_persist.trigger, "OnPersist");
    assert_eq!(on_persist.vmstate, "HALT");
    assert_eq!(on_persist.gasconsumed, "0");
    assert!(on_persist.notifications.is_empty());

    // Verificar segunda execução (PostPersist)
    let post_persist = &app_log.executions[1];
    assert_eq!(post_persist.trigger, "PostPersist");
    assert_eq!(post_persist.vmstate, "HALT");
    assert_eq!(post_persist.gasconsumed, "0");
    assert_eq!(post_persist.notifications.len(), 1);

    // Verificar notificação de Transfer
    let transfer = &post_persist.notifications[0];
    assert_eq!(
        transfer.contract,
        "0xd2a4cff31913016155e38e474a2c06d08be276cf"
    );
    assert_eq!(transfer.eventname, "Transfer");
}

#[test]
fn test_get_balance_of_historic_integration() {
    let client = Client::new();
    let rt = Runtime::new().unwrap();
    let address = address_to_hash160("NV96QgerjXNmu4jLdMW4ZWkhySVMYX52Ex");
    let result = rt
        .block_on(client.get_balance_of_historic(
            90387,
            "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5",
            &address,
        ))
        .unwrap();
    assert_eq!(result.state, "HALT");
    assert_eq!(
        result.stack[0].value.as_ref().unwrap().to_string(),
        "\"276010\""
    );
}

#[test]
fn test_get_candidates_of_historic_integration() {
    let client = Client::new();
    let rt = Runtime::new().unwrap();
    let result = rt
        .block_on(client.get_candidates_of_historic(673628))
        .unwrap();

    assert_eq!(result.state, "HALT");
    assert_eq!(result.gasconsumed, "12681318");
    assert!(result.notifications.is_empty());

    // Verificar array de candidatos
    let candidates = &result.stack[0].value.as_ref().unwrap().as_array().unwrap();
    assert_eq!(candidates.len(), 25); // Total de candidatos

    // Verificar primeiro candidato
    let first = &candidates[0];
    let first_data = first["value"].as_array().unwrap();
    assert_eq!(first_data[0]["type"].as_str().unwrap(), "ByteString");
    assert_eq!(
        first_data[0]["value"].as_str().unwrap(),
        "AiNzCaBjP/kw1RhW2wHRfIKaWy5cwmOOnAO0z6jpyflx"
    );
    assert_eq!(first_data[1]["type"].as_str().unwrap(), "Integer");
    assert_eq!(first_data[1]["value"].as_str().unwrap(), "632733");

    // Verificar último candidato
    let last = &candidates[24];
    let last_data = last["value"].as_array().unwrap();
    assert_eq!(last_data[0]["type"].as_str().unwrap(), "ByteString");
    assert_eq!(
        last_data[0]["value"].as_str().unwrap(),
        "A/0E3pg/TgTJYpqzz8g/Qb50Mblr+FKpGHPDjKj3N+4s"
    );
    assert_eq!(last_data[1]["type"].as_str().unwrap(), "Integer");
    assert_eq!(last_data[1]["value"].as_str().unwrap(), "540732");
}
