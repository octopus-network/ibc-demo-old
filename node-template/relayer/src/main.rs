use calls::{ibc, NodeRuntime as Runtime};
use clap::{App, ArgMatches, SubCommand};
use codec::{Decode, Encode};
use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::stream::StreamExt;
use sp_keyring::AccountKeyring;
use std::error::Error;
use substrate_subxt::{system::System, ClientBuilder};
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
            tokio_compat::run_std(async {
                run(addr1, addr2).await.expect("Failed to run relayer");
            });
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

async fn run(addr1: Url, addr2: Url) -> Result<(), Box<dyn Error>> {
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr1.clone())
        .build()
        .compat()
        .await?;

    let mut block_headers = client.subscribe_finalized_blocks().compat().await?.compat();
    let addr2_1 = addr2.clone();
    tokio::spawn(async move {
        while let Some(Ok(block_header)) = block_headers.next().await {
            let header_number = block_header.number;
            let state_root = block_header.state_root;
            let block_hash = block_header.hash();
            println!("header_number: {:?}", header_number);
            println!("state_root: {:?}", state_root);
            println!("block_hash: {:?}", block_hash);
            let addr2_1 = addr2_1.clone();
            tokio::spawn(async move {
                let datagram = pallet_ibc::Datagram::ClientUpdate {
                    identifier: 0,
                    header: block_header,
                };
                if let Err(e) = submit_datagram(addr2_1.clone(), datagram).await {
                    println!("failed to update_client; error = {}", e);
                }
            });
        }
    });

    type EventRecords =
        Vec<frame_system::EventRecord<node_runtime::Event, <Runtime as System>::Hash>>;

    let mut block_events = client.subscribe_events().compat().await?.compat();
    let addr2_2 = addr2.clone();
    tokio::spawn(async move {
        while let Some(Ok(change_set)) = block_events.next().await {
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
                            let addr2_2 = addr2_2.clone();
                            tokio::spawn(async move {
                                if let Err(e) =
                                    recv_packet(addr2_2.clone(), vec![], vec![vec![]], 0).await
                                {
                                    println!("failed to recv_packet; error = {}", e);
                                }
                            });
                        }
                        _ => {}
                    });
                });
        }
    });
    Ok(())
}

async fn submit_datagram(
    addr: Url,
    datagram: pallet_ibc::Datagram<<Runtime as System>::Header>,
) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Alice.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(ibc::submit_datagram::<Runtime>(datagram))
        .compat()
        .await?;
    Ok(())
}

async fn update_client(addr: Url, id: u32, header: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Alice.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(ibc::update_client(id, header)).compat().await?;
    Ok(())
}

async fn recv_packet(
    addr: Url,
    packet: Vec<u8>,
    proof: Vec<Vec<u8>>,
    proof_height: <Runtime as System>::BlockNumber,
) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(ibc::recv_packet::<Runtime>(packet, proof, proof_height))
        .compat()
        .await?;
    Ok(())
}
