//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use crate::eth::{
    db_config_dir, new_frontier_partial, EthConfiguration, FrontierBackend,
    FrontierPartialComponents,
};
use academy_pow_runtime::{self, opaque::Block, RuntimeApi, TransactionConverter};
use account::AccountId20;
use core::clone::Clone;
use fc_storage::overrides_handle;
use futures::channel::mpsc;
use parity_scale_codec::Encode;
use sc_consensus::LongestChain;
use sc_executor::NativeElseWasmExecutor;
use sc_service::{error::Error as ServiceError, Configuration, PartialComponents, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sha3pow::Sha3Algorithm;
use sp_api::TransactionFor;
use std::sync::Arc;

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

pub type FullClient =
    sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
pub type FullBackend = sc_service::TFullBackend<Block>;
pub type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub type BasicImportQueue = sc_consensus::DefaultImportQueue<Block, FullClient>;
pub type BoxBlockImport = sc_consensus::BoxBlockImport<Block, TransactionFor<FullClient, Block>>;
pub type ServicePartialComponents = PartialComponents<
    FullClient,
    FullBackend,
    FullSelectChain,
    BasicImportQueue,
    sc_transaction_pool::FullPool<Block, FullClient>,
    (BoxBlockImport, Option<Telemetry>),
>;

/// Returns most parts of a service. Not enough to run a full chain,
/// But enough to perform chain operations like purge-chain
pub fn new_partial<BIQ>(
    config: &Configuration,
    build_import_queue: BIQ,
) -> Result<ServicePartialComponents, ServiceError>
where
    BIQ: FnOnce(
        Arc<FullClient>,
        &Configuration,
        &FullSelectChain,
        &TaskManager,
    ) -> Result<(BasicImportQueue, BoxBlockImport), ServiceError>,
{
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

    let executor = sc_service::new_native_or_wasm_executor(config);

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let (import_queue, block_import) =
        build_import_queue(client.clone(), config, &select_chain, &task_manager)?;

    Ok(PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, telemetry),
    })
}

/// Build the import queue for the manual seal service.
pub fn build_manual_seal_import_queue(
    client: Arc<FullClient>,
    config: &Configuration,
    _select_chain: &FullSelectChain,
    task_manager: &TaskManager,
) -> Result<(BasicImportQueue, BoxBlockImport), ServiceError> {
    Ok((
        sc_consensus_manual_seal::import_queue(
            Box::new(client.clone()),
            &task_manager.spawn_essential_handle(),
            config.prometheus_registry(),
        ),
        Box::new(client),
    ))
}

/// Build the import queue for the pow service
pub fn build_pow_import_queue(
    client: Arc<FullClient>,
    config: &Configuration,
    select_chain: &FullSelectChain,
    task_manager: &TaskManager,
) -> Result<(BasicImportQueue, BoxBlockImport), ServiceError> {
    let pow_block_import = sc_consensus_pow::PowBlockImport::new(
        client.clone(),
        client.clone(),
        Sha3Algorithm::new(client.clone()),
        0, // check inherents starting at block 0
        select_chain.clone(),
        move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

            // We don't need the current mining key to check inherents, so we just use a default.
            let author =
                academy_pow_runtime::block_author::InherentDataProvider(Default::default());

            Ok((timestamp, author))
        },
    );

    let import_queue = sc_consensus_pow::import_queue(
        Box::new(pow_block_import.clone()),
        None,
        Sha3Algorithm::new(client),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    )?;

    Ok((import_queue, Box::new(pow_block_import)))
}

/// Builds a new service for a full client.
pub fn new_full(
    config: Configuration,
    mining_account_id: AccountId20,
    instant_seal: bool,
    eth_config: &EthConfiguration,
) -> Result<TaskManager, ServiceError> {
    let build_import_queue = if instant_seal {
        build_manual_seal_import_queue
    } else {
        build_pow_import_queue
    };

    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (pow_block_import, mut telemetry),
    } = new_partial(&config, build_import_queue)?;

    let FrontierPartialComponents {
        filter_pool: _filter_pool,
        fee_history_cache,
        fee_history_cache_limit,
    } = new_frontier_partial(eth_config)?;

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

    let frontier_backend = Arc::new(FrontierBackend::open(
        client.clone(),
        &config.database,
        &db_config_dir(&config),
    )?);

    // for ethereum-compatibility rpc.
    // config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider)); // TODO: wants mut...
    let overrides = overrides_handle(client.clone());
    let eth_rpc_params = crate::rpc::EthDeps {
        client: client.clone(),
        pool: transaction_pool.clone(),
        graph: transaction_pool.pool().clone(),
        converter: Some(TransactionConverter),
        is_authority: config.role.is_authority(),
        enable_dev_signer: eth_config.enable_dev_signer,
        network: network.clone(),
        sync: sync_service.clone(),
        frontier_backend,
        overrides: overrides.clone(),
        block_data_cache: Arc::new(fc_rpc::EthBlockDataCacheTask::new(
            task_manager.spawn_handle(),
            overrides,
            eth_config.eth_log_block_cache,
            eth_config.eth_statuses_cache,
            prometheus_registry.clone(),
        )),
        filter_pool: None,
        max_past_logs: eth_config.max_past_logs,
        fee_history_cache,
        fee_history_cache_limit,
        execute_gas_limit_multiplier: eth_config.execute_gas_limit_multiplier,
        forced_parent_hashes: None,
    };

    // Channel for the rpc handler to communicate with the authorship task.
    let (command_sink, _commands_stream) = mpsc::channel(1000);

    let rpc_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, subscription_task_executor| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                deny_unsafe,
                command_sink: if instant_seal {
                    Some(command_sink.clone())
                } else {
                    None
                },
                eth: eth_rpc_params.clone(),
            };

            crate::rpc::create_full(deps, subscription_task_executor)
                .map_err(Into::<ServiceError>::into)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network,
        client: client.clone(),
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder,
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
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        // If instant seal is requested, we just start it. Otherwise, we do the full PoW setup.
        if instant_seal {
            let params = sc_consensus_manual_seal::InstantSealParams {
                block_import: client.clone(),
                env: proposer,
                client,
                pool: transaction_pool,
                select_chain,
                consensus_data_provider: None,
                create_inherent_data_providers: move |_, ()| async move {
                    Ok(sp_timestamp::InherentDataProvider::from_system_time())
                },
            };

            let authorship_future = sc_consensus_manual_seal::run_instant_seal(params);

            task_manager.spawn_essential_handle().spawn_blocking(
                "instant-seal",
                None,
                authorship_future,
            );
        } else {
            let (mining_worker, mining_worker_task) = sc_consensus_pow::start_mining_worker(
                Box::new(pow_block_import),
                client.clone(),
                select_chain,
                Sha3Algorithm::new(client),
                proposer,
                sync_service.clone(),
                sync_service,
                None,
                // This code is copied from above. Would be better to not repeat it.
                move |_, ()| async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let author = academy_pow_runtime::block_author::InherentDataProvider(
                        mining_account_id.encode(),
                    );

                    Ok((timestamp, author))
                },
                std::time::Duration::from_secs(10),
                std::time::Duration::from_secs(5),
            );

            task_manager.spawn_essential_handle().spawn_blocking(
                "pow-miner",
                Some("pow-mining"),
                mining_worker_task,
            );

            // Start Mining
            //TODO Some of this should move into the sha3pow crate.
            use sha3pow::{hash_meets_difficulty, Compute};
            use sp_core::U256;
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
    }

    network_starter.start_network();
    Ok(task_manager)
}
