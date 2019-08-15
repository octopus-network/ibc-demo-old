use clap::load_yaml;
use codec::{Decode, Encode};
use futures::Future;
use hyper::rt;
use ibc_node_runtime::{BalancesCall, Call, Runtime, UncheckedExtrinsic};
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
        ("set-head", Some(matches)) => {
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
            let amount = 666;
            let bob = AccountKeyring::Bob.pair();

            let to = AddressPublic::from_raw(bob.public().0);
            let from = AddressPublic::from_raw(pair.public().0);
            let signer = pair.clone();

            let function = Call::Balances(BalancesCall::transfer(to.into(), amount));
            let extra = |i: Index| {
                (
                    system::CheckGenesis::<Runtime>::new(),
                    system::CheckNonce::<Runtime>::from(i),
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
