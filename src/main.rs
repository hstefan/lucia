use clap::Parser;
use config::Config;
use futures_util::{pin_mut, stream::StreamExt};
use mdns::{Record, RecordKind};

use std::io::Write;
use std::net::Ipv4Addr;
use std::str;
use std::{net::IpAddr, time::Duration};
use tokio::time::sleep;

mod common;
mod config;
mod philips_hue;

use common::Result;
use philips_hue::NewUserResult;

#[derive(Parser)]
#[clap(about, version, author)]
enum Lucia {
    Discover(Discover),
    Configure(Configure),
    Devices(Devices),
    Light(Light),
    Groups(Groups),
}

#[derive(clap::Args)]
/// Print all discovered Hue Bridge devices
struct Discover {
    #[clap(short, long, default_value_t = 5)]
    timeout_secs: u64,
}

#[derive(clap::Args)]
/// Connect to a Hue Bridge address and attempts to pair with it
struct Configure {
    #[clap(short, long, default_value_t = 300)]
    max_poll_secs: u64,

    #[clap(short, long, default_value_t = 3)]
    poll_interval_secs: u64,

    #[clap(short, long)]
    address: String,
}

#[derive(clap::Args)]
/// Print all lights known by the pre-configured bridges
struct Devices {}

#[derive(clap::Args)]
/// Set properties of a list of light sources identified by their id.
///
/// If any of the supported arguments is not passed in that property will not be modified.
struct Light {
    #[clap(last = true)]
    ids: Vec<String>,

    #[clap(short, long)]
    // Brightness percentage
    brightness: Option<f32>,

    #[clap(short, long)]
    // Color temperature value in kelvin (range is device dependant)
    temperature: Option<u16>,

    #[clap(short, long)]
    // Boolean value for powering on or off.
    power: Option<bool>,

    #[clap(short, long)]
    // The IDs of every group to update.
    group_ids: Vec<String>,
}

#[derive(clap::Args)]
/// Print all groups known by the pre-configured bridges
struct Groups {}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}

fn flush_stdout() {
    if let Err(e) = std::io::stdout().flush() {
        eprintln!("failed to flush stdout: {:?}", e);
    }
}

async fn discover(cmd: &Discover) -> Result<()> {
    let stream =
        mdns::discover::all("_hue._tcp.local", Duration::from_secs(cmd.timeout_secs))?.listen();
    pin_mut!(stream);

    if let Some(Ok(response)) = stream.next().await {
        let addr = response.records().filter_map(self::to_ip_addr).next();
        if let Some(addr) = addr {
            println!("found bridge at {}", addr);
        }
    }
    Ok(())
}

async fn api_client(config: &Config) -> Result<philips_hue::ApiClient> {
    let addr = config
        .bridge_ip
        .as_ref()
        .expect("missing bridge_ip in config");
    let addr: Ipv4Addr = addr.parse()?;
    let client = philips_hue::ApiClient::new(std::net::IpAddr::V4(addr))?;
    Ok(client)
}

async fn devices(_cmd: &Devices) -> Result<()> {
    let config = Config::load()?;
    let username = config
        .user_name
        .as_ref()
        .expect("missing api_key in config");
    let client = api_client(&config).await?;
    for (id, light) in client.get_lights(username).await? {
        println!(
            "{}: {} (type={}, on={}, bri={})",
            id, light.name, light.type_, light.state.on, light.state.bri
        );
    }
    Ok(())
}

async fn list_groups(_cmd: &Groups) -> Result<()> {
    let config = Config::load()?;
    let username = config
        .user_name
        .as_ref()
        .expect("missing api_key in config");
    let client = api_client(&config).await?;
    for (id, group) in client.get_groups(username).await? {
        println!(
            "{}: {} (type={}, on={}, bri={}, lights={:?})",
            id, group.name, group.type_, group.action.on, group.action.bri, group.lights,
        );
    }
    Ok(())
}

async fn create_new_user(
    client: &philips_hue::ApiClient,
    poll_interval: Duration,
    poll_max: Duration,
) -> Result<(String, Option<String>)> {
    print!("waiting for the link button to be pushed..");
    flush_stdout();
    let start = std::time::Instant::now();
    loop {
        print!(".");
        flush_stdout();
        match client.post_new_user("lucia#windy").await {
            Ok(NewUserResult::Error(_)) => (),
            Ok(NewUserResult::Success(s)) => return Ok((s.username, s.client_key)),
            Err(e) => return Err(e),
        }
        if std::time::Instant::now() - start > poll_max {
            break;
        }
        sleep(poll_interval).await;
    }
    Err(common::LuciaError::PollingTimeout)
}

async fn configure(cmd: &Configure) -> Result<()> {
    let mut cfg = config::Config::load()?;
    cfg.bridge_ip = Some(cmd.address.to_owned());
    let client = api_client(&cfg).await?;
    let user = create_new_user(
        &client,
        Duration::from_secs(cmd.poll_interval_secs),
        Duration::from_secs(cmd.max_poll_secs),
    )
    .await?;
    cfg.user_name = Some(user.0);
    cfg.client_key = user.1;
    cfg.persist().expect("unable to save user configuration");
    Ok(())
}

async fn set_lights(cmd: &Light) -> Result<()> {
    let config = Config::load()?;
    let username = config
        .user_name
        .as_ref()
        .expect("missing api_key in config");
    let brightness: Option<u8> = cmd.brightness.map(|x| ((x / 100.0) * 255.0) as u8);
    let ct = cmd.temperature.map(|x| (1_000_000 / (x as u32)) as u16); // calculated micro reciprocal degree (mired);
    let client = api_client(&config).await?;
    for id in &cmd.ids {
        client
            .set_light_state(username, id, brightness, ct, cmd.power)
            .await?
    }
    for gid in &cmd.group_ids {
        client
            .set_group_state(username, gid, brightness, ct, cmd.power)
            .await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Lucia::parse();
    match args {
        Lucia::Discover(d) => discover(&d).await.expect("failed to discover"),
        Lucia::Configure(l) => configure(&l).await.expect("link failed"),
        Lucia::Devices(d) => devices(&d).await.expect("unable to list devices"),
        Lucia::Groups(g) => list_groups(&g).await.expect("unable to list groups"),
        Lucia::Light(b) => set_lights(&b)
            .await
            .expect("unable to change device brightness"),
    }
    Ok(())
}
