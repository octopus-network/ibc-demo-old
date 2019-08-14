use codec::Encode;
use futures::Future;
use hex_literal::hex;
use hyper::rt;
use ibc_node_runtime::{BalancesCall, Call, Runtime, UncheckedExtrinsic};
use jsonrpc_core_client::{transports::http, RpcError};
use keyring::AccountKeyring;
use node_primitives::Hash;
use primitives::{blake2_256, hexdisplay::HexDisplay, sr25519::Public as AddressPublic, Pair};
use rpc::author::AuthorClient;

fn main() {
    env_logger::init();

    rt::run(rt::lazy(|| {
        let uri = "http://localhost:9933";

        http::connect(uri)
            .and_then(|client: AuthorClient<Hash, Hash>| submit(client))
            .map_err(|e| {
                println!("Error: {:?}", e);
            })
    }))
}

fn uxt() -> UncheckedExtrinsic {
    let amount = 666;
    let alice = AccountKeyring::Alice.pair();
    let bob = AccountKeyring::Bob.pair();

    let to = AddressPublic::from_raw(bob.public().0);
    let from = AddressPublic::from_raw(alice.public().0);
    let genesis_hash: Hash =
        hex!["a3f956cd7ca88cef8c4a21107027e3882bc86bd8f217b0ea8b054e002d7800f2"].into();
    let signer = alice.clone();

    let function = Call::Balances(BalancesCall::transfer(to.into(), amount));
    let extra = (
        system::CheckGenesis::<Runtime>::new(),
        system::CheckWeight::<Runtime>::new(),
    );

    let raw_payload = (function, extra.clone(), genesis_hash);
    let signature = raw_payload.using_encoded(|payload| {
        if payload.len() > 256 {
            signer.sign(&blake2_256(payload)[..])
        } else {
            println!("payload: {}", HexDisplay::from(&payload.as_ref()));
            signer.sign(payload)
        }
    });
    println!("signature: {:?}", signature);
    UncheckedExtrinsic::new_signed(raw_payload.0, from.into(), signature.into(), extra)
}

fn submit(client: AuthorClient<Hash, Hash>) -> impl Future<Item = (), Error = RpcError> {
    let xt = uxt().encode();
    client.submit_extrinsic(xt.into()).map(|hash| {
        println!("return hash: {:?}", hash);
    })
}
