use calls::{template, NodeRuntime as Runtime};
use clap::{App, ArgMatches, SubCommand};
use futures::compat::Future01CompatExt;
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
<chain-name> 'The name of demo chain'
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
