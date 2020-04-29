use calls::{template, NodeRuntime as Runtime};
use clap::{App, Arg, ArgMatches, SubCommand};
use lazy_static::lazy_static;
// use rand::RngCore;
use sp_core::{storage::StorageKey, Blake2Hasher, Hasher, H256};
use sp_finality_grandpa::{AuthorityList, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY};
use sp_keyring::AccountKeyring;
use std::collections::HashMap;
use std::error::Error;
use substrate_subxt::{BlockNumber, ClientBuilder};

lazy_static! {
    static ref ENDPOINTS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("appia", "ws://127.0.0.1:9944");
        m.insert("flaminia", "ws://127.0.0.1:8844");
        m
    };
}

fn execute(matches: ArgMatches) {
    let chain = matches.value_of("CHAIN").unwrap();
    let addr = ENDPOINTS.get(chain).unwrap();
    match matches.subcommand() {
        ("create-client", Some(matches)) => {
            let chain_name = matches
                .value_of("chain-name")
                .expect("The name of chain is required; qed");
            let identifier = Blake2Hasher::hash(chain.as_bytes());
            println!("identifier: {:?}", identifier);

            let counterparty_addr = ENDPOINTS.get(chain_name).unwrap();
            let result =
                async_std::task::block_on(create_client(&addr, &counterparty_addr, identifier));
            println!("create_client: {:?}", result);
        }
        ("conn-open-init", Some(matches)) => {
            if chain != "appia" {
                println!("CHAIN can only be appia in this demo");
                return;
            }
            let client_identifier = matches
                .value_of("client-identifier")
                .expect("The identifier of chain is required; qed");
            let client_identifier = hex::decode(client_identifier).unwrap();
            let client_identifier = H256::from_slice(&client_identifier);

            let counterparty_client_identifier = matches
                .value_of("counterparty-client-identifier")
                .expect("The identifier of counterparty chain is required; qed");
            let counterparty_client_identifier =
                hex::decode(counterparty_client_identifier).unwrap();
            let counterparty_client_identifier = H256::from_slice(&counterparty_client_identifier);

            // let mut data = [0u8; 32];
            // rand::thread_rng().fill_bytes(&mut data);
            // let identifier = H256::from_slice(&data);
            // rand::thread_rng().fill_bytes(&mut data);
            // let desired_counterparty_connection_identifier = H256::from_slice(&data);

            let identifier = Blake2Hasher::hash(b"appia-connection");
            println!("identifier: {:?}", identifier);
            let desired_counterparty_connection_identifier =
                Blake2Hasher::hash(b"flaminia-connection");
            println!(
                "desired_counterparty_connection_identifier: {:?}",
                desired_counterparty_connection_identifier
            );

            let result = async_std::task::block_on(conn_open_init(
                &addr,
                identifier,
                desired_counterparty_connection_identifier,
                client_identifier,
                counterparty_client_identifier,
            ));
            println!("conn_open_init: {:?}", result);
        }
        ("bind-port", Some(matches)) => {
            let identifier = matches
                .value_of("identifier")
                .expect("The identifier of port is required; qed");
            let identifier = identifier.as_bytes().to_vec();
            println!("identifier: {:?}", identifier);

            let result = async_std::task::block_on(bind_port(&addr, identifier));
            println!("bind_port: {:?}", result);
        }
        ("release-port", Some(matches)) => {
            let identifier = matches
                .value_of("identifier")
                .expect("The identifier of port is required; qed");
            let identifier = identifier.as_bytes().to_vec();
            println!("identifier: {:?}", identifier);

            let result = async_std::task::block_on(release_port(&addr, identifier));
            println!("release_port: {:?}", result);
        }
        ("chan-open-init", Some(matches)) => {
            if chain != "appia" {
                println!("CHAIN can only be appia in this demo");
                return;
            }
            let unordered = matches.is_present("unordered");
            let connection_identifier = matches
                .value_of("connection-identifier")
                .expect("The identifier of connection is required; qed");
            let connection_identifier = hex::decode(connection_identifier).unwrap();
            let connection_identifier = H256::from_slice(&connection_identifier);
            let connection_hops = vec![connection_identifier];
            let port_identifier = matches
                .value_of("port-identifier")
                .expect("The identifier of port is required; qed");
            let port_identifier = port_identifier.as_bytes().to_vec();
            let counterparty_port_identifier = matches
                .value_of("counterparty-port-identifier")
                .expect("The identifier of counterparty port is required; qed");
            let counterparty_port_identifier = counterparty_port_identifier.as_bytes().to_vec();

            // let mut data = [0u8; 32];
            // rand::thread_rng().fill_bytes(&mut data);
            // let channel_identifier = H256::from_slice(&data);
            // rand::thread_rng().fill_bytes(&mut data);
            // let desired_counterparty_channel_identifier = H256::from_slice(&data);

            let channel_identifier = Blake2Hasher::hash(b"appia-channel");
            println!("channel_identifier: {:?}", channel_identifier);
            let desired_counterparty_channel_identifier = Blake2Hasher::hash(b"flaminia-channel");
            println!(
                "desired_counterparty_channel_identifier: {:?}",
                desired_counterparty_channel_identifier
            );

            let result = async_std::task::block_on(chan_open_init(
                &addr,
                unordered,
                connection_hops,
                port_identifier,
                channel_identifier,
                counterparty_port_identifier,
                desired_counterparty_channel_identifier,
            ));
            println!("chan_open_init: {:?}", result);
        }
        ("send-packet", Some(matches)) => {
            if chain != "appia" {
                println!("CHAIN can only be appia in this demo");
                return;
            }
            let sequence = matches
                .value_of("sequence")
                .expect("The sequence of packet is required; qed");
            let sequence: u64 = sequence.parse().unwrap();
            let timeout_height = matches
                .value_of("timeout-height")
                .expect("The timeout-height of packet is required; qed");
            let timeout_height: u32 = timeout_height.parse().unwrap();
            let source_port = matches
                .value_of("source-port")
                .expect("The source-port of packet is required; qed");
            let source_port = source_port.as_bytes().to_vec();
            let source_channel = matches
                .value_of("source-channel")
                .expect("The source-channel of packet is required; qed");
            let source_channel = hex::decode(source_channel).unwrap();
            let source_channel = H256::from_slice(&source_channel);
            let dest_port = matches
                .value_of("dest-port")
                .expect("The dest-port of packet is required; qed");
            let dest_port = dest_port.as_bytes().to_vec();
            let dest_channel = matches
                .value_of("dest-channel")
                .expect("The dest-channel of packet is required; qed");
            let dest_channel = hex::decode(dest_channel).unwrap();
            let dest_channel = H256::from_slice(&dest_channel);
            let data = matches
                .value_of("data")
                .expect("The data of packet is required; qed");
            let data: Vec<u8> = hex::decode(data).expect("Invalid message");

            let result = async_std::task::block_on(send_packet(
                &addr,
                sequence,
                timeout_height,
                source_port,
                source_channel,
                dest_port,
                dest_channel,
                data,
            ));
            println!("send_packet: {:?}", result);
        }
        _ => print_usage(&matches),
    }
}

fn print_usage(matches: &ArgMatches) {
    println!("{}", matches.usage());
}

fn main() {
    let matches = App::new("cli")
        .author("Cdot Network <ys@cdot.network>")
        .about("cli is a tool for testing IBC protocol")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("CHAIN")
             .help("Sets the chain to be operated")
             .required(true))
        .subcommands(vec![SubCommand::with_name("create-client")
            .about("Create a new client")
            .args_from_usage(
                "
<chain-name> 'The name of counterparty demo chain'
",
            )])
        .subcommands(vec![SubCommand::with_name("conn-open-init")
            .about("Open a new connection")
            .args_from_usage(
                "
<client-identifier> 'The client identifier of demo chain'
<counterparty-client-identifier> 'The client identifier of counterparty demo chain'
",
            )])
        .subcommands(vec![SubCommand::with_name("bind-port")
            .about("Bind module to an unallocated port")
            .args_from_usage(
                "
<identifier> 'The identifier of port'
",
            )])
        .subcommands(vec![SubCommand::with_name("release-port")
            .about("Release a port")
            .args_from_usage(
                "
<identifier> 'The identifier of port'
",
            )])
        .subcommands(vec![SubCommand::with_name("chan-open-init")
            .about("Open a new channel")
            .args_from_usage(
                "
--unordered 'Channel is unordered'
<connection-identifier> 'The connection identifier of demo chain'
<port-identifier> 'The identifier of port'
<counterparty-port-identifier> 'The identifier of port on counterparty chain'
",
            )])
        .subcommands(vec![SubCommand::with_name("send-packet")
            .about("Send an IBC packet")
            .args_from_usage(
                "
<sequence> 'The sequence number corresponds to the order of sends and receives'
<timeout-height> 'The timeoutHeight indicates a consensus height on the destination chain after which the packet will no longer be processed, and will instead count as having timed-out'
<source-port> 'The sourcePort identifies the port on the sending chain'
<source-channel> 'The sourceChannel identifies the channel end on the sending chain'
<dest-port> 'The destPort identifies the port on the receiving chain'
<dest-channel> 'The destChannel identifies the channel end on the receiving chain'
<data> 'The data is an opaque value which can be defined by the application logic of the associated modules'
",
            )])
        .get_matches();
    execute(matches);
}

async fn create_client(
    addr: &str,
    counterparty_addr: &str,
    identifier: H256,
) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();

    let counterparty_client = ClientBuilder::<Runtime>::new()
        .set_url(counterparty_addr)
        .build()
        .await?;

    let genesis_hash = counterparty_client
        .block_hash(Some(BlockNumber::from(0u32)))
        .await?;
    println!("counterparty genesis_hash: {:?}", genesis_hash);
    let genesis_header = counterparty_client.header(genesis_hash).await?.unwrap();
    println!("counterparty genesis_header: {:?}", genesis_header);
    let storage_key = StorageKey(GRANDPA_AUTHORITIES_KEY.to_vec());
    let genesis_authorities: AuthorityList = counterparty_client
        .rpc
        .storage::<VersionedAuthorityList>(storage_key, genesis_hash)
        .await?
        .map(|versioned: VersionedAuthorityList| versioned.into())
        .unwrap();
    println!(
        "counterparty genesis_authorities: {:?}",
        genesis_authorities
    );
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr)
        .build()
        .await?;
    let xt = client.xt(signer, None).await?;
    xt.submit(template::TestCreateClientCall {
        identifier,
        height: 0,
        set_id: 0,
        authority_list: genesis_authorities,
        commitment_root: genesis_header.state_root,
    })
    .await?;
    Ok(())
}

async fn conn_open_init(
    addr: &str,
    identifier: H256,
    desired_counterparty_connection_identifier: H256,
    client_identifier: H256,
    counterparty_client_identifier: H256,
) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .await?;
    let xt = client.xt(signer, None).await?;
    xt.submit(template::TestConnOpenInitCall {
        identifier,
        desired_counterparty_connection_identifier,
        client_identifier,
        counterparty_client_identifier,
    })
    .await?;
    Ok(())
}

async fn bind_port(addr: &str, identifier: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .await?;
    let xt = client.xt(signer, None).await?;
    xt.submit(template::TestBindPortCall { identifier }).await?;
    Ok(())
}

async fn release_port(addr: &str, identifier: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .await?;
    let xt = client.xt(signer, None).await?;
    xt.submit(template::TestReleasePortCall { identifier })
        .await?;
    Ok(())
}

async fn chan_open_init(
    addr: &str,
    unordered: bool,
    connection_hops: Vec<H256>,
    port_identifier: Vec<u8>,
    channel_identifier: H256,
    counterparty_port_identifier: Vec<u8>,
    counterparty_channel_identifier: H256,
) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .await?;
    let xt = client.xt(signer, None).await?;
    xt.submit(template::TestChanOpenInitCall {
        unordered,
        connection_hops,
        port_identifier,
        channel_identifier,
        counterparty_port_identifier,
        counterparty_channel_identifier,
    })
    .await?;
    Ok(())
}

async fn send_packet(
    addr: &str,
    sequence: u64,
    timeout_height: u32,
    source_port: Vec<u8>,
    source_channel: H256,
    dest_port: Vec<u8>,
    dest_channel: H256,
    data: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .await?;
    let xt = client.xt(signer, None).await?;
    xt.submit(template::TestSendPacketCall {
        sequence,
        timeout_height,
        source_port,
        source_channel,
        dest_port,
        dest_channel,
        data,
    })
    .await?;
    Ok(())
}
