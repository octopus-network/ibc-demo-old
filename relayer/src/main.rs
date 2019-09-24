use clap::load_yaml;
use client::{
    // backend::NewBlockState, in_mem::Blockchain as InMemoryBlockchain,
    light::blockchain::Blockchain,
    light::fetcher::{FetchChecker, RemoteReadRequest},
};
use client_db::light::LightStorage;
use codec::Decode;
use executor::{native_executor_instance, NativeExecutor};
use futures::stream::Stream;
use futures::Future;
use ibc_node_runtime::{self, ibc::ParaId, ibc::RawEvent as IbcEvent, Block};
use keyring::AccountKeyring;
use node_primitives::{Hash, Index};
use primitives::{
    hexdisplay::HexDisplay,
    sr25519,
    storage::{well_known_keys, StorageKey},
    Pair,
};
use runtime_primitives::{generic::Era, traits::Header};
use std::sync::Arc;
use substrate_subxt::{
    srml::{
        ibc::{Ibc, IbcXt},
        system::System,
    },
    Client, ClientBuilder,
};
use url::Url;

native_executor_instance!(
	pub Executor,
	ibc_node_runtime::api::dispatch,
	ibc_node_runtime::native_version
);

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    execute(matches)
}

fn print_usage(matches: &clap::ArgMatches) {
    println!("{}", matches.usage());
}

fn execute(matches: clap::ArgMatches) {
    let password = matches.value_of("password");
    match matches.subcommand() {
        ("set-heads", Some(matches)) => {
            let suri = matches
                .value_of("suri")
                .expect("secret URI parameter is required; thus it can't be None; qed");
            let pair = sr25519::Pair::from_string(suri, password).expect("Invalid phrase");

            let index = matches
                .value_of("nonce")
                .expect("nonce is required; thus it can't be None; qed");
            let index = str::parse::<Index>(index)
                .expect("Invalid 'nonce' parameter; expecting an integer.");

            let genesis_hash = matches
                .value_of("genesis")
                .expect("genesis is required; thus it can't be None; qed");
            let genesis_hash: Hash = hex::decode(genesis_hash)
                .ok()
                .and_then(|x| Decode::decode(&mut &x[..]).ok())
                .expect("Invalid genesis hash");

            println!(
                "Using a genesis hash of {}",
                HexDisplay::from(&genesis_hash.as_ref())
            );
        }
        ("run", Some(matches)) => {
            let addr1 = matches
                .value_of("addr1")
                .expect("The address of chain A is required; thus it can't be None; qed");
            let addr1 = Url::parse(&format!("ws://{}", addr1)).expect("Is valid url; qed");
            let addr2 = matches
                .value_of("addr2")
                .expect("The address of chain B is required; thus it can't be None; qed");
            let addr2 = Url::parse(&format!("ws://{}", addr2)).expect("Is valid url; qed");

            let mut rt = tokio::runtime::Runtime::new().unwrap();
            let executor = rt.executor();
            let client_future = ClientBuilder::<Runtime>::new().set_url(addr1).build();
            let client = rt.block_on(client_future).unwrap();

            let stream = rt.block_on(client.subscribe_finalized_blocks()).unwrap();
            // TODO
            let read_proof =
                rt.block_on(client.read_proof(StorageKey(well_known_keys::HEAP_PAGES.to_vec())));
            let db_storage = LightStorage::<Block>::new_test();
            let light_blockchain: Arc<Blockchain<LightStorage<Block>>> =
                client::light::new_light_blockchain(db_storage);
            // let local_storage = InMemoryBlockchain::<Block>::new();
            let blocks = stream.for_each(move |block_header| {
                let header_number = block_header.number();
                let state_root = block_header.state_root();
                let block_hash = block_header.hash();
                println!("header_number: {:?}", header_number);
                println!("state_root: {:?}", state_root);
                println!("block_hash: {:?}", block_hash);
                let local_executor = NativeExecutor::<Executor>::new(None);
                let local_checker =
                    client::light::new_fetch_checker(light_blockchain.clone(), local_executor);

                let heap_pages = (&local_checker as &dyn FetchChecker<Block>)
                    .check_read_proof(
                        &RemoteReadRequest::<ibc_node_runtime::Header> {
                            block: block_header.hash(),
                            header: block_header,
                            keys: vec![well_known_keys::HEAP_PAGES.to_vec()],
                            retry_count: None,
                        },
                        // remote_read_proof,
                        vec![vec![0]],
                    )
                    .unwrap()
                    .remove(well_known_keys::HEAP_PAGES)
                    .unwrap()
                    .unwrap()[0];
                println!("heap_pages: {:?}", heap_pages);
                // local_storage
                //     .insert(
                //         block_hash,
                //         block_header.clone(),
                //         None,
                //         None,
                //         NewBlockState::Final,
                //     )
                //     .unwrap();
                Ok(())
            });
            executor.spawn(blocks.map_err(|_| ()));

            let stream = rt.block_on(client.subscribe_events()).unwrap();
            let block_events = stream
                .for_each(move |change_set| {
                    change_set
                        .changes
                        .iter()
                        .filter_map(|(_key, data)| {
                            data.as_ref().map(|data| Decode::decode(&mut &data.0[..]))
                        })
                        .for_each(
                            |result: Result<
                                Vec<
                                    srml_system::EventRecord<
                                        <Runtime as System>::Event,
                                        <Runtime as System>::Hash,
                                    >,
                                >,
                                codec::Error,
                            >| {
                                let _ = result.map(|events| {
                                    events.into_iter().for_each(|event| match event.event {
                                        ibc_node_runtime::Event::ibc(
                                            IbcEvent::InterchainMessageSent(id, message),
                                        ) => {
                                            println!("id: {}, message: {:?}", id, message);

                                            // TODO: remove this
                                            // let local_executor =
                                            //     NativeExecutor::<Executor>::new(None);
                                            // let local_checker = client::light::new_fetch_checker(
                                            //     light_blockchain.clone(),
                                            //     local_executor,
                                            // );

                                            // let heap_pages = (&local_checker
                                            //     as &dyn FetchChecker<Block>)
                                            //     .check_read_proof(
                                            //         &RemoteReadRequest::<ibc_node_runtime::Header> {
                                            //             block: remote_block_header.hash(),
                                            //             header: remote_block_header,
                                            //             key: well_known_keys::HEAP_PAGES.to_vec(),
                                            //             retry_count: None,
                                            //         },
                                            //         // remote_read_proof,
                                            //         vec![vec![0]],
                                            //     )
                                            //     .unwrap()
                                            //     .unwrap()[0];

                                            // TODO: find the corresponding rpc address according to para_id
                                            let signer = AccountKeyring::Bob.pair();
                                            let ibc_packet = ClientBuilder::<Runtime>::new()
                                                .set_url(addr2.clone())
                                                .build()
                                                .and_then(move |client| client.xt(signer, None))
                                                .and_then(move |xt| {
                                                    xt.ibc(|calls| {
                                                        calls.ibc_packet(message.to_vec())
                                                    })
                                                    .submit()
                                                })
                                                .map(|_| ())
                                                .map_err(|e| println!("{:?}", e));

                                            executor.spawn(ibc_packet);
                                        }
                                        _ => {}
                                    })
                                });
                            },
                        );
                    Ok(())
                })
                .map_err(|e| println!("{:?}", e));
            rt.spawn(block_events);
            rt.shutdown_on_idle().wait().unwrap();
        }
        ("interchain-message", Some(matches)) => {
            let para_id = matches
                .value_of("para-id")
                .expect("para-id is required; thus it can't be None; qed");
            let para_id = str::parse::<ParaId>(para_id)
                .expect("Invalid 'para-id' parameter; expecting an integer.");

            let message = matches
                .value_of("message")
                .expect("message is required; thus it can't be None; qed");
            let message: Vec<u8> = hex::decode(message).expect("Invalid message");

            let (mut rt, client) = setup();

            let signer = AccountKeyring::Alice.pair();
            let xt = rt.block_on(client.xt(signer, None)).unwrap();

            let interchain_message = xt
                .ibc(|calls| calls.interchain_message(para_id, message))
                .submit();
            rt.block_on(interchain_message).unwrap();
        }
        _ => print_usage(&matches),
    }
}

