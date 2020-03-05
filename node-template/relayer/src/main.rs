use calls::{
    ibc::{self, IbcStore},
    NodeRuntime as Runtime,
};
use clap::{App, ArgMatches, SubCommand};
use codec::Decode;
use log::{debug, error, info};
use pallet_ibc::{ChannelState, ConnectionState, Datagram, Packet};
use sp_core::{storage::StorageKey, twox_128, Blake2Hasher, Hasher, H256};
use sp_finality_grandpa::GRANDPA_AUTHORITIES_KEY;
use sp_keyring::AccountKeyring;
use sp_rpc::number::NumberOrHex;
use sp_runtime::generic;
use sp_storage::StorageChangeSet;
use std::error::Error;
use std::sync::mpsc::{channel, Sender};
use substrate_subxt::{system::System, Client, ClientBuilder};

type EventRecords = Vec<system::EventRecord<node_runtime::Event, <Runtime as System>::Hash>>;

fn execute(matches: ArgMatches) {
    match matches.subcommand() {
        ("run", Some(matches)) => {
            let appia_addr = matches
                .value_of("appia-addr")
                .expect("The address of appia chain is required; qed");
            let appia_addr = format!("ws://{}", appia_addr);
            let flaminia_addr = matches
                .value_of("flaminia-addr")
                .expect("The address of flaminia chain is required; qed");
            let flaminia_addr = format!("ws://{}", flaminia_addr);
            let result = async_std::task::block_on(run(&appia_addr, &flaminia_addr));
            println!("run: {:?}", result);
        }
        _ => print_usage(&matches),
    }
}

fn print_usage(matches: &ArgMatches) {
    println!("{}", matches.usage());
}

