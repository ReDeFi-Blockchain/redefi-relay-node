use sc_client_api::Backend;

use crate::*;

pub struct DefaultEthConfig<BE, C>(PhantomData<(BE, C)>);
impl<BE, C> EthConfig<Block, C> for DefaultEthConfig<BE, C>
where
	BE: Backend<Block> + 'static,
	C: StorageProvider<Block, BE> + Sync + Send + 'static,
{
	type EstimateGasAdapter = ();
	type RuntimeStorageOverride = SystemAccountId32StorageOverride<Block, C, BE>;
}

pub struct EthDeps<C, P, CA: ChainApi, CIDP> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Graph pool instance.
	pub graph: Arc<Pool<CA>>,
	/// Syncing service
	pub sync: Arc<SyncingService<Block>>,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,

	/// Ethereum Backend.
	pub eth_backend: Arc<dyn fc_api::Backend<Block> + Send + Sync>,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Maximum fee history cache size.
	pub fee_history_limit: u64,
	/// Fee history cache.
	pub fee_history_cache: FeeHistoryCache,
	pub eth_block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
	/// EthFilterApi pool.
	pub eth_filter_pool: Option<FilterPool>,
	pub eth_pubsub_notification_sinks:
		Arc<EthereumBlockNotificationSinks<EthereumBlockNotification<Block>>>,
	/// Whether to enable eth dev signer
	pub enable_dev_signer: bool,

	pub overrides: Arc<OverrideHandle<Block>>,
	pub pending_create_inherent_data_providers: CIDP,
}

pub fn create_eth<C, P, CA, B, CIDP, EC>(
	io: &mut RpcExtension,
	deps: EthDeps<C, P, CA, CIDP>,
	subscription_task_executor: SubscriptionTaskExecutor,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, B> + AuxStore,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C: BlockchainEvents<Block>,
	C: UsageProvider<Block>,
	C::Api: RuntimeApiDep,
	P: TransactionPool<Block = Block> + 'static,
	CA: ChainApi<Block = Block> + 'static,
	B: sc_client_api::Backend<Block> + Send + Sync + 'static,
	C: sp_api::CallApiAt<Block>,
	CIDP: CreateInherentDataProviders<Block, ()> + Send + 'static,
	EC: EthConfig<Block, C>,
{
	use fc_rpc::{
		Eth, EthApiServer, EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub,
		EthPubSubApiServer, EthSigner, Net, NetApiServer, Web3, Web3ApiServer,
	};

	let EthDeps {
		client,
		pool,
		graph,
		eth_backend,
		max_past_logs,
		fee_history_limit,
		fee_history_cache,
		eth_block_data_cache,
		eth_filter_pool,
		eth_pubsub_notification_sinks,
		enable_dev_signer,
		sync,
		is_authority,
		network,
		overrides,
		pending_create_inherent_data_providers,
	} = deps;

	let mut signers = Vec::new();
	if enable_dev_signer {
		signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
	}
	let execute_gas_limit_multiplier = 10;
	io.merge(
		Eth::<_, _, _, _, _, _, _, EC>::new(
			client.clone(),
			pool.clone(),
			graph.clone(),
			// We have no runtimes old enough to only accept converted transactions.
			None::<NoTransactionConverter>,
			sync.clone(),
			signers,
			overrides.clone(),
			eth_backend.clone(),
			is_authority,
			eth_block_data_cache.clone(),
			fee_history_cache,
			fee_history_limit,
			execute_gas_limit_multiplier,
			None,
			pending_create_inherent_data_providers,
			// Our extrinsics have nothing to do with consensus digest items yet.
			None,
		)
		.into_rpc(),
	)?;

	if let Some(filter_pool) = eth_filter_pool {
		io.merge(
			EthFilter::new(
				client.clone(),
				eth_backend,
				graph,
				filter_pool,
				500_usize, // max stored filters
				max_past_logs,
				eth_block_data_cache,
			)
			.into_rpc(),
		)?;
	}
	io.merge(
		Net::new(
			client.clone(),
			network,
			// Whether to format the `peer_count` response as Hex (default) or not.
			true,
		)
		.into_rpc(),
	)?;
	io.merge(Web3::new(client.clone()).into_rpc())?;
	io.merge(
		EthPubSub::new(
			pool,
			client,
			sync,
			subscription_task_executor,
			overrides,
			eth_pubsub_notification_sinks,
		)
		.into_rpc(),
	)?;

	Ok(())
}
