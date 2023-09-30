use std::sync::Arc;

use ethers::prelude::*;
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{direct::NotKeyed, InMemoryState},
};
use tokio::sync::{AcquireError, Semaphore, SemaphorePermit};

use crate::CollectError;

/// RateLimiter based on governor crate
pub type RateLimiter = governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

/// Options for fetching data from node
#[derive(Clone)]
pub struct Source {
    /// Shared provider for rpc data
    pub fetcher: Arc<Fetcher<Http>>,
    /// chain_id of network
    pub chain_id: u64,
    /// number of blocks per log request
    pub inner_request_size: u64,
    /// Maximum chunks collected concurrently
    pub max_concurrent_chunks: u64,
}

/// Wrapper over `Provider<P>` that adds concurrency and rate limiting controls
pub struct Fetcher<P> {
    /// provider data source
    pub provider: Provider<P>,
    /// semaphore for controlling concurrency
    pub semaphore: Option<Semaphore>,
    /// rate limiter for controlling request rate
    pub rate_limiter: Option<RateLimiter>,
}

type Result<T> = ::core::result::Result<T, CollectError>;

impl<P: JsonRpcClient> Fetcher<P> {
    /// Returns an array (possibly empty) of logs that match the filter
    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.get_logs(filter).await)
    }

    /// Replays all transactions in a block returning the requested traces for each transaction
    pub async fn trace_replay_block_transactions(
        &self,
        block: BlockNumber,
        trace_types: Vec<TraceType>,
    ) -> Result<Vec<BlockTrace>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.trace_replay_block_transactions(block, trace_types).await)
    }

    /// Get state diff traces of block
    pub async fn trace_block_state_diffs(
        &self,
        block: u32,
    ) -> Result<(Option<u32>, Option<Vec<u8>>, Vec<BlockTrace>)> {
        let result = self
            .trace_replay_block_transactions(
                block.into(),
                vec![ethers::types::TraceType::StateDiff],
            )
            .await;
        Ok((Some(block), None, result?))
    }

    /// Get VM traces of block
    pub async fn trace_block_vm_traces(
        &self,
        block: u32,
    ) -> Result<(Option<u32>, Option<Vec<u8>>, Vec<BlockTrace>)> {
        let result = self
            .trace_replay_block_transactions(block.into(), vec![ethers::types::TraceType::VmTrace])
            .await;
        Ok((Some(block), None, result?))
    }

    /// Replays a transaction, returning the traces
    pub async fn trace_replay_transaction(
        &self,
        tx_hash: TxHash,
        trace_types: Vec<TraceType>,
    ) -> Result<BlockTrace> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.trace_replay_transaction(tx_hash, trace_types).await)
    }

    /// Get state diff traces of transaction
    pub async fn trace_transaction_state_diffs(
        &self,
        transaction_hash: Vec<u8>,
    ) -> Result<(Option<u32>, Option<Vec<u8>>, Vec<BlockTrace>)> {
        let result = self
            .trace_replay_transaction(
                H256::from_slice(&transaction_hash),
                vec![ethers::types::TraceType::StateDiff],
            )
            .await;
        Ok((None, Some(transaction_hash), vec![result?]))
    }

    /// Get VM traces of transaction
    pub async fn trace_transaction_vm_traces(
        &self,
        transaction_hash: Vec<u8>,
    ) -> Result<(Option<u32>, Option<Vec<u8>>, Vec<BlockTrace>)> {
        let result = self
            .trace_replay_transaction(
                H256::from_slice(&transaction_hash),
                vec![ethers::types::TraceType::VmTrace],
            )
            .await;
        Ok((None, Some(transaction_hash), vec![result?]))
    }

    /// Gets the transaction with transaction_hash
    pub async fn get_transaction(&self, tx_hash: TxHash) -> Result<Option<Transaction>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.get_transaction(tx_hash).await)
    }

    /// Gets the transaction receipt with transaction_hash
    pub async fn get_transaction_receipt(
        &self,
        tx_hash: TxHash,
    ) -> Result<Option<TransactionReceipt>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.get_transaction_receipt(tx_hash).await)
    }

    /// Gets the block at `block_num` (transaction hashes only)
    pub async fn get_block(&self, block_num: u64) -> Result<Option<Block<TxHash>>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.get_block(block_num).await)
    }

    /// Gets the block at `block_num` (transaction hashes only)
    pub async fn get_block_by_hash(&self, block_hash: H256) -> Result<Option<Block<TxHash>>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.get_block(BlockId::Hash(block_hash)).await)
    }

    /// Gets the block at `block_num` (full transactions included)
    pub async fn get_block_with_txs(&self, block_num: u64) -> Result<Option<Block<Transaction>>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.get_block_with_txs(block_num).await)
    }

    /// Returns all receipts for a block.
    pub async fn get_block_receipts(&self, block_num: u64) -> Result<Vec<TransactionReceipt>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.get_block_receipts(block_num).await)
    }

    /// Returns traces created at given block
    pub async fn trace_block(&self, block_num: BlockNumber) -> Result<Vec<Trace>> {
        let _permit = self.permit_request().await;
        Self::map_err(self.provider.trace_block(block_num).await)
    }

    /// Returns all traces of a given transaction
    pub async fn trace_transaction(&self, tx_hash: TxHash) -> Result<Vec<Trace>> {
        let _permit = self.permit_request().await;
        self.provider.trace_transaction(tx_hash).await.map_err(CollectError::ProviderError)
    }

    /// Return output data of a contract call
    pub async fn call2(
        &self,
        address: H160,
        call_data: Vec<u8>,
        block_number: BlockNumber,
    ) -> Result<Bytes> {
        let transaction = TransactionRequest {
            to: Some(address.into()),
            data: Some(call_data.into()),
            ..Default::default()
        };
        let _permit = self.permit_request().await;
        self.provider
            .call(&transaction.into(), Some(block_number.into()))
            .await
            .map_err(CollectError::ProviderError)
    }

    /// Deprecated
    pub async fn call(
        &self,
        transaction: TransactionRequest,
        block_number: BlockNumber,
    ) -> Result<Bytes> {
        let _permit = self.permit_request().await;
        self.provider
            .call(&transaction.into(), Some(block_number.into()))
            .await
            .map_err(CollectError::ProviderError)
    }

    /// Returns traces for given call data
    pub async fn trace_call(
        &self,
        transaction: TransactionRequest,
        trace_type: Vec<TraceType>,
        block_number: Option<BlockNumber>,
    ) -> Result<BlockTrace> {
        let _permit = self.permit_request().await;
        self.provider
            .trace_call(transaction, trace_type, block_number)
            .await
            .map_err(CollectError::ProviderError)
    }

    /// Get nonce of address
    pub async fn get_transaction_count(
        &self,
        address: H160,
        block_number: BlockNumber,
    ) -> Result<U256> {
        let _permit = self.permit_request().await;
        self.provider
            .get_transaction_count(address, Some(block_number.into()))
            .await
            .map_err(CollectError::ProviderError)
    }

    /// Get code at address
    pub async fn get_balance(&self, address: H160, block_number: BlockNumber) -> Result<U256> {
        let _permit = self.permit_request().await;
        self.provider
            .get_balance(address, Some(block_number.into()))
            .await
            .map_err(CollectError::ProviderError)
    }

    /// Get code at address
    pub async fn get_code(&self, address: H160, block_number: BlockNumber) -> Result<Bytes> {
        let _permit = self.permit_request().await;
        self.provider
            .get_code(address, Some(block_number.into()))
            .await
            .map_err(CollectError::ProviderError)
    }

    /// Get stored data at given location
    pub async fn get_storage_at(
        &self,
        address: H160,
        slot: H256,
        block_number: BlockNumber,
    ) -> Result<H256> {
        let _permit = self.permit_request().await;
        self.provider
            .get_storage_at(address, slot, Some(block_number.into()))
            .await
            .map_err(CollectError::ProviderError)
    }

    /// Get the block number
    pub async fn get_block_number(&self) -> Result<U64> {
        Self::map_err(self.provider.get_block_number().await)
    }

    // extra helpers below

    /// block number of transaction
    pub async fn get_transaction_block_number(&self, transaction_hash: Vec<u8>) -> Result<u32> {
        let block = self.get_transaction(H256::from_slice(&transaction_hash)).await?;
        let block = block.ok_or(CollectError::CollectError("could not get block".to_string()))?;
        Ok(block
            .block_number
            .ok_or(CollectError::CollectError("could not get block number".to_string()))?
            .as_u32())
    }

    /// block number of transaction
    pub async fn get_transaction_logs(&self, transaction_hash: Vec<u8>) -> Result<Vec<Log>> {
        Ok(self
            .get_transaction_receipt(H256::from_slice(&transaction_hash))
            .await?
            .ok_or(CollectError::CollectError("transaction receipt not found".to_string()))?
            .logs)
    }

    async fn permit_request(
        &self,
    ) -> Option<::core::result::Result<SemaphorePermit<'_>, AcquireError>> {
        let permit = match &self.semaphore {
            Some(semaphore) => Some(semaphore.acquire().await),
            _ => None,
        };
        if let Some(limiter) = &self.rate_limiter {
            limiter.until_ready().await;
        }
        permit
    }

    fn map_err<T>(res: ::core::result::Result<T, ProviderError>) -> Result<T> {
        res.map_err(CollectError::ProviderError)
    }
}

