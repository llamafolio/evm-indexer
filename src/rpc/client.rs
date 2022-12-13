use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use reth_primitives::{
    rpc::{Block, Transaction, TransactionReceipt},
    BlockNumber, H256, U256,
};

#[rpc(server, client)]
#[async_trait]
pub trait EthApi {
    #[method(name = "eth_blockNumber")]
    fn block_number(&self) -> RpcResult<U256>;

    #[method(name = "eth_getBlockByNumber")]
    async fn block_by_number(
        &self,
        number: BlockNumber,
        full: bool,
    ) -> RpcResult<Option<Block<Transaction>>>;

    /// Returns the receipt of a transaction by transaction hash.
    #[method(name = "eth_getTransactionReceipt")]
    async fn transaction_receipt(&self, hash: H256) -> RpcResult<Option<TransactionReceipt>>;

    /// Returns the receipt of a transaction by transaction hash.
    #[method(name = "eth_getBlockReceipts")]
    async fn block_receipts(
        &self,
        block: BlockNumber,
    ) -> RpcResult<Option<Vec<TransactionReceipt>>>;
}
