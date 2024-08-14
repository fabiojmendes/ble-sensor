use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use bluer::{AdapterEvent, Session};
use byteorder::{ByteOrder, LittleEndian};
use clap::Parser;
use futures::StreamExt;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::task;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Config {
    #[clap(
        short,
        long,
        default_value = "tsprod",
        help = "MQTT id for persistent connection"
    )]
    id: String,
    #[clap(short, long, default_value = "localhost", help = "MQTT server host")]
    host: String,
    #[clap(short, long, default_value = "1883", help = "MQTT server port")]
    port: u16,
    #[clap(short, long, help = "MQTT topic")]
    topic: String,
    #[clap(long, env = "MQTT_USERNAME")]
    username: String,
    #[clap(long, env = "MQTT_PASSWORD")]
    password: String,
}

struct TempReading {
    version: u8,
    counter: u8,
    voltage: u16,
    temp: i16,
}

impl TempReading {
    fn temperature(&self) -> Result<f32, String> {
        if self.temp == i16::MAX {
            return Err(format!("Invalid temperature {}", self.temp));
        }
        Ok(f32::from(self.temp) / 100.0)
    }
}

impl TryFrom<&[u8]> for TempReading {
    type Error = &'static str;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 6 {
            return Err("Buffer is too small");
        }
        let version = data[0];
        let counter = data[1];
        let voltage = LittleEndian::read_u16(&data[2..4]);
        let temp = LittleEndian::read_i16(&data[4..]);

        Ok(TempReading {
            version,
            counter,
            voltage,
            temp,
        })
    }
}

fn timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    env_logger::builder().format_timestamp(None).init();
    let config = Config::parse();

    let mut opts = MqttOptions::new(config.id, config.host, config.port);
    opts.set_credentials(config.username, config.password);

    let (client, mut eventloop) = AsyncClient::new(opts, 10);

    task::spawn(async move {
        let session = Session::new().await?;
        let adapter = session.default_adapter().await?;
        log::info!(
            "Discovering devices using Bluetooth adapater {}",
            adapter.name()
        );
        adapter.set_powered(true).await?;
        adapter
            .set_discovery_filter(bluer::DiscoveryFilter {
                duplicate_data: false,
                pattern: Some(String::from("Tempsys")),
                ..Default::default()
            })
            .await?;

        let mut last_counts = HashMap::new();

        let mut device_events = adapter.discover_devices_with_changes().await?;
        while let Some(evt) = device_events.next().await {
            if let AdapterEvent::DeviceAdded(addr) = evt {
                let device = adapter.device(addr)?;
                log::trace!("Device: {:?} name: {:?}", device, device.name().await?);
                let rssi = device.rssi().await?.unwrap_or(0);
                match device
                    .manufacturer_data()
                    .await?
                    .and_then(|mut d| d.remove(&0xffff))
                    .map(|data| TempReading::try_from(&*data))
                {
                    Some(Ok(reading)) => {
                        if Some(&reading.counter) == last_counts.get(&addr) {
                            continue;
                        }
                        last_counts.insert(addr, reading.counter);
                        let timestamp = timestamp_nanos();
                        let payload = match reading.temperature() {
                            Ok(temp) => {
                                format!("sensor,sender={},version={} temperature={:.2},voltage={},rssi={} {}",
                                            addr, reading.version, temp, reading.voltage, rssi, timestamp)
                            }
                            Err(e) => {
                                log::warn!("Error parsing temperature: {}", e);
                                format!(
                                    "sensor,sender={},version={} voltage={},rssi={} {}",
                                    addr, reading.version, reading.voltage, rssi, timestamp
                                )
                            }
                        };
                        log::info!("{} (counter={})", payload, reading.counter);
                        client
                            .publish(&config.topic, QoS::AtLeastOnce, false, payload)
                            .await
                            .unwrap();
                    }
                    Some(Err(e)) => {
                        log::error!("Error reading manufacturer data {}", e);
                    }
                    None => {
                        log::warn!("Manufacurer data not found");
                    }
                }
            }
        }
        Ok::<(), bluer::Error>(())
    });

    loop {
        match eventloop.poll().await {
            Ok(notification) => log::debug!("Received: {:?}", notification),
            Err(e) => log::error!("Error on mqtt: {}", e),
        }
    }
}
