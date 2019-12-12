use calls::{template, NodeRuntime as Runtime};
use clap::{App, ArgMatches, SubCommand};
use futures::compat::Future01CompatExt;
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
            let identifier = matches
                .value_of("identifier")
                .expect("The identifier of chain is required; qed")
                .to_string();
            tokio_compat::run_std(async {
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
<identifier> 'The identifier of demo chain'
",
            )])
        .get_matches();
    execute(matches);
}

async fn create_client(addr: Url, identifier: String) -> Result<(), Box<dyn Error>> {
    let signer = AccountKeyring::Alice.pair();
    let client = ClientBuilder::<Runtime>::new()
        .set_url(addr.clone())
        .build()
        .compat()
        .await?;
    let xt = client.xt(signer, None).compat().await?;
    xt.submit(template::test_create_client(identifier.as_bytes().to_vec()))
        .compat()
        .await?;
    Ok(())
}
