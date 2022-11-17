use jsonrpsee_ws_client::{ WsClient, WsClientBuilder };

pub struct IndexerRPC {
    pub client: WsClient,
}

impl IndexerRPC {
    pub async fn new(url: &str) -> Self {
        IndexerRPC {
            client: WsClientBuilder::default().build(url).await.unwrap(),
        }
    }
}