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
