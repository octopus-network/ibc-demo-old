use clap::load_yaml;
use codec::{Decode, Encode};
use futures::stream::Stream;
use futures::Future;
use hyper::rt;
use ibc_node_runtime::{
    self, ibc::Call as IbcCall, ibc::ParaId, ibc::RawEvent as IbcEvent, Call, UncheckedExtrinsic,
};
use jsonrpc_core_client::{transports::http, RpcError};
use keyring::AccountKeyring;
use node_primitives::{Hash, Index};
use primitives::{
    blake2_256,
    hexdisplay::HexDisplay,
    sr25519::{self, Public as AddressPublic},
    Pair,
};
use rpc::author::AuthorClient;
use substrate_subxt::{srml::system::System, Client, ClientBuilder};

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

            let from = AddressPublic::from_raw(pair.public().0);
            let signer = pair.clone();

            let id: u32 = 0;
            let head: Vec<u8> = [1, 2, 3].to_vec();

            let function = Call::Ibc(IbcCall::set_heads(id, head));
            let extra = |i: Index| {
                (
                    system::CheckGenesis::<ibc_node_runtime::Runtime>::new(),
                    system::CheckNonce::<ibc_node_runtime::Runtime>::from(i),
                )
            };

            let raw_payload = (function, extra(index), genesis_hash);
            let signature = raw_payload.using_encoded(|payload| {
                if payload.len() > 256 {
                    signer.sign(&blake2_256(payload)[..])
                } else {
                    println!("Signing {}", HexDisplay::from(&payload));
                    signer.sign(payload)
                }
            });
            println!("Signature {:?}", signature);
            let xt = UncheckedExtrinsic::new_signed(
                raw_payload.0,
                from.into(),
                signature.into(),
                extra(index),
            )
            .encode();

            println!("0x{}", hex::encode(&xt));
            rt::run(rt::lazy(|| {
                let uri = "http://localhost:9933";

                http::connect(uri)
                    .and_then(|client: AuthorClient<Hash, Hash>| submit(client, xt))
                    .map_err(|e| {
                        println!("Error: {:?}", e);
                    })
            }))
        }
        ("start", Some(_matches)) => {
            let (mut rt, client) = setup();
            let stream = rt.block_on(client.subscribe_events()).unwrap();
            let block_events = stream
                .for_each(|change_set| {
                    change_set
                        .changes
                        .iter()
                        .filter_map(|(_key, data)| {
                            data.as_ref().map(|data| Decode::decode(&mut &data.0[..]))
                        })
                        .for_each(
                            |result: Result<
                                Vec<
                                    system::EventRecord<
                                        <Runtime as System>::Event,
                                        <Runtime as System>::Hash,
                                    >,
                                >,
                                codec::Error,
                            >| {
                                let _ = result.map(|events| {
                                    events.iter().for_each(|event| match &event.event {
                                        ibc_node_runtime::Event::ibc(
                                            IbcEvent::InterchainMessageSent(id, message),
                                        ) => {
                                            println!("id: {}, message: {:?}", id, message);
                                            // TODO: find the corresponding genesis_hash and rpc address according to para_id
                                        }
                                        _ => {}
                                    })
                                });
                            },
                        );
                    Ok(())
                })
                .map_err(|_| ());
            tokio::run(block_events);
        }
        ("interchain-message", Some(matches)) => {
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

            let para_id = matches
                .value_of("para-id")
                .expect("para-id is required; thus it can't be None; qed");
            let para_id = str::parse::<ParaId>(para_id)
                .expect("Invalid 'para-id' parameter; expecting an integer.");

            let message = matches
                .value_of("message")
                .expect("message is required; thus it can't be None; qed");
            let message: Vec<u8> = hex::decode(message).expect("Invalid message");

            let from = AddressPublic::from_raw(pair.public().0);
            let signer = pair.clone();

            let function = Call::Ibc(IbcCall::interchain_message(para_id, message));
            let extra = |i: Index| {
                (
                    system::CheckGenesis::<ibc_node_runtime::Runtime>::new(),
                    system::CheckNonce::<ibc_node_runtime::Runtime>::from(i),
                )
            };

            let raw_payload = (function, extra(index), genesis_hash);
            let signature = raw_payload.using_encoded(|payload| {
                if payload.len() > 256 {
                    signer.sign(&blake2_256(payload)[..])
                } else {
                    println!("Signing {}", HexDisplay::from(&payload));
                    signer.sign(payload)
                }
            });
            println!("Signature {:?}", signature);
            let xt = UncheckedExtrinsic::new_signed(
                raw_payload.0,
                from.into(),
                signature.into(),
                extra(index),
            )
            .encode();

            println!("0x{}", hex::encode(&xt));
            rt::run(rt::lazy(|| {
                let uri = "http://localhost:9933";

                http::connect(uri)
                    .and_then(|client: AuthorClient<Hash, Hash>| submit(client, xt))
                    .map_err(|e| {
                        println!("Error: {:?}", e);
                    })
            }))
        }
        _ => print_usage(&matches),
    }
}

fn submit(
    client: AuthorClient<Hash, Hash>,
    xt: Vec<u8>,
) -> impl Future<Item = (), Error = RpcError> {
    client.submit_extrinsic(xt.into()).map(|hash| {
        println!("return hash: {:?}", hash);
    })
}

struct Runtime;

impl System for Runtime {
    type Index = <ibc_node_runtime::Runtime as system::Trait>::Index;
    type BlockNumber = <ibc_node_runtime::Runtime as system::Trait>::BlockNumber;
    type Hash = <ibc_node_runtime::Runtime as system::Trait>::Hash;
    type Hashing = <ibc_node_runtime::Runtime as system::Trait>::Hashing;
    type AccountId = <ibc_node_runtime::Runtime as system::Trait>::AccountId;
    type Lookup = <ibc_node_runtime::Runtime as system::Trait>::Lookup;
    type Header = <ibc_node_runtime::Runtime as system::Trait>::Header;
    type Event = <ibc_node_runtime::Runtime as system::Trait>::Event;

    type SignedExtra = (
        system::CheckGenesis<ibc_node_runtime::Runtime>,
        system::CheckNonce<ibc_node_runtime::Runtime>,
    );
    fn extra(nonce: Self::Index) -> Self::SignedExtra {
        (
            system::CheckGenesis::<ibc_node_runtime::Runtime>::new(),
            system::CheckNonce::<ibc_node_runtime::Runtime>::from(nonce),
        )
    }
}

fn setup() -> (tokio::runtime::Runtime, Client<Runtime>) {
    env_logger::try_init().ok();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let client_future = ClientBuilder::<Runtime>::new().build();
    let client = rt.block_on(client_future).unwrap();
    (rt, client)
}
