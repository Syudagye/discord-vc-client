use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::Error;

#[derive(Debug, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
}

pub async fn list_guilds(client: &Client, token: &str) -> Result<Vec<Guild>, Error> {
    let res = client
        .get(super::endpoint("/users/@me/guilds"))
        .header("Authorization", token)
        .send()
        .await?;

    if res.status().is_client_error() {
        return Err(Error::Unauthorized);
    }

    let data = res.text().await?;

    Ok(serde_json::from_str(&data)?)
}

#[derive(Debug, Deserialize)]
pub struct VcChannel {
    pub id: String,
    pub name: String,
    pub user_limit: Option<usize>,

    #[serde(rename = "type")]
    channel_type: u8,
}

pub async fn list_vc_channels(
    client: &Client,
    token: &str,
    guild: &Guild,
) -> Result<Vec<VcChannel>, Error> {
    let res = client
        .get(super::endpoint(&format!("/guilds/{}/channels", guild.id)))
        .header("Authorization", token)
        .send()
        .await?;

    if res.status().is_client_error() {
        return Err(Error::Unauthorized);
    }

    let data = res.text().await?;
    let channel: Vec<VcChannel> = serde_json::from_str(&data)?;

    Ok(channel
        .into_iter()
        // Voice channels have id 2
        .filter(|c| c.channel_type == 2)
        .collect())
}