use tokio::task;

impl Source {
    /// get gas used by transactions in block
    pub async fn get_txs_gas_used(&self, block: &Block<Transaction>) -> Result<Vec<u32>> {
        match get_txs_gas_used_per_block(block, self.fetcher.clone()).await {
            Ok(value) => Ok(value),
            Err(_) => get_txs_gas_used_per_tx(block, self.fetcher.clone()).await,
        }
    }
}

async fn get_txs_gas_used_per_block<P: JsonRpcClient>(
    block: &Block<Transaction>,
    fetcher: Arc<Fetcher<P>>,
) -> Result<Vec<u32>> {
    // let fetcher = Arc::new(fetcher);
    let block_number = match block.number {
        Some(number) => number,
        None => return Err(CollectError::CollectError("no block number".to_string())),
    };
    let receipts = fetcher.get_block_receipts(block_number.as_u64()).await?;
    let mut gas_used: Vec<u32> = Vec::new();
    for receipt in receipts {
        match receipt.gas_used {
            Some(value) => gas_used.push(value.as_u32()),
            None => return Err(CollectError::CollectError("no gas_used for tx".to_string())),
        }
    }
    Ok(gas_used)
}

async fn get_txs_gas_used_per_tx<P: JsonRpcClient + 'static>(
    block: &Block<Transaction>,
    fetcher: Arc<Fetcher<P>>,
) -> Result<Vec<u32>> {
    // let fetcher = Arc::new(*fetcher.clone());
    let mut tasks = Vec::new();
    for tx in &block.transactions {
        let tx_clone = tx.hash;
        let fetcher = fetcher.clone();
        let task = task::spawn(async move {
            match fetcher.get_transaction_receipt(tx_clone).await? {
                Some(receipt) => Ok(receipt.gas_used),
                None => Err(CollectError::CollectError("could not find tx receipt".to_string())),
            }
        });
        tasks.push(task);
    }

    let mut gas_used: Vec<u32> = Vec::new();
    for task in tasks {
        match task.await {
            Ok(Ok(Some(value))) => gas_used.push(value.as_u32()),
            _ => return Err(CollectError::CollectError("gas_used not available from node".into())),
        }
    }

    Ok(gas_used)
}

