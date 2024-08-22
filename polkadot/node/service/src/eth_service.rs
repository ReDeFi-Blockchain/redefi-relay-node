use crate::*;
use futures::StreamExt;

pub struct FrontierTaskParams<'a, C, B> {
	pub task_manager: &'a TaskManager,
	pub client: Arc<C>,
	pub substrate_backend: Arc<B>,
	pub eth_backend: Arc<fc_db::kv::Backend<Block, C>>,
	pub eth_filter_pool: Option<FilterPool>,
	pub storage_override: Arc<dyn StorageOverride<Block>>,
	pub fee_history_limit: u64,
	pub fee_history_cache: FeeHistoryCache,
	pub sync_strategy: SyncStrategy,
	pub prometheus_registry: Option<Registry>,
}

pub(crate) fn ethereum_relay_inherent() -> sp_timestamp::InherentDataProvider {
	sp_timestamp::InherentDataProvider::from_system_time()
}

pub fn spawn_frontier_tasks<C, B>(
	params: FrontierTaskParams<C, B>,
	sync: Arc<sc_network_sync::SyncingService<Block>>,
	pubsub_notification_sinks: Arc<
		EthereumBlockNotificationSinks<fc_mapping_sync::EthereumBlockNotification<Block>>,
	>,
) -> Arc<fc_rpc::EthBlockDataCacheTask<Block>>
where
	C: ProvideRuntimeApi<Block> + BlockOf,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = sp_blockchain::Error> + 'static,
	C: BlockchainEvents<Block> + StorageProvider<Block, B>,
	C: Send + Sync + 'static,
	C::Api: EthereumRuntimeRPCApi<Block>,
	C::Api: BlockBuilder<Block>,
	B: Backend<Block> + 'static,
	//B::State: StateBackend<BlakeTwo256>,
{
	let FrontierTaskParams {
		task_manager,
		client,
		substrate_backend,
		eth_backend,
		eth_filter_pool,
		storage_override,
		fee_history_limit,
		fee_history_cache,
		sync_strategy,
		prometheus_registry,
	} = params;
	// Frontier offchain DB task. Essential.
	// Maps emulated ethereum data to substrate native data.
	params.task_manager.spawn_essential_handle().spawn(
		"frontier-mapping-sync-worker",
		Some("frontier"),
		MappingSyncWorker::new(
			client.import_notification_stream(),
			Duration::new(6, 0),
			client.clone(),
			substrate_backend,
			storage_override.clone(),
			eth_backend,
			3,
			0,
			sync_strategy,
			sync,
			pubsub_notification_sinks,
		)
		.for_each(|()| futures::future::ready(())),
	);

	// Frontier `EthFilterApi` maintenance.
	// Manages the pool of user-created Filters.
	if let Some(eth_filter_pool) = eth_filter_pool {
		// Each filter is allowed to stay in the pool for 100 blocks.
		const FILTER_RETAIN_THRESHOLD: u64 = 100;
		params.task_manager.spawn_essential_handle().spawn(
			"frontier-filter-pool",
			Some("frontier"),
			EthTask::filter_pool_task(client.clone(), eth_filter_pool, FILTER_RETAIN_THRESHOLD),
		);
	}

	// Spawn Frontier FeeHistory cache maintenance task.
	params.task_manager.spawn_essential_handle().spawn(
		"frontier-fee-history",
		Some("frontier"),
		EthTask::fee_history_task(client, storage_override.clone(), fee_history_cache, fee_history_limit),
	);

	Arc::new(EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		storage_override,
		50,
		50,
		prometheus_registry,
	))
}
