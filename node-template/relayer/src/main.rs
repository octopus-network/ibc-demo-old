use calls::{
    ibc::{self, IbcStore},
    NodeRuntime as Runtime,
};
use clap::{App, ArgMatches, SubCommand};
use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::stream::StreamExt;
use sp_core::{Blake2Hasher, Hasher, H256};
use sp_keyring::AccountKeyring;
use sp_runtime::generic;
use std::error::Error;
use substrate_subxt::{Client, ClientBuilder};
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

    let mut appia_block_headers = appia_client
        .subscribe_finalized_blocks()
        .compat()
        .await?
        .compat();

    let flaminia_client = ClientBuilder::<Runtime>::new()
        .set_url(flaminia_addr.clone())
        .build()
        .compat()
        .await?;

    let mut flaminia_block_headers = flaminia_client
        .subscribe_finalized_blocks()
        .compat()
        .await?
        .compat();

    let appia = Blake2Hasher::hash(b"appia");
    let flaminia = Blake2Hasher::hash(b"flaminia");

    let appia_client_1 = appia_client.clone();
    let flaminia_client_1 = flaminia_client.clone();
    tokio::spawn(async move {
        while let Some(Ok(block_header)) = appia_block_headers.next().await {
            if let Err(e) = relay(
                block_header,
                &appia_client,
                flaminia,
                &flaminia_client,
                appia,
            )
            .await
            {
                println!("failed to relay; error = {}", e);
            }
        }
    });

    tokio::spawn(async move {
        while let Some(Ok(block_header)) = flaminia_block_headers.next().await {
            if let Err(e) = relay(
                block_header,
                &flaminia_client_1,
                appia,
                &appia_client_1,
                flaminia,
            )
            .await
            {
                println!("failed to relay; error = {}", e);
            }
        }
    });
    Ok(())
}
async fn relay(
    block_header: generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
    client: &Client<Runtime>,
    client_identifier: H256,
    counterparty_client: &Client<Runtime>,
    counterparty_client_identifier: H256,
) -> Result<(), Box<dyn Error>> {
    let header_number = block_header.number;
    let state_root = block_header.state_root;
    let block_hash = block_header.hash();
    println!("header_number: {:?}", header_number);
    println!("state_root: {:?}", state_root);
    println!("block_hash: {:?}", block_hash);
    let map = counterparty_client
        .query_client_consensus_state(&counterparty_client_identifier)
        .compat()
        .await?;
    println!("query client on counterparty chain: {:?}", map);
    let signer = AccountKeyring::Alice.pair();
    let mut counterparty_xt = counterparty_client.xt(signer, None).compat().await?;
    if map.consensus_state.height < header_number {
        let datagram = pallet_ibc::Datagram::ClientUpdate {
            identifier: counterparty_client_identifier,
            header: block_header,
        };
        if let Err(e) = counterparty_xt
            .submit(ibc::submit_datagram(datagram))
            .compat()
            .await
        {
            println!("failed to update_client; error = {}", e);
        }
    }
    let connections = client
        .get_connections_using_client(&client_identifier)
        .compat()
        .await?;
    println!("connections: {:?}", connections);
    for connection in connections.iter() {
        let connection_end = client.get_connection(&connection).compat().await?;
        println!("connection_end: {:?}", connection_end);
        let remote_connection_end = counterparty_client
            .get_connection(&connection_end.counterparty_connection_identifier)
            .compat()
            .await?;
        println!("remote connection_end: {:?}", remote_connection_end);
        // TODO: remote_connection_end == None ??
        if connection_end.state == pallet_ibc::ConnectionState::Init
            && remote_connection_end.state == pallet_ibc::ConnectionState::None
        {
            let datagram = pallet_ibc::Datagram::ConnOpenTry {
                desired_identifier: connection_end.counterparty_connection_identifier,
                counterparty_connection_identifier: *connection,
                counterparty_client_identifier: client_identifier,
                client_identifier: counterparty_client_identifier,
                version: vec![],
                counterparty_version: vec![],
                proof_init: vec![],
                proof_consensus: vec![],
                proof_height: header_number,
                consensus_height: 0, // TODO: local consensus state height
            };
            if let Err(e) = counterparty_xt
                .increment_nonce()
                .submit(ibc::submit_datagram(datagram))
                .compat()
                .await
            {
                println!("failed to send ConnOpenTry; error = {}", e);
            }
        } else if connection_end.state == pallet_ibc::ConnectionState::TryOpen
            && remote_connection_end.state == pallet_ibc::ConnectionState::Init
        {
            let datagram = pallet_ibc::Datagram::ConnOpenAck {
                identifier: connection_end.counterparty_connection_identifier,
                version: vec![],
                proof_try: vec![],
                proof_consensus: vec![],
                proof_height: 0,
                consensus_height: 0,
            };
            if let Err(e) = counterparty_xt
                .increment_nonce()
                .submit(ibc::submit_datagram(datagram))
                .compat()
                .await
            {
                println!("failed to send ConnOpenAck; error = {}", e);
            }
        } else if connection_end.state == pallet_ibc::ConnectionState::Open
            && remote_connection_end.state == pallet_ibc::ConnectionState::TryOpen
        {
            let datagram = pallet_ibc::Datagram::ConnOpenConfirm {
                identifier: connection_end.counterparty_connection_identifier,
                proof_ack: vec![],
                proof_height: 0,
            };
            if let Err(e) = counterparty_xt
                .increment_nonce()
                .submit(ibc::submit_datagram(datagram))
                .compat()
                .await
            {
                println!("failed to send ConnOpenConfirm; error = {}", e);
            }
        }
    }
    Ok(())
}