// impl<Q: JsonRpcClient> Fetcher<Q> {
//     /// get gas used by transactions in block
//     pub async fn get_txs_gas_used<P: JsonRpcClient + 'static>(
//         block: &Block<Transaction>,
//         fetcher: Arc<Fetcher<P>>,
//     ) -> Result<Vec<u32>, CollectError> { match get_txs_gas_used_per_block(block,
//       fetcher.clone()).await { Ok(value) => Ok(value), Err(_) => get_txs_gas_used_per_tx(block,
//       fetcher).await, }
//     }

// }

// async fn get_txs_gas_used_per_block<P: JsonRpcClient + 'static>(
//     block: &Block<Transaction>,
//     fetcher: Arc<Fetcher<P>>,
// ) -> Result<Vec<u32>, CollectError> { let block_number = match block.number { Some(number) =>
//   number, None => return Err(CollectError::CollectError("no block number".to_string())), }; let
//   receipts = fetcher.get_block_receipts(block_number.as_u64()).await?; let mut gas_used: Vec<u32>
//   = Vec::new(); for receipt in receipts { match receipt.gas_used { Some(value) =>
//   gas_used.push(value.as_u32()), None => return Err(CollectError::CollectError("no gas_used for
//   tx".to_string())), } } Ok(gas_used)
// }

// async fn get_txs_gas_used_per_tx<P: JsonRpcClient + 'static>(
//     block: &Block<Transaction>,
//     fetcher: Arc<Fetcher<P>>,
// ) -> Result<Vec<u32>, CollectError> { let mut tasks = Vec::new(); for tx in &block.transactions {
//   let tx_clone = tx.hash; let fetcher = fetcher.clone(); let task = task::spawn(async move {
//   match fetcher.get_transaction_receipt(tx_clone).await? { Some(receipt) => Ok(receipt.gas_used),
//   None => Err(CollectError::CollectError("could not find tx receipt".to_string())), } });
//   tasks.push(task); }

//     let mut gas_used: Vec<u32> = Vec::new();
//     for task in tasks {
//         match task.await {
//             Ok(Ok(Some(value))) => gas_used.push(value.as_u32()),
//             _ => return Err(CollectError::CollectError("gas_used not available from
// node".into())),         }
//     }

//     Ok(gas_used)
// }
