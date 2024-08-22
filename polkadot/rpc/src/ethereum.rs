use ethereum_types::H256;

use fc_storage::StorageOverride;
use fp_rpc::{ConvertTransactionRuntimeApi, EthereumRuntimeRPCApi};

use sc_client_api::Backend;
use sc_network::service::traits::NetworkService;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_runtime::traits::Block as BlockT;

use crate::*;

pub struct DefaultEthConfig<C, BE>(PhantomData<(C, BE)>);

impl<C, BE> EthConfig<Block, C> for DefaultEthConfig<C, BE>
where
	C: StorageProvider<Block, BE> + Sync + Send + 'static,
	BE: Backend<Block> + 'static,
{
	type EstimateGasAdapter = ();
	type RuntimeStorageOverride = SystemAccountId32StorageOverride<Block, C, BE>;
}

pub struct EthDeps<B: BlockT, C, P, A: ChainApi, CIDP> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Graph pool instance.
	pub graph: Arc<Pool<A>>,
	/// Syncing service
	pub sync: Arc<SyncingService<B>>,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<dyn NetworkService>,

	/// Ethereum Backend.
	pub eth_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
	/// Ethereum data access overrides.
	pub storage_override: Arc<dyn StorageOverride<B>>,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Maximum fee history cache size.
	pub fee_history_limit: u64,
	/// Fee history cache.
	pub fee_history_cache: FeeHistoryCache,
	pub eth_block_data_cache: Arc<EthBlockDataCacheTask<B>>,
	/// EthFilterApi pool.
	pub eth_filter_pool: Option<FilterPool>,
	pub eth_pubsub_notification_sinks:
		Arc<EthereumBlockNotificationSinks<EthereumBlockNotification<B>>>,
	/// Whether to enable eth dev signer
	pub enable_dev_signer: bool,
	
	pub pending_create_inherent_data_providers: CIDP,
}

pub fn create_eth<B, C, BE, P, A, CIDP, EC>(
	io: &mut RpcExtension,
	deps: EthDeps<B, C, P, A, CIDP>,
	subscription_task_executor: SubscriptionTaskExecutor,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
	B: BlockT<Hash = H256>,
	BE: Backend<B> + 'static,
	C: sp_api::CallApiAt<B> + ProvideRuntimeApi<B>,
	C: StorageProvider<B, BE> + AuxStore,
	C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C: BlockchainEvents<B>,
	C: UsageProvider<B>,
	C::Api: RuntimeApiDep
		+ BlockBuilderApi<B>
		+ ConvertTransactionRuntimeApi<B>
		+ EthereumRuntimeRPCApi<B>,
	P: TransactionPool<Block = B> + 'static,
	A: ChainApi<Block = B> + 'static,
	CIDP: CreateInherentDataProviders<B, ()> + Send + 'static,
	EC: EthConfig<B, C>,
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
		storage_override,
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
		pending_create_inherent_data_providers,
	} = deps;

	let mut signers = Vec::new();
	if enable_dev_signer {
		signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
	}
	let execute_gas_limit_multiplier = 10;
	io.merge(
		Eth::<B, C, P, _, BE, A, CIDP, EC>::new(
			client.clone(),
			pool.clone(),
			graph.clone(),
			// We have no runtimes old enough to only accept converted transactions.
			None::<NoTransactionConverter>,
			sync.clone(),
			signers,
			storage_override.clone(),
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
		.replace_config::<EC>()
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
			storage_override,
			eth_pubsub_notification_sinks,
		)
		.into_rpc(),
	)?;

	Ok(())
}
