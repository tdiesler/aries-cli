use crate::agent::agents::{Agent, HttpAgentExtended};
use crate::typing::{
    Connection, Connections, CredentialDefinition, CredentialDefinitionConfig, Features,
    Invitation, InvitationConfig, IssueCredentialConfig, MessageConfig, Schema, SchemaConfig,
};
use crate::utils::http::HttpCalls;
use async_trait::async_trait;
use reqwest::Url;
use serde_json::{json, Value};

/// HTTP cloudagent
#[derive(Debug, Clone)]
pub struct HttpAgent {
    /// base url of the cloudagent
    pub url: String,

    /// admin Api key for the cloudagent
    pub api_key: Option<String>,
}

/// All the available endpoints
struct Endpoint;

/// Default value for every endpoint
impl Endpoint {
    /// base + connections
    fn connections(url: &str) -> Url {
        reqwest::Url::parse(url)
            .unwrap_or_else(|_| panic!("Could not parse {}", url))
            .join("connections")
            .unwrap_or_else(|_| panic!("Could not join on connections"))
    }
    /// base + connections + :id
    fn get_connection_by_id(url: &str, id: &str) -> Url {
        reqwest::Url::parse(url)
            .unwrap_or_else(|_| panic!("Could not parse {}", url))
            .join("connections/")
            .unwrap_or_else(|_| panic!("Could not join on connections"))
            .join(id)
            .unwrap_or_else(|_| panic!("Could not join on {}", id))
    }
    /// base + connections + create-invitation
    fn create_invitation(url: &str) -> Url {
        reqwest::Url::parse(url)
            .unwrap_or_else(|_| panic!("Could not parse {}", url))
            .join("connections/")
            .unwrap_or_else(|_| panic!("Could not join on connections"))
            .join("create-invitation")
            .unwrap_or_else(|_| panic!("Could not join on create-invitation"))
    }
    /// base + features
    fn discover_features(url: &str) -> Url {
        reqwest::Url::parse(url)
            .unwrap_or_else(|_| panic!("Could not parse {}", url))
            .join("features")
            .unwrap_or_else(|_| panic!("Could not join on features"))
    }
    /// base + connections + :id + send-message
    fn basic_message(url: &str, id: &str) -> Url {
        reqwest::Url::parse(url)
            .unwrap_or_else(|_| panic!("Could not parse {}", url))
            .join("connections/")
            .unwrap_or_else(|_| panic!("Could not join on connections"))
            .join(&format!("{}/", id))
            .unwrap_or_else(|_| panic!("Could not join on {}", id))
            .join("send-message")
            .unwrap_or_else(|_| panic!("Could not join on send-message"))
    }
    /// base + issue-credential + send-offer
    fn credential_offer(url: &str) -> Url {
        reqwest::Url::parse(url)
            .unwrap_or_else(|_| panic!("Could not parse {}", url))
            .join("issue-credential")
            .unwrap_or_else(|_| panic!("Could not join on issue-credential"))
            .join("send-offer")
            .unwrap_or_else(|_| panic!("Could not join on send-offer"))
    }
    /// base + schemas
    fn schema(url: &str) -> Url {
        reqwest::Url::parse(url)
            .unwrap_or_else(|_| panic!("Could not parse {}", url))
            .join("schemas")
            .unwrap_or_else(|_| panic!("Could not join on schemas"))
    }
    /// base + credential-definitions
    fn credential_definition(url: &str) -> Url {
        reqwest::Url::parse(url)
            .unwrap_or_else(|_| panic!("Could not parse {}", url))
            .join("credential-definitions")
            .unwrap_or_else(|_| panic!("Could not join on credential-definitions"))
    }
}

#[async_trait]
impl HttpAgentExtended for HttpAgent {
    fn new(endpoint: String, api_key: Option<String>) -> Self {
        HttpAgent { url: endpoint, api_key }
    }

    /// Check if the endpoint is valid
    async fn check_endpoint(&self) -> () {
        self.get::<Connections>(Endpoint::connections(&self.url), None).await;
    }
}

#[async_trait]
impl Agent for HttpAgent {
    /// Gets all the connections
    async fn get_connections(&self, filter: Option<String>) -> Connections {
        let mut query: Vec<(&str, String)> = vec![];

        if let Some(alias) = filter {
            query.push(("alias", alias));
        }

        self.get::<Connections>(Endpoint::connections(&self.url), Some(query)).await
    }

    /// Get a connection by id
    async fn get_connection_by_id(&self, id: String) -> Connection {
        self.get::<Connection>(Endpoint::get_connection_by_id(&self.url, &id), None).await
    }

    /// Prints an invitation, as url or qr, in stdout
    async fn create_invitation(&self, config: &InvitationConfig) -> Invitation {
        let mut query: Vec<(&str, String)> = vec![];
        let mut body = None;

        if config.toolbox {
            query.push(("multi_use", false.to_string()));
            query.push(("auto_accept", true.to_string()));
            query.push(("alias", String::from("toolbox")));

            body = Some(json!({
                "metadata": {
                    "group": "admin"
                }
            }));
        } else {
            if config.multi_use {
                query.push(("multi_use", true.to_string()));
            }
            if config.auto_accept {
                query.push(("auto_accept", true.to_string()))
            }
            if let Some(alias) = &config.alias {
                query.push(("alias", alias.to_string()));
            }
        }

        self.post(Endpoint::create_invitation(&self.url), Some(query), body).await
    }

    /// Requests all the features from the cloudagent
    async fn discover_features(&self) -> Features {
        self.get::<Features>(Endpoint::discover_features(&self.url), None).await
    }

    /// Send a basic message to another agent
    async fn send_message(&self, config: &MessageConfig) -> () {
        let body = json!({
            "content": config.message,
        });

        self.post::<serde_json::Value>(
            Endpoint::basic_message(&self.url, &config.connection_id),
            None,
            Some(body),
        )
        .await;
    }

    async fn credential(&self, config: &IssueCredentialConfig) {
        let body = json!({
          "connection_id": config.connection_id,
          "cred_def_id": config.credential_definition_id,
          "credential_preview": {
            "@type": "issue-credential/1.0/credential-preview",
            "attributes": config.attributes
          }
        });

        self.post::<Value>(Endpoint::credential_offer(&self.url), None, Some(body)).await;
    }

    async fn schema(&self, config: &SchemaConfig) -> Schema {
        let body = json!({
          "attributes": config.attributes,
          "schema_name": config.name,
          "schema_version": config.version
        });

       self.post::<Schema>(Endpoint::schema(&self.url), None, Some(body)).await
    }

    async fn credential_definition(
        &self,
        config: &CredentialDefinitionConfig,
    ) -> CredentialDefinition {
        let body = json!({
          "schema_id": config.schema_id,
          "support_revocation": false,
          "tag": config.tag
        });

        self.post::<CredentialDefinition>(
            Endpoint::credential_definition(&self.url),
            None,
            Some(body),
        )
        .await
    }
}
