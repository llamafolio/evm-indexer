use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use reth_primitives::{U256, U64};

#[rpc(server, client)]
#[async_trait]
pub trait EthApi {
    #[method(name = "eth_chainId")]
    async fn chain_id(&self) -> RpcResult<U64>;

    #[method(name = "eth_blockNumber")]
    fn block_number(&self) -> RpcResult<U256>;
}
