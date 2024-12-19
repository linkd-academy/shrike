use serde::Deserialize;

use super::models::{AppLogResult, BlockResult, NeoParam};

pub trait RpcMethod {
    type ReturnType: for<'de> Deserialize<'de>;

    fn method_name(&self) -> &'static str;
    fn params(&self) -> Vec<NeoParam>;
}

pub struct GetBlockCount;

impl RpcMethod for GetBlockCount {
    type ReturnType = u64;

    fn method_name(&self) -> &'static str {
        "getblockcount"
    }

    fn params(&self) -> Vec<NeoParam> {
        vec![]
    }
}

pub struct GetBlock {
    pub block_height: u64,
    pub verbosity: u8,
}

impl RpcMethod for GetBlock {
    type ReturnType = BlockResult;

    fn method_name(&self) -> &'static str {
        "getblock"
    }

    fn params(&self) -> Vec<NeoParam> {
        vec![
            NeoParam::Integer(self.block_height),
            NeoParam::Integer(u64::from(self.verbosity)),
        ]
    }
}

pub struct GetApplicationLog {
    pub hash: String,
}

impl RpcMethod for GetApplicationLog {
    type ReturnType = AppLogResult;

    fn method_name(&self) -> &'static str {
        "getapplicationlog"
    }

    fn params(&self) -> Vec<NeoParam> {
        vec![NeoParam::String(self.hash.clone())]
    }
}

pub struct InvokeFunctionHistoric {
    pub state_root_or_block: u64,
    pub script_hash: String,
    pub operation: String,
    pub args: Vec<NeoParam>,
}

impl RpcMethod for InvokeFunctionHistoric {
    type ReturnType = serde_json::Value; // Ajuste conforme o tipo esperado do RPC

    fn method_name(&self) -> &'static str {
        "invokefunctionhistoric"
    }

    fn params(&self) -> Vec<NeoParam> {
        vec![
            NeoParam::Integer(self.state_root_or_block),
            NeoParam::String(self.script_hash.clone()),
            NeoParam::String(self.operation.clone()),
            NeoParam::Array(self.args.clone()),
        ]
    }
}

pub struct GetBlockHistoric {
    pub block_height: u64,
}

impl RpcMethod for GetBlockHistoric {
    type ReturnType = BlockResult;

    fn method_name(&self) -> &'static str {
        "getblockhistoric"
    }

    fn params(&self) -> Vec<NeoParam> {
        vec![NeoParam::Integer(self.block_height)]
    }
}

pub struct InvokeFunction {
    pub script_hash: String,
    pub operation: String,
    pub args: Vec<NeoParam>,
}

impl RpcMethod for InvokeFunction {
    type ReturnType = serde_json::Value; // Retorno genérico em JSON

    fn method_name(&self) -> &'static str {
        "invokefunction"
    }

    fn params(&self) -> Vec<NeoParam> {
        vec![
            NeoParam::String(self.script_hash.clone()),
            NeoParam::String(self.operation.clone()),
            NeoParam::Array(self.args.clone()), // Aqui estamos serializando os argumentos
        ]
    }
}
