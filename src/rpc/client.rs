use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use reth_primitives::{BlockNumber, H256, U256, U64};
use reth_rpc_types::{RichBlock, TransactionReceipt};

#[rpc(server, client)]
#[async_trait]
pub trait EthApi {
    #[method(name = "eth_chainId")]
    async fn chain_id(&self) -> RpcResult<U64>;

    #[method(name = "eth_blockNumber")]
    fn block_number(&self) -> RpcResult<U256>;

    #[method(name = "eth_getBlockByNumber")]
    async fn block_by_number(
        &self,
        number: BlockNumber,
        full: bool,
    ) -> RpcResult<Option<RichBlock>>;

    /// Returns the receipt of a transaction by transaction hash.
    #[method(name = "eth_getTransactionReceipt")]
    async fn transaction_receipt(&self, hash: H256) -> RpcResult<Option<TransactionReceipt>>;
}