fn main() {
    env_logger::init();
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

async fn run(appia_addr: &str, flaminia_addr: &str) -> Result<(), Box<dyn Error>> {
    let appia_client = ClientBuilder::<Runtime>::new()
        .set_url(appia_addr)
        .build()
        .await?;
    let flaminia_client = ClientBuilder::<Runtime>::new()
        .set_url(flaminia_addr)
        .build()
        .await?;

    let mut appia_block_headers = appia_client.subscribe_finalized_blocks().await?;
    let mut flaminia_block_headers = flaminia_client.subscribe_finalized_blocks().await?;

    let appia = Blake2Hasher::hash(b"appia");
    let flaminia = Blake2Hasher::hash(b"flaminia");

    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    {
        let appia_client = appia_client.clone();
        let flaminia_client = flaminia_client.clone();
        async_std::task::spawn(async move {
            loop {
                let block_header = appia_block_headers.next().await;
                let tx = tx1.clone();
                if let Err(e) = relay(
                    "appia",
                    tx,
                    block_header,
                    &appia_client,
                    flaminia,
                    &flaminia_client,
                    appia,
                )
                .await
                {
                    error!("[appia] failed to relay; error = {}", e);
                }
            }
        });
    }

    {
        let appia_client = appia_client.clone();
        let flaminia_client = flaminia_client.clone();
        async_std::task::spawn(async move {
            loop {
                let block_header = flaminia_block_headers.next().await;
                let tx = tx2.clone();
                if let Err(e) = relay(
                    "flaminia",
                    tx,
                    block_header,
                    &flaminia_client,
                    appia,
                    &appia_client,
                    flaminia,
                )
                .await
                {
                    error!("[flaminia] failed to relay; error = {}", e);
                }
            }
        });
    }

    async_std::task::spawn(async move {
        let signer = AccountKeyring::Alice.pair();
        let mut xt = flaminia_client.xt(signer, None).await.unwrap();
        let mut first = true;
        loop {
            let datagram = rx1.recv().unwrap();
            match datagram {
                Datagram::ClientUpdate { .. } => {
                    info!("[relayer => flaminia] datagram: {:?}", datagram)
                }
                _ => info!("[relayer => flaminia] datagram: {:#?}", datagram),
            }
            if first {
                if let Err(e) = xt.submit(ibc::submit_datagram(datagram)).await {
                    error!(
                        "[relayer => flaminia] failed to send datagram; error = {}",
                        e
                    );
                }
                first = false;
                continue;
            }
            if let Err(e) = xt
                .increment_nonce()
                .submit(ibc::submit_datagram(datagram))
                .await
            {
                error!(
                    "[relayer => flaminia] failed to send datagram; error = {}",
                    e
                );
            }
        }
    });

    async_std::task::block_on(async move {
        let signer = AccountKeyring::Alice.pair();
        let mut xt = appia_client.xt(signer, None).await.unwrap();
        let mut first = true;
        loop {
            let datagram = rx2.recv().unwrap();
            match datagram {
                Datagram::ClientUpdate { .. } => {
                    info!("[relayer => appia] datagram: {:?}", datagram)
                }
                _ => info!("[relayer => appia] datagram: {:#?}", datagram),
            }
            if first {
                if let Err(e) = xt.submit(ibc::submit_datagram(datagram)).await {
                    error!("[relayer => appia] failed to send datagram; error = {}", e);
                }
                first = false;
                continue;
            }
            if let Err(e) = xt
                .increment_nonce()
                .submit(ibc::submit_datagram(datagram))
                .await
            {
                error!("[relayer => appia] failed to send datagram; error = {}", e);
            }
        }
    });

    Ok(())
}

async fn relay(
    chain_name: &str,
    tx: Sender<Datagram>,
    block_header: generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
    client: &Client<Runtime>,
    client_identifier: H256,
    counterparty_client: &Client<Runtime>,
    counterparty_client_identifier: H256,
) -> Result<(), Box<dyn Error>> {
    let mut storage_key = twox_128(b"System").to_vec();
    storage_key.extend(twox_128(b"Events").to_vec());
    let events_storage_key = StorageKey(storage_key);

    let header_number = block_header.number;
    let state_root = block_header.state_root;
    let block_hash = block_header.hash();
    info!("[{}] header_number: {:?}", chain_name, header_number);
    info!("[{}] state_root: {:?}", chain_name, state_root);
    info!("[{}] block_hash: {:?}", chain_name, block_hash);
    let map = counterparty_client
        .query_client(counterparty_client_identifier)
        .await?;
    debug!("[{}] query counterparty client: {:#?}", chain_name, map);
    if map.latest_height < header_number {
        for height in map.latest_height + 1..=header_number {
            let hash = client.block_hash(Some(NumberOrHex::Number(height))).await?;
            let signed_block = client.block(hash).await?;
            let authorities_proof = client
                .read_proof(hash.unwrap(), vec![GRANDPA_AUTHORITIES_KEY.to_vec()])
                .await?;
            if let Some(signed_block) = signed_block {
                if let Some(justification) = signed_block.justification {
                    let datagram = Datagram::ClientUpdate {
                        identifier: counterparty_client_identifier,
                        header: signed_block.block.header,
                        justification,
                        authorities_proof,
                    };
                    tx.send(datagram).unwrap();
                }
            }
        }
    }
    let connections = client
        .get_connections_using_client(client_identifier)
        .await?;
    info!("[{}] connections: {:?}", chain_name, connections);
    for connection in connections.iter() {
        let connection_end = client.get_connection(*connection).await?;
        debug!("[{}] connection_end: {:#?}", chain_name, connection_end);
        let remote_connection_end = counterparty_client
            .get_connection(connection_end.counterparty_connection_identifier)
            .await?;
        debug!(
            "[{}] remote_connection_end: {:#?}",
            chain_name, remote_connection_end
        );
        info!(
            "[{}] connection state: {:?}, remote connection state: {:?}",
            chain_name, connection_end.state, remote_connection_end.state
        );
        // TODO: remote_connection_end == None ??
        if connection_end.state == ConnectionState::Init
            && remote_connection_end.state == ConnectionState::None
        {
            let datagram = Datagram::ConnOpenTry {
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
            tx.send(datagram).unwrap();
        } else if connection_end.state == ConnectionState::TryOpen
            && remote_connection_end.state == ConnectionState::Init
        {
            let datagram = Datagram::ConnOpenAck {
                identifier: connection_end.counterparty_connection_identifier,
                version: vec![],
                proof_try: vec![],
                proof_consensus: vec![],
                proof_height: 0,
                consensus_height: 0,
            };
            tx.send(datagram).unwrap();
        } else if connection_end.state == ConnectionState::Open
            && remote_connection_end.state == ConnectionState::TryOpen
        {
            let datagram = Datagram::ConnOpenConfirm {
                identifier: connection_end.counterparty_connection_identifier,
                proof_ack: vec![],
                proof_height: 0,
            };
            tx.send(datagram).unwrap();
        }
    }
    let channels = client.get_channel_keys().await?;
    info!("[{}] channels: {:?}", chain_name, channels);
    for channel in channels.iter() {
        let channel_end = client
            .get_channels_using_connections(vec![], channel.0.clone(), channel.1)
            .await?;
        debug!("[{}] channel_end: {:#?}", chain_name, channel_end);
        let remote_channel_end = counterparty_client
            .get_channel((
                channel_end.counterparty_port_identifier.clone(),
                channel_end.counterparty_channel_identifier,
            ))
            .await?;
        debug!(
            "[{}] remote_channel_end: {:#?}",
            chain_name, remote_channel_end
        );
        info!(
            "[{}] channle state: {:?}, remote channel state: {:?}",
            chain_name, channel_end.state, remote_channel_end.state
        );
        if channel_end.state == ChannelState::Init && remote_channel_end.state == ChannelState::None
        {
            let connection_end = client
                .get_connection(channel_end.connection_hops[0])
                .await?;
            let datagram = Datagram::ChanOpenTry {
                order: channel_end.ordering,
                // connection_hops: channel_end.connection_hops.into_iter().rev().collect(), // ??
                connection_hops: vec![connection_end.counterparty_connection_identifier],
                port_identifier: channel_end.counterparty_port_identifier,
                channel_identifier: channel_end.counterparty_channel_identifier,
                counterparty_port_identifier: channel.0.clone(),
                counterparty_channel_identifier: channel.1,
                version: channel_end.version.clone(),
                counterparty_version: channel_end.version,
                proof_init: vec![],
                proof_height: 0,
            };
            tx.send(datagram).unwrap();
        } else if channel_end.state == ChannelState::TryOpen
            && remote_channel_end.state == ChannelState::Init
        {
            let datagram = Datagram::ChanOpenAck {
                port_identifier: channel_end.counterparty_port_identifier,
                channel_identifier: channel_end.counterparty_channel_identifier,
                version: remote_channel_end.version,
                proof_try: vec![],
                proof_height: 0,
            };
            tx.send(datagram).unwrap();
        } else if channel_end.state == ChannelState::Open
            && remote_channel_end.state == ChannelState::TryOpen
        {
            let datagram = Datagram::ChanOpenConfirm {
                port_identifier: channel_end.counterparty_port_identifier,
                channel_identifier: channel_end.counterparty_channel_identifier,
                proof_ack: vec![],
                proof_height: 0,
            };
            tx.send(datagram).unwrap();
        }
    }

    let change_sets: Vec<StorageChangeSet<H256>> = client
        .query_storage(vec![events_storage_key], block_hash, None)
        .await?;
    info!("length of change_sets: {:?}", change_sets.len());
    info!("change_sets: {:?}", change_sets);
    change_sets.iter().for_each(|change_set| {
        change_set
            .changes
            .iter()
            .filter_map(|(_key, data)| data.as_ref().map(|data| Decode::decode(&mut &data.0[..])))
            .filter_map(|result: Result<EventRecords, codec::Error>| result.ok())
            .for_each(|events| {
                events.into_iter().for_each(|event| match event.event {
                    node_runtime::Event::ibc(pallet_ibc::RawEvent::SendPacket(
                        sequence,
                        data,
                        timeout_height,
                        source_port,
                        source_channel,
                        dest_port,
                        dest_channel,
                    )) => {
                        let block_hash = change_set.block.clone();
                        info!(
                            "[{}] SendPacket hash: {:?}, sequence: {}, data: {:?}, \
                             timeout_height: {}, source_port: {:?}, source_channel: {:?}, \
                             dest_port: {:?}, dest_channel: {:?}",
                            chain_name,
                            block_hash,
                            sequence,
                            data,
                            timeout_height,
                            source_port,
                            source_channel,
                            dest_port,
                            dest_channel
                        );
                        let packet_data = Packet {
                            sequence,
                            timeout_height,
                            source_port,
                            source_channel,
                            dest_port,
                            dest_channel,
                            data,
                        };
                        let datagram = Datagram::PacketRecv {
                            packet: packet_data,
                            proof: vec![],
                            proof_height: 0,
                        };
                        tx.send(datagram).unwrap();
                    }
                    node_runtime::Event::ibc(pallet_ibc::RawEvent::RecvPacket(
                        sequence,
                        data,
                        timeout_height,
                        source_port,
                        source_channel,
                        dest_port,
                        dest_channel,
                    )) => {
                        info!(
                            "[{}] RecvPacket sequence: {}, data: {:?}, timeout_height: {}, \
                             source_port: {:?}, source_channel: {:?}, dest_port: {:?}, \
                             dest_channel: {:?}",
                            chain_name,
                            sequence,
                            data,
                            timeout_height,
                            source_port,
                            source_channel,
                            dest_port,
                            dest_channel
                        );
                        // relay packet acknowledgement with this sequence number
                        let packet_data = Packet {
                            sequence,
                            timeout_height,
                            source_port,
                            source_channel,
                            dest_port,
                            dest_channel,
                            data,
                        };
                        let datagram = Datagram::PacketAcknowledgement {
                            packet: packet_data,
                            acknowledgement: vec![],
                            proof: vec![],
                            proof_height: 0,
                        };
                        tx.send(datagram).unwrap();
                    }
                    _ => {}
                });
            });
    });

    Ok(())
}
