use calls::{
    ibc::{self, IbcStore},
    NodeRuntime as Runtime,
};
use clap::{App, Arg, ArgMatches};
use codec::Decode;
use log::{debug, error, info};
use pallet_ibc::{ChannelState, ConnectionState, Datagram, Header, Packet};
use serde_derive::Deserialize;
use sp_core::{storage::StorageKey, twox_128, Blake2Hasher, Hasher, H256};
use sp_finality_grandpa::GRANDPA_AUTHORITIES_KEY;
use sp_keyring::AccountKeyring;
use sp_runtime::generic;
use sp_storage::StorageChangeSet;
use sp_trie::StorageProof;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Sender};
use substrate_subxt::{system::System, BlockNumber, Client, ClientBuilder};

#[derive(Debug, Deserialize)]
struct Config {
    chains: Vec<ChainConfig>,
    relay: Vec<RelayConfig>,
}

#[derive(Debug, Deserialize)]
struct ChainConfig {
    name: String,
    endpoint: String,
    client_identifier: String,
}

#[derive(Debug, Deserialize)]
struct RelayConfig {
    from: String,
    to: String,
}

type EventRecords = Vec<system::EventRecord<node_runtime::Event, <Runtime as System>::Hash>>;

fn execute(matches: ArgMatches) {
    let file_path = matches.value_of("config").unwrap();
    let mut file = File::open(file_path).expect("config.toml not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("can not read config.toml");
    let config: Config = toml::from_str(&contents).expect("can not parse config.toml");
    println!("config: {:#?}", config);
    let result = async_std::task::block_on(run("ws://127.0.0.1:9944", "ws://127.0.0.1:8844"));
    println!("run: {:?}", result);
}

fn print_usage(matches: &ArgMatches) {
    println!("{}", matches.usage());
}

fn main() {
    env_logger::init();
    let matches = App::new("relayer")
        .author("Cdot Network <ys@cdot.network>")
        .about("Relayer is an off-chain process to relay IBC packets between chains")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .required(true),
        )
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
                    appia,
                    &flaminia_client,
                    flaminia,
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
                    flaminia,
                    &appia_client,
                    appia,
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
                    debug!("[relayer => flaminia] datagram: {:?}", datagram)
                }
                _ => debug!("[relayer => flaminia] datagram: {:#?}", datagram),
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
                    debug!("[relayer => appia] datagram: {:?}", datagram)
                }
                _ => debug!("[relayer => appia] datagram: {:#?}", datagram),
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

    let block_number = block_header.number;
    let state_root = block_header.state_root;
    let block_hash = block_header.hash();
    debug!("[{}] block_number: {}", chain_name, block_number);
    debug!("[{}] state_root: {:?}", chain_name, state_root);
    debug!("[{}] block_hash: {:?}", chain_name, block_hash);
    let client_state = client.query_client(None, client_identifier).await?;
    let counterparty_block_hash = counterparty_client
        .block_hash(Some(BlockNumber::from(client_state.latest_height)))
        .await?;
    info!(
        "[{}] client latest height: {}",
        chain_name, client_state.latest_height
    );
    let map = counterparty_client
        .query_client(None, counterparty_client_identifier)
        .await?;
    if map.latest_height < block_number {
        for height in map.latest_height + 1..=block_number {
            let hash = client.block_hash(Some(BlockNumber::from(height))).await?;
            let signed_block = client.block(hash).await?;
            let authorities_proof = client
                .read_proof(
                    vec![StorageKey(GRANDPA_AUTHORITIES_KEY.to_vec())],
                    Some(hash.unwrap()),
                )
                .await?;
            if let Some(signed_block) = signed_block {
                if let Some(justification) = signed_block.justification {
                    let datagram = Datagram::ClientUpdate {
                        identifier: counterparty_client_identifier,
                        header: Header {
                            height: signed_block.block.header.number,
                            block_hash: signed_block.block.header.hash(),
                            commitment_root: signed_block.block.header.state_root,
                            justification,
                            authorities_proof: StorageProof::new(
                                authorities_proof.proof.into_iter().map(|b| b.0).collect(),
                            ),
                        },
                    };
                    tx.send(datagram).unwrap();
                }
            }
        }
    }
    let connections = client
        .get_connections_using_client(block_hash, client_identifier)
        .await?;
    if connections.len() > 0 {
        info!("[{}] connections: {:?}", chain_name, connections);
    }
    for connection in connections.iter() {
        let connection_end = client.get_connection(Some(block_hash), *connection).await?;
        debug!("[{}] connection_end: {:#?}", chain_name, connection_end);
        let remote_connection_end = counterparty_client
            .get_connection(
                counterparty_block_hash,
                connection_end.counterparty_connection_identifier,
            )
            .await?;
        debug!(
            "[{}] remote_connection_end: {:#?}",
            chain_name, remote_connection_end
        );
        info!(
            "[{}] connection state: {:?}, counterparty connection state: {:?}",
            chain_name, connection_end.state, remote_connection_end.state
        );
        // TODO: remote_connection_end == None ??
        if connection_end.state == ConnectionState::Init
            && remote_connection_end.state == ConnectionState::None
        {
            let proof_consensus = client
                .consensus_state_proof(block_hash, (client_identifier, block_number))
                .await?;
            let proof_init = client.connection_proof(block_hash, *connection).await?;
            let datagram = Datagram::ConnOpenTry {
                desired_identifier: connection_end.counterparty_connection_identifier,
                counterparty_connection_identifier: *connection,
                counterparty_client_identifier: client_identifier,
                client_identifier: counterparty_client_identifier,
                version: vec![],
                counterparty_version: vec![],
                proof_init: StorageProof::new(proof_init.proof.into_iter().map(|b| b.0).collect()),
                proof_consensus: StorageProof::new(
                    proof_consensus.proof.into_iter().map(|b| b.0).collect(),
                ),
                proof_height: block_number,
                consensus_height: 0, // TODO: local consensus state height
            };
            tx.send(datagram).unwrap();
        } else if connection_end.state == ConnectionState::TryOpen
            && remote_connection_end.state == ConnectionState::Init
        {
            let proof_try = client.connection_proof(block_hash, *connection).await?;
            let datagram = Datagram::ConnOpenAck {
                identifier: connection_end.counterparty_connection_identifier,
                version: vec![],
                proof_try: StorageProof::new(proof_try.proof.into_iter().map(|b| b.0).collect()),
                proof_consensus: StorageProof::empty(),
                proof_height: block_number,
                consensus_height: 0,
            };
            tx.send(datagram).unwrap();
        } else if connection_end.state == ConnectionState::Open
            && remote_connection_end.state == ConnectionState::TryOpen
        {
            let proof_ack = client.connection_proof(block_hash, *connection).await?;
            let datagram = Datagram::ConnOpenConfirm {
                identifier: connection_end.counterparty_connection_identifier,
                proof_ack: StorageProof::new(proof_ack.proof.into_iter().map(|b| b.0).collect()),
                proof_height: block_number,
            };
            tx.send(datagram).unwrap();
        }
    }
    let channels = client
        .get_channels_using_client(block_hash, client_identifier)
        .await?;
    if channels.len() > 0 {
        info!("[{}] channels: {:?}", chain_name, channels);
    }
    for channel in channels.iter() {
        let channel_end = client
            .get_channel(Some(block_hash), channel.clone())
            .await?;
        debug!("[{}] channel_end: {:#?}", chain_name, channel_end);
        let remote_channel_end = counterparty_client
            .get_channel(
                counterparty_block_hash,
                (
                    channel_end.counterparty_port_identifier.clone(),
                    channel_end.counterparty_channel_identifier,
                ),
            )
            .await?;
        debug!(
            "[{}] remote_channel_end: {:#?}",
            chain_name, remote_channel_end
        );
        info!(
            "[{}] channle state: {:?}, counterparty channel state: {:?}",
            chain_name, channel_end.state, remote_channel_end.state
        );
        if channel_end.state == ChannelState::Init && remote_channel_end.state == ChannelState::None
        {
            let connection_end = client
                .get_connection(Some(block_hash), channel_end.connection_hops[0])
                .await?;
            let proof_init = client.channel_proof(block_hash, channel.clone()).await?;
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
                proof_init: StorageProof::new(proof_init.proof.into_iter().map(|b| b.0).collect()),
                proof_height: block_number,
            };
            tx.send(datagram).unwrap();
        } else if channel_end.state == ChannelState::TryOpen
            && remote_channel_end.state == ChannelState::Init
        {
            let proof_try = client.channel_proof(block_hash, channel.clone()).await?;
            let datagram = Datagram::ChanOpenAck {
                port_identifier: channel_end.counterparty_port_identifier,
                channel_identifier: channel_end.counterparty_channel_identifier,
                version: remote_channel_end.version,
                proof_try: StorageProof::new(proof_try.proof.into_iter().map(|b| b.0).collect()),
                proof_height: block_number,
            };
            tx.send(datagram).unwrap();
        } else if channel_end.state == ChannelState::Open
            && remote_channel_end.state == ChannelState::TryOpen
        {
            let proof_ack = client.channel_proof(block_hash, channel.clone()).await?;
            let datagram = Datagram::ChanOpenConfirm {
                port_identifier: channel_end.counterparty_port_identifier,
                channel_identifier: channel_end.counterparty_channel_identifier,
                proof_ack: StorageProof::new(proof_ack.proof.into_iter().map(|b| b.0).collect()),
                proof_height: block_number,
            };
            tx.send(datagram).unwrap();
        }
    }

    let change_sets: Vec<StorageChangeSet<H256>> = client
        .query_storage(vec![events_storage_key], block_hash, None)
        .await?;
    debug!("length of change_sets: {:?}", change_sets.len());
    debug!("change_sets: {:?}", change_sets);
    let events = change_sets
        .into_iter()
        .map(|change_set| change_set.changes)
        .flatten()
        .filter_map(|(_key, data)| data.as_ref().map(|data| Decode::decode(&mut &data.0[..])))
        .filter_map(|result: Result<EventRecords, codec::Error>| result.ok())
        .flatten()
        .collect::<Vec<system::EventRecord<node_runtime::Event, <Runtime as System>::Hash>>>();
    for event in events.into_iter() {
        match event.event {
            node_runtime::Event::ibc(pallet_ibc::RawEvent::SendPacket(
                sequence,
                data,
                timeout_height,
                source_port,
                source_channel,
                dest_port,
                dest_channel,
            )) => {
                info!("[{}] SendPacket data: {:?}", chain_name, data);
                let packet_data = Packet {
                    sequence,
                    timeout_height,
                    source_port: source_port.clone(),
                    source_channel,
                    dest_port,
                    dest_channel,
                    data,
                };
                let proof = client
                    .packet_proof(block_hash, (source_port, source_channel, timeout_height))
                    .await?;
                let datagram = Datagram::PacketRecv {
                    packet: packet_data,
                    proof: StorageProof::new(proof.proof.into_iter().map(|b| b.0).collect()),
                    proof_height: block_number,
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
                acknowledgement,
            )) => {
                debug!(
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
                info!("[{}] RecvPacket data: {:?}", chain_name, data);
                // relay packet acknowledgement with this sequence number
                let packet_data = Packet {
                    sequence,
                    timeout_height,
                    source_port: source_port.clone(),
                    source_channel,
                    dest_port,
                    dest_channel,
                    data,
                };
                let proof = client
                    .acknowledgement_proof(
                        block_hash,
                        (source_port, source_channel, timeout_height),
                    )
                    .await?;
                let datagram = Datagram::PacketAcknowledgement {
                    packet: packet_data,
                    acknowledgement,
                    proof: StorageProof::new(proof.proof.into_iter().map(|b| b.0).collect()),
                    proof_height: block_number,
                };
                tx.send(datagram).unwrap();
            }
            _ => {}
        }
    }

    Ok(())
}
