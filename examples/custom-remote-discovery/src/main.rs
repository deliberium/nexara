use async_trait::async_trait;
use nexara_core::ToolDescriptor;
use nexara_remote::{ToolDiscovery, ToolDiscoveryRequest, ToolDiscoveryResponse};

struct StaticDiscovery;

#[async_trait]
impl ToolDiscovery for StaticDiscovery {
    async fn list_tools(
        &self,
        request: ToolDiscoveryRequest,
    ) -> anyhow::Result<ToolDiscoveryResponse> {
        let mut tool = ToolDescriptor::read_only(
            "remote_search",
            format!(
                "Searches the remote endpoint at {}",
                request.endpoint.base_url
            ),
        );
        tool.tags = vec!["remote".to_string(), "search".to_string()];
        Ok(ToolDiscoveryResponse { tools: vec![tool] })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let discovery = StaticDiscovery;
    let response = discovery
        .list_tools(ToolDiscoveryRequest {
            endpoint: nexara_remote::RemoteEndpoint {
                base_url: "https://tools.example.com".to_string(),
            },
            prompt: Some("find a repository".to_string()),
        })
        .await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
