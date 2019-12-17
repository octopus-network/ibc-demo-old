use calls::{
    ibc::{self, IbcStore},
    NodeRuntime as Runtime,
};
use clap::{App, ArgMatches, SubCommand};
use codec::Decode;
use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::stream::StreamExt;
use sp_core::{Blake2Hasher, Hasher};
use sp_keyring::AccountKeyring;
use std::error::Error;
use substrate_subxt::{system::System, ClientBuilder};
use url::Url;

fn execute(matches: ArgMatches) {
    match matches.subcommand() {
        ("run", Some(matches)) => {
            let appia_addr = matches
                .value_of("appia-addr")
                .expect("The address of appia chain is required; qed");
            let appia_addr =
                Url::parse(&format!("ws://{}", appia_addr)).expect("Is valid url; qed");
            let flaminia_addr = matches
                .value_of("flaminia-addr")
                .expect("The address of flaminia chain is required; qed");
            let flaminia_addr =
                Url::parse(&format!("ws://{}", flaminia_addr)).expect("Is valid url; qed");
            tokio_compat::run_std(async {
                run(appia_addr, flaminia_addr)
                    .await
                    .expect("Failed to run relayer");
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
<appia-addr> 'The address of demo chain - Appia'
<flaminia-addr> 'The address of demo chain - Flaminia'
",
            )])
        .get_matches();
    execute(matches);
}

async fn run(appia_addr: Url, flaminia_addr: Url) -> Result<(), Box<dyn Error>> {
    let appia_client = ClientBuilder::<Runtime>::new()
        .set_url(appia_addr.clone())
        .build()
        .compat()
        .await?;

    let flaminia_client = ClientBuilder::<Runtime>::new()
        .set_url(flaminia_addr.clone())
        .build()
        .compat()
        .await?;

    let mut block_headers = appia_client
        .subscribe_finalized_blocks()
        .compat()
        .await?
        .compat();
    let identifier = Blake2Hasher::hash(b"appia");
    tokio::spawn(async move {
        while let Some(Ok(block_header)) = block_headers.next().await {
            let header_number = block_header.number;
            let state_root = block_header.state_root;
            let block_hash = block_header.hash();
            println!("header_number: {:?}", header_number);
            println!("state_root: {:?}", state_root);
            println!("block_hash: {:?}", block_hash);
            let map = flaminia_client
                .query_client_consensus_state(&identifier)
                .compat()
                .await
                .unwrap();
            println!("Clients: {:?}", map);
            if map.consensus_state.height < header_number {
                let datagram = pallet_ibc::Datagram::ClientUpdate {
                    identifier: identifier,
                    header: block_header,
                };
                let signer = AccountKeyring::Alice.pair();
                let xt = flaminia_client.xt(signer, None).compat().await.unwrap();
                if let Err(e) = xt.submit(ibc::submit_datagram(datagram)).compat().await {
                    println!("failed to update_client; error = {}", e);
                }
            }
        }
    });

    type EventRecords =
        Vec<frame_system::EventRecord<node_runtime::Event, <Runtime as System>::Hash>>;

    let mut block_events = appia_client.subscribe_events().compat().await?.compat();
    let addr2 = flaminia_addr.clone();
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
                            let addr2 = addr2.clone();
                            tokio::spawn(async move {
                                if let Err(e) =
                                    recv_packet(addr2.clone(), vec![], vec![vec![]], 0).await
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
