use reqwest::{self};

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::net::IpAddr;

use crate::common::Result;

pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

// {"error":{"type":101,"address":"","description":"link button not pressed"}}

#[derive(Deserialize, Debug)]
pub enum NewUserResult {
    #[serde(rename = "error")]
    Error(ApiError),
    #[serde(rename = "success")]
    Success(NewUserResponse),
}

#[derive(Deserialize, Debug)]
pub struct ApiError {
    #[serde(rename = "type")]
    pub type_: i32,
    pub address: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct NewUserResponse {
    pub username: String,
    #[serde(rename = "clientkey")]
    pub client_key: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
struct NewUserPayload {
    #[serde(rename = "devicetype")]
    pub device_type: String,
    #[serde(rename = "generateclientkey")]
    pub generate_client_key: bool,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct LightState {
    pub on: bool,
    pub bri: u8,
    pub hue: Option<i32>,
    pub sat: Option<i32>,
    pub effect: Option<String>,
    pub xy: Option<Vec<f32>>,
    pub ct: i32,
    pub alert: String,
    #[serde(rename = "colormode")]
    pub color_mode: String,
    pub reachable: bool,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct LightControls {
    #[serde(rename = "mindimlevel")]
    pub min_dim_level: i32,
    #[serde(rename = "maxlumen")]
    pub max_lumen: i32,
    #[serde(rename = "colorgamuttype")]
    pub color_gamut_type: Option<String>,
    #[serde(rename = "colorgamut")]
    pub color_gamut: Option<Vec<Vec<f32>>>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct LightCapabilities {
    pub certified: bool,
    pub control: LightControls,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct LightConfig {
    pub archetype: String,
    pub function: String,
    pub direction: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct LightResponseItem {
    pub state: LightState,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub capabilities: LightCapabilities,
    pub config: LightConfig,
}

pub type LightsResponse = BTreeMap<String, LightResponseItem>;

#[derive(Serialize, Debug)]
struct SetLightStateBody {
    #[serde(rename = "bri", skip_serializing_if = "Option::is_none")]
    brightness: Option<u8>,
    #[serde(rename = "ct", skip_serializing_if = "Option::is_none")]
    color_mired: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    on: Option<bool>,
}

impl ApiClient {
    pub fn new(address: IpAddr) -> Result<ApiClient> {
        let client = reqwest::Client::builder().build()?;
        Ok(ApiClient {
            base_url: format!("http://{}", address),
            client,
        })
    }

    pub async fn post_new_user(&self, user: &str) -> Result<NewUserResult> {
        let payload = NewUserPayload {
            device_type: user.to_owned(),
            generate_client_key: true,
        };
        let url = format!("{}/api", self.base_url);
        let body = self.client.post(&url).json(&payload).send().await?;
        let results: Vec<NewUserResult> = body.json().await?;
        assert!(results.len() == 1);
        Ok(results.into_iter().next().unwrap())
    }

    pub async fn get_lights(&self, api_key: &str) -> Result<LightsResponse> {
        let url = format!("{}/api/{}/lights", self.base_url, api_key);
        let body = self.client.get(&url).send().await?;
        let lights: LightsResponse = body.json().await?;
        Ok(lights)
    }

    pub async fn set_light_state(
        &self,
        api_key: &str,
        id: &str,
        brightness: Option<u8>,
        temperature: Option<u16>,
        on: Option<bool>,
    ) -> Result<()> {
        let url = format!("{}/api/{}/lights/{}/state", self.base_url, api_key, id);
        let payload = SetLightStateBody {brightness, color_mired: temperature, on};
        let body = self.client.put(&url).json(&payload).send().await?;
        println!("{}", body.text().await?);
        Ok(())
    }
}
