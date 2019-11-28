use frame::{ibc, NodeRuntime as Runtime};
use futures::{future::Future, stream::Stream};
use keyring::AccountKeyring;
use parity_scale_codec::Encode;
use substrate_subxt::ClientBuilder;
use tokio::runtime::TaskExecutor;
use url::Url;

fn main() {
    let addr1 = String::from("127.0.0.1:9944");
    let addr1 = Url::parse(&format!("ws://{}", addr1)).expect("Is valid url; qed");

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let executor = rt.executor();

    let client_future = ClientBuilder::<Runtime>::new()
        .set_url(addr1.clone())
        .build();
    let client = rt.block_on(client_future).unwrap();

    let stream = rt.block_on(client.subscribe_finalized_blocks()).unwrap();
    let blocks = stream.for_each(move |block_header| {
        let header_number = block_header.number;
        let state_root = block_header.state_root;
        let block_hash = block_header.hash();
        println!("header_number: {:?}", header_number);
        println!("state_root: {:?}", state_root);
        println!("block_hash: {:?}", block_hash);
        update_client(executor.clone(), addr1.clone(), 0, block_header.encode());
        Ok(())
    });

    rt.spawn(blocks.map_err(|_| ()));
    rt.shutdown_on_idle().wait().unwrap();
}

fn update_client(executor: TaskExecutor, addr: Url, id: u32, header: Vec<u8>) {
    let signer = AccountKeyring::Alice.pair();
    let call = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .and_then(|client| client.xt(signer, None))
        .and_then(move |xt| xt.submit(ibc::update_client(id, header)))
        .map(|_| ())
        .map_err(|e| println!("{:?}", e));

    executor.spawn(call);
}
