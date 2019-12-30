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
            let ordered = matches.is_present("ordered");
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
                    ordered,
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
--ordered 'The ordering of channel'
<addr> 'The address of demo chain'
<connection-identifier> 'The connection identifier of demo chain'
<port-identifier> 'The identifier of port'
<counterparty-port-identifier> 'The identifier of port on counterparty chain'
",
            )])
        .get_matches();
    execute(matches);
}

async fn create_client(addr: Url, identifier: H256) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Alice.pair();
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
    let signer = AccountKeyring::Alice.pair();
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
    let signer = AccountKeyring::Alice.pair();
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
    let signer = AccountKeyring::Alice.pair();
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
    ordered: bool,
    connection_hops: Vec<H256>,
    port_identifier: Vec<u8>,
    channel_identifier: H256,
    counterparty_port_identifier: Vec<u8>,
    counterparty_channel_identifier: H256,
) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Alice.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(template::test_chan_open_init(
        ordered,
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
