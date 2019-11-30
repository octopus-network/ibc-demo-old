use clap::{App, ArgMatches, SubCommand};
use codec::{Decode, Encode};
use frame::{ibc, NodeRuntime as Runtime};
use futures::{future::Future, stream::Stream};
use keyring::AccountKeyring;
use substrate_subxt::{system::System, ClientBuilder};
use tokio::runtime::TaskExecutor;
use url::Url;

fn execute(matches: ArgMatches) {
    match matches.subcommand() {
        ("run", Some(matches)) => {
            let addr1 = matches
                .value_of("addr1")
                .expect("The address of chain-1 is required; qed");
            let addr1 = Url::parse(&format!("ws://{}", addr1)).expect("Is valid url; qed");
            let addr2 = matches
                .value_of("addr2")
                .expect("The address of chain-2 is required; qed");
            let addr2 = Url::parse(&format!("ws://{}", addr2)).expect("Is valid url; qed");
            run(addr1, addr2);
        }
        _ => print_usage(&matches),
    }
}

fn print_usage(matches: &ArgMatches) {
    println!("{}", matches.usage());
}

fn main() {
    let matches = App::new("relayer")
        .author("Cdot Network <ys@cdot.network>")
        .about("Relayer is an off-chain process to relay IBC packets between two demo chains")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommands(vec![SubCommand::with_name("run")
            .about("Start a relayer process")
            .args_from_usage(
                "
<addr1> 'The address of demo chain-1'
<addr2> 'The address of demo chain-2'
",
            )])
        .get_matches();
    execute(matches);
}

fn run(addr1: Url, addr2: Url) {
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
        update_client(&executor, addr2.clone(), 0, block_header.encode());
        Ok(())
    });

    let executor = rt.executor();
    executor.spawn(blocks.map_err(|_| ()));

    type EventRecords =
        Vec<frame_system::EventRecord<node_runtime::Event, <Runtime as System>::Hash>>;

    let stream = rt.block_on(client.subscribe_events()).unwrap();
    let block_events = stream
        .for_each(|change_set| {
            change_set
                .changes
                .iter()
                .filter_map(|(_key, data)| {
                    data.as_ref().map(|data| Decode::decode(&mut &data.0[..]))
                })
                .filter_map(|result: Result<EventRecords, codec::Error>| result.ok())
                .for_each(|events| {
                    events.into_iter().for_each(|event| match event.event {
                        node_runtime::Event::template(
                            node_runtime::TemplateRawEvent::SomethingStored(something, who),
                        ) => {
                            let block_hash = change_set.block.clone();
                            println!(
                                "block_hash: {:?}, something: {}, who: {:?}",
                                block_hash, something, who
                            );
                        }
                        _ => {}
                    });
                });
            Ok(())
        })
        .map_err(|e| println!("{:?}", e));
    rt.spawn(block_events);
    rt.shutdown_on_idle().wait().unwrap();
}

fn update_client(executor: &TaskExecutor, addr: Url, id: u32, header: Vec<u8>) {
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