struct Runtime;

impl System for Runtime {
    type Index = <ibc_node_runtime::Runtime as srml_system::Trait>::Index;
    type BlockNumber = <ibc_node_runtime::Runtime as srml_system::Trait>::BlockNumber;
    type Hash = <ibc_node_runtime::Runtime as srml_system::Trait>::Hash;
    type Hashing = <ibc_node_runtime::Runtime as srml_system::Trait>::Hashing;
    type AccountId = <ibc_node_runtime::Runtime as srml_system::Trait>::AccountId;
    type Lookup = <ibc_node_runtime::Runtime as srml_system::Trait>::Lookup;
    type Header = <ibc_node_runtime::Runtime as srml_system::Trait>::Header;
    type Event = <ibc_node_runtime::Runtime as srml_system::Trait>::Event;

    type SignedExtra = (
        srml_system::CheckVersion<ibc_node_runtime::Runtime>,
        srml_system::CheckGenesis<ibc_node_runtime::Runtime>,
        srml_system::CheckEra<ibc_node_runtime::Runtime>,
        srml_system::CheckNonce<ibc_node_runtime::Runtime>,
        srml_system::CheckWeight<ibc_node_runtime::Runtime>,
        srml_balances::TakeFees<ibc_node_runtime::Runtime>,
    );
    fn extra(nonce: Self::Index) -> Self::SignedExtra {
        (
            srml_system::CheckVersion::<ibc_node_runtime::Runtime>::new(),
            srml_system::CheckGenesis::<ibc_node_runtime::Runtime>::new(),
            srml_system::CheckEra::<ibc_node_runtime::Runtime>::from(Era::Immortal),
            srml_system::CheckNonce::<ibc_node_runtime::Runtime>::from(nonce),
            srml_system::CheckWeight::<ibc_node_runtime::Runtime>::new(),
            srml_balances::TakeFees::<ibc_node_runtime::Runtime>::from(0),
        )
    }
}

impl Ibc for Runtime {}

fn setup() -> (tokio::runtime::Runtime, Client<Runtime>) {
    env_logger::try_init().ok();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let client_future = ClientBuilder::<Runtime>::new().build();
    let client = rt.block_on(client_future).unwrap();
    (rt, client)
}
