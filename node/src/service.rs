//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::sync::Arc;
use sp_inherents::CreateInherentDataProviders;
use academy_pow_runtime::{self, opaque::Block, RuntimeApi};
use sc_service::{error::Error as ServiceError, Configuration, PartialComponents, TaskManager};
use sc_executor::NativeElseWasmExecutor;
use sha3pow::Sha3Algorithm;
use core::clone::Clone;
use sp_core::sr25519;
use parity_scale_codec::Encode;
use sc_telemetry::{Telemetry, TelemetryWorker};

// Our native executor instance.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {

	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		academy_pow_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		academy_pow_runtime::native_version()
	}
}

//TODO We'll need the mining worker. Can probably copy from recipes

type FullClient = sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// Returns most parts of a service. Not enough to run a full chain,
/// But enough to perform chain operations like purge-chain
pub fn new_partial(config: &Configuration, sr25519_public_key: sr25519::Public) -> Result<
	PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block, FullClient>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			sc_consensus_pow::PowBlockImport<
				Block,
				Arc<FullClient>,
				FullClient,
				FullSelectChain,
				Sha3Algorithm<FullClient>,
				impl CreateInherentDataProviders<Block, ()>,
			>,
			Option<Telemetry>,
		),
	>,
	ServiceError,
> {
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

		let executor = sc_service::new_native_or_wasm_executor(&config);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			&config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let pow_block_import = sc_consensus_pow::PowBlockImport::new(
		client.clone(),
		client.clone(),
		Sha3Algorithm::new(client.clone()),
		0, // check inherents starting at block 0
		select_chain.clone(),
		move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			let author =
				academy_pow_runtime::block_author::InherentDataProvider(
					sr25519_public_key.encode(),
				);

			Ok((timestamp, author))
		},
	);

	let import_queue = sc_consensus_pow::import_queue(
		Box::new(pow_block_import.clone()),
		None,
		Sha3Algorithm::new(client.clone()),
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
	)?;

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (pow_block_import, telemetry),
	})
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration, sr25519_public_key: sr25519::Public) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (pow_block_import, mut telemetry),
	} = new_partial(&config, sr25519_public_key)?;

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: None,
		})?;

	let role = config.role.clone();
	let prometheus_registry = config.prometheus_registry().cloned();

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_builder: Box::new(|_, _| Ok(jsonrpsee::RpcModule::new(()))),
		backend,
		system_rpc_tx,
		tx_handler_controller,
		sync_service: sync_service.clone(),
		config,
		telemetry: telemetry.as_mut(),
	})?;



	if role.is_authority() {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let (mining_worker, mining_worker_task) = sc_consensus_pow::start_mining_worker(
			Box::new(pow_block_import),
			client.clone(),
			select_chain,
			Sha3Algorithm::new(client.clone()),
			proposer,
			sync_service.clone(),
			sync_service.clone(),
			None,
			// This code is copied from above. Would be better to not repeat it.
			move |_, ()| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
	
				let author =
					academy_pow_runtime::block_author::InherentDataProvider(
						sr25519_public_key.encode(),
					);
	
				Ok((timestamp, author))
			},
			std::time::Duration::from_secs(10),
			std::time::Duration::from_secs(5),
		);

		task_manager
			.spawn_essential_handle()
			.spawn_blocking("pow-miner", Some("pow-mining"), mining_worker_task);

		// Start Mining
		//TODO Some of this should move into the sha3pow crate.
		use sp_core::U256;
		use sha3pow::{Compute, hash_meets_difficulty};
		let mut nonce: U256 = U256::from(0);
		std::thread::spawn(move || loop {
			let worker = mining_worker.clone();
			let metadata = worker.metadata();
			if let Some(metadata) = metadata {
				let compute = Compute {
					difficulty: metadata.difficulty,
					pre_hash: metadata.pre_hash,
					nonce,
				};
				let seal = compute.compute();
				if hash_meets_difficulty(&seal.work, seal.difficulty) {
					nonce = U256::from(0);
					let _ = futures::executor::block_on(worker.submit(seal.encode()));
				} else {
					nonce = nonce.saturating_add(U256::from(1));
					if nonce == U256::MAX {
						nonce = U256::from(0);
					}
				}
			} else {
				std::thread::sleep(std::time::Duration::from_secs(1));
			}
		});
	}

	network_starter.start_network();
	Ok(task_manager)
}