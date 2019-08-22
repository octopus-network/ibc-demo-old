use clap::load_yaml;
use codec::Decode;
use futures::stream::Stream;
use futures::Future;
use ibc_node_runtime::{self, ibc::ParaId, ibc::RawEvent as IbcEvent};
use keyring::AccountKeyring;
use node_primitives::{Hash, Index};
use primitives::{hexdisplay::HexDisplay, sr25519, Pair};
use runtime_primitives::generic::Era;
use substrate_subxt::{
    srml::{
        ibc::{Ibc, IbcXt},
        system::System,
    },
    Client, ClientBuilder,
};

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
                                    srml_system::EventRecord<
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

            let (mut rt, client) = setup();

            let signer = AccountKeyring::Alice.pair();
            let mut xt = rt.block_on(client.xt(signer, Some(index))).unwrap();

            let transfer = xt
                .ibc(|calls| calls.interchain_message(para_id, message))
                .submit();
            rt.block_on(transfer).unwrap();
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
        // srml_system::CheckVersion<ibc_node_runtime::Runtime>,
        srml_system::CheckGenesis<ibc_node_runtime::Runtime>,
        srml_system::CheckEra<ibc_node_runtime::Runtime>,
        srml_system::CheckNonce<ibc_node_runtime::Runtime>,
        srml_system::CheckWeight<ibc_node_runtime::Runtime>,
        srml_balances::TakeFees<ibc_node_runtime::Runtime>,
    );
    fn extra(nonce: Self::Index) -> Self::SignedExtra {
        (
            // srml_system::CheckVersion::<ibc_node_runtime::Runtime>::new(),
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
