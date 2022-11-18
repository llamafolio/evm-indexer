use anyhow::Result;
use web3::{ Web3, transports::{WebSocket, Http, Batch}, types::{BlockId, Block, Transaction, U64} };

pub struct IndexerRPC {
    pub batch: Web3<Batch<Http>>,
    pub wss: Web3<WebSocket>,
}

impl IndexerRPC {
    pub async fn new(rpc_ws_url: &str, rpc_http_url: &str) -> Result<Self> {
        log::info!("==> IndexerRPC: Initializing IndexerRPC");

        let http = Http::new(rpc_http_url)?;
        let ws = WebSocket::new(rpc_ws_url).await?;

        Ok(IndexerRPC {
            wss: Web3::new(ws),
            batch: Web3::new(web3::transports::Batch::new(http))
        })
    }

    pub async fn last_block(&self) -> Result<i64> {
        let last_block = self.wss.eth().block_number().await.expect("Unable to fetch last block").as_u64();

        Ok(last_block as i64)
    
    }

    pub async fn fetch_block_batch(&self, range: &[i64]) -> Result<Vec<Block<Transaction>>> {

        for block_height in range.iter() {

            let block_number = U64::from_str_radix(&block_height.to_string(), 10).expect("Unable to parse block number");

            let block_id = <BlockId as From<U64>>::from(block_number);

            self.batch.eth().block_with_txs(block_id);
        }

        let result = self.batch.transport().submit_batch().await.unwrap();

        let mut blocks: Vec<Block<Transaction>> = Vec::new();

        for block_response in result {
            let block_type: Block<Transaction> = serde_json::from_value(block_response.unwrap().clone()).unwrap();
            blocks.push(block_type);
        }

        Ok(blocks)

    }

}