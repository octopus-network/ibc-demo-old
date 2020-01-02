use calls::{template, NodeRuntime as Runtime};
use clap::{App, ArgMatches, SubCommand};
use futures::compat::Future01CompatExt;
use rand::RngCore;
use sp_core::{Blake2Hasher, Hasher, H256};
use sp_keyring::AccountKeyring;
use std::error::Error;
use substrate_subxt::ClientBuilder;
use url::Url;

fn execute(matches: ArgMatches) {
    match matches.subcommand() {
        ("create-client", Some(matches)) => {
            let addr = matches
                .value_of("addr")
                .expect("The address of chain is required; qed");
            let addr = Url::parse(&format!("ws://{}", addr)).expect("Is valid url; qed");
            let chain_name = matches
                .value_of("chain-name")
                .expect("The name of chain is required; qed");
            let identifier = Blake2Hasher::hash(chain_name.as_bytes());
            println!("identifier: {:?}", identifier);

            tokio_compat::run_std(async move {
                create_client(addr, identifier)
                    .await
                    .expect("Failed to create client");
            });
        }
        ("conn-open-init", Some(matches)) => {
            let addr = matches
                .value_of("addr")
                .expect("The address of chain is required; qed");
            let addr = Url::parse(&format!("ws://{}", addr)).expect("Is valid url; qed");
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

            let mut data = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut data);
            let identifier = H256::from_slice(&data);
            rand::thread_rng().fill_bytes(&mut data);
            let desired_counterparty_connection_identifier = H256::from_slice(&data);

            tokio_compat::run_std(async move {
                conn_open_init(
                    addr,
                    identifier,
                    desired_counterparty_connection_identifier,
                    client_identifier,
                    counterparty_client_identifier,
                )
                .await
                .expect("Failed to open connection");
            });
        }
        ("bind-port", Some(matches)) => {
            let addr = matches
                .value_of("addr")
                .expect("The address of chain is required; qed");
            let addr = Url::parse(&format!("ws://{}", addr)).expect("Is valid url; qed");
            let identifier = matches
                .value_of("identifier")
                .expect("The identifier of port is required; qed");
            let identifier = identifier.as_bytes().to_vec();
            println!("identifier: {:?}", identifier);

            tokio_compat::run_std(async move {
                bind_port(addr, identifier)
                    .await
                    .expect("Failed to bind port");
            });
        }
        ("release-port", Some(matches)) => {
            let addr = matches
                .value_of("addr")
                .expect("The address of chain is required; qed");
            let addr = Url::parse(&format!("ws://{}", addr)).expect("Is valid url; qed");
            let identifier = matches
                .value_of("identifier")
                .expect("The identifier of port is required; qed");
            let identifier = identifier.as_bytes().to_vec();
            println!("identifier: {:?}", identifier);

            tokio_compat::run_std(async move {
                release_port(addr, identifier)
                    .await
                    .expect("Failed to release port");
            });
        }
        ("chan-open-init", Some(matches)) => {
            let addr = matches
                .value_of("addr")
                .expect("The address of chain is required; qed");
            let addr = Url::parse(&format!("ws://{}", addr)).expect("Is valid url; qed");
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

            let mut data = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut data);
            let channel_identifier = H256::from_slice(&data);
            rand::thread_rng().fill_bytes(&mut data);
            let desired_counterparty_channel_identifier = H256::from_slice(&data);

            tokio_compat::run_std(async move {
                chan_open_init(
                    addr,
                    unordered,
                    connection_hops,
                    port_identifier,
                    channel_identifier,
                    counterparty_port_identifier,
                    desired_counterparty_channel_identifier,
                )
                .await
                .expect("Failed to open channel");
            });
        }
        ("send-packet", Some(matches)) => {
            let addr = matches
                .value_of("addr")
                .expect("The address of chain is required; qed");
            let addr = Url::parse(&format!("ws://{}", addr)).expect("Is valid url; qed");
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

            tokio_compat::run_std(async move {
                send_packet(
                    addr,
                    sequence,
                    timeout_height,
                    source_port,
                    source_channel,
                    dest_port,
                    dest_channel,
                    data,
                )
                .await
                .expect("Failed to send packet");
            });
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
        .subcommands(vec![SubCommand::with_name("create-client")
            .about("Create a new client")
            .args_from_usage(
                "
<addr> 'The address of demo chain'
<chain-name> 'The name of counterparty demo chain'
",
            )])
        .subcommands(vec![SubCommand::with_name("conn-open-init")
            .about("Open a new connection")
            .args_from_usage(
                "
<addr> 'The address of demo chain'
<client-identifier> 'The client identifier of demo chain'
<counterparty-client-identifier> 'The client identifier of counterparty demo chain'
",
            )])
        .subcommands(vec![SubCommand::with_name("bind-port")
            .about("Bind module to an unallocated port")
            .args_from_usage(
                "
<addr> 'The address of demo chain'
<identifier> 'The identifier of port'
",
            )])
        .subcommands(vec![SubCommand::with_name("release-port")
            .about("Release a port")
            .args_from_usage(
                "
<addr> 'The address of demo chain'
<identifier> 'The identifier of port'
",
            )])
        .subcommands(vec![SubCommand::with_name("chan-open-init")
            .about("Open a new channel")
            .args_from_usage(
                "
--unordered 'Channel is unordered'
<addr> 'The address of demo chain'
<connection-identifier> 'The connection identifier of demo chain'
<port-identifier> 'The identifier of port'
<counterparty-port-identifier> 'The identifier of port on counterparty chain'
",
            )])
        .subcommands(vec![SubCommand::with_name("send-packet")
            .about("Send an IBC packet")
            .args_from_usage(
                "
<addr> 'The address of demo chain'
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

async fn create_client(addr: Url, identifier: H256) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(template::test_create_client(identifier))
        .compat()
        .await?;
    Ok(())
}

async fn conn_open_init(
    addr: Url,
    identifier: H256,
    desired_counterparty_connection_identifier: H256,
    client_identifier: H256,
    counterparty_client_identifier: H256,
) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(template::test_conn_open_init(
        identifier,
        desired_counterparty_connection_identifier,
        client_identifier,
        counterparty_client_identifier,
    ))
    .compat()
    .await?;
    Ok(())
}

async fn bind_port(addr: Url, identifier: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(template::test_bind_port(identifier))
        .compat()
        .await?;
    Ok(())
}

async fn release_port(addr: Url, identifier: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Bob.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(template::test_release_port(identifier))
        .compat()
        .await?;
    Ok(())
}

async fn chan_open_init(
    addr: Url,
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
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(template::test_chan_open_init(
        unordered,
        connection_hops,
        port_identifier,
        channel_identifier,
        counterparty_port_identifier,
        counterparty_channel_identifier,
    ))
    .compat()
    .await?;
    Ok(())
}

async fn send_packet(
    addr: Url,
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
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(template::test_send_packet(
        sequence,
        timeout_height,
        source_port,
        source_channel,
        dest_port,
        dest_channel,
        data,
    ))
    .compat()
    .await?;
    Ok(())
}
