use agent::modules::credential::{CredentialModule, CredentialOfferOptions};
use clap::{Args, Subcommand};
use log::{debug, info};

use crate::error::{Error, Result};
use crate::help_strings::HelpStrings;
use crate::utils::{
    loader::{Loader, LoaderVariant},
    logger::pretty_stringify_obj,
};
use colored::*;

#[derive(Args)]
pub struct CredentialOptions {
    #[clap(subcommand)]
    pub commands: CredentialSubcommands,
}

#[derive(Subcommand, Debug)]
#[clap(about = HelpStrings::Credentials)]
pub enum CredentialSubcommands {
    #[clap(about = HelpStrings::CredentialsOffer)]
    Offer {
        #[clap(long, short = 'i', help  = HelpStrings::CredentialsOfferConnectionId)]
        connection_id: String,
        #[clap(long, short, help = HelpStrings::CredentialsOfferCredentialDefinitionId)]
        cred_def_id: String,
        #[clap(long, short, help = HelpStrings::CredentialsOfferKey)]
        key: Vec<String>,
        #[clap(long, short, help = HelpStrings::CredentialsOfferValue)]
        value: Vec<String>,
    },
}

pub async fn parse_credentials_args(
    commands: &CredentialSubcommands,
    agent: impl CredentialModule,
) -> Result<()> {
    let loader = Loader::start(LoaderVariant::default());
    match commands {
        CredentialSubcommands::Offer {
            connection_id,
            cred_def_id,
            key,
            value,
        } => {
            if key.len() != value.len() {
                return Err(Error::UnequalAmountKeyValue.into());
            }

            let options = CredentialOfferOptions {
                connection_id: connection_id.to_string(),
                cred_def_id: cred_def_id.to_string(),
                keys: key.iter().map(|k| k.to_string()).collect(),
                values: value.iter().map(|v| v.to_string()).collect(),
            };
            agent.send_offer(options).await.map(|cred| {
                loader.stop();
                debug!("{}", pretty_stringify_obj(&cred));
                info!(
                    "{} offered a credential. Credential exchange id: ",
                    "Sucessefully".green()
                );
                println!("{}", cred.credential_exchange_id)
            })
        }
    }
}
