use crate::*;
/// Generate a supertrait based on bounds, and blanket impl for it.
macro_rules! ez_bounds {
	($vis:vis trait $name:ident$(<$($gen:ident $(: $($(+)? $bound:path)*)?),* $(,)?>)? $(:)? $($(+)? $super:path)* {}) => {
		$vis trait $name $(<$($gen $(: $($bound+)*)?,)*>)?: $($super +)* {}
		impl<T, $($($gen $(: $($bound+)*)?,)*)?> $name$(<$($gen,)*>)? for T
		where T: $($super +)* {}
	}
}
ez_bounds!(
	//TODO
	pub trait RuntimeApiDep:
		sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_consensus_babe::BabeApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ sp_api::Metadata<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ fp_rpc::ConvertTransactionRuntimeApi<Block>
	{
	}
);

pub fn open_frontier_backend<C: HeaderBackend<Block>>(
	client: Arc<C>,
	config: &sc_service::Configuration,
) -> Result<Arc<fc_db::kv::Backend<Block, C>>, String> {
	let config_dir = config.base_path.config_dir(config.chain_spec.id());
	let database_dir = config_dir.join("frontier").join("db");

	Ok(Arc::new(fc_db::kv::Backend::<Block, C>::new(
		client,
		&fc_db::kv::DatabaseSettings {
			source: fc_db::DatabaseSource::RocksDb { path: database_dir, cache_size: 0 },
		},
	)?))
}
