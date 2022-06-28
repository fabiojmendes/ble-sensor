use std::time::{SystemTime, UNIX_EPOCH};

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
    temp: f64,
    voltage: u16,
    version: u8,
    counter: u8,
}

impl TryFrom<&[u8]> for TempReading {
    type Error = ();

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 6 {
            return Err(());
        }
        let temp = f64::from(LittleEndian::read_i16(&data[..2])) / 100.0;
        let voltage = LittleEndian::read_u16(&data[2..4]);
        let version = data[4];
        let counter = data[5];

        Ok(TempReading {
            temp,
            voltage,
            version,
            counter,
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
    let config = Config::parse();
    env_logger::builder().format_timestamp(None).init();

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

        let mut last_count = 0u8;

        let mut device_events = adapter.discover_devices_with_changes().await?;
        while let Some(evt) = device_events.next().await {
            if let AdapterEvent::DeviceAdded(addr) = evt {
                let device = adapter.device(addr)?;
                if device.name().await? != Some(String::from("BLETempSensor")) {
                    continue;
                }
                let rssi = device.rssi().await?.unwrap_or(0);
                if let Some(reading) = device
                    .manufacturer_data()
                    .await?
                    .and_then(|mut d| d.remove(&0xffff))
                    .and_then(|data| TempReading::try_from(&*data).ok())
                {
                    if reading.counter == last_count {
                        continue;
                    }
                    last_count = reading.counter;
                    let timestamp = timestamp_nanos();
                    let payload = format!(
                        "sensor,sender={},version={} temperature={:.2},voltage={},rssi={} {}",
                        addr, reading.version, reading.temp, reading.voltage, rssi, timestamp
                    );
                    log::info!("{} counter={}", payload, reading.counter);
                    client
                        .publish(&config.topic, QoS::AtLeastOnce, false, payload)
                        .await
                        .unwrap();
                }
            }
        }
        Ok::<(), bluer::Error>(())
    });

    loop {
        match eventloop.poll().await {
            Ok(notification) => log::debug!("Received: {:?}", notification),
            Err(e) => log::warn!("Error on mqtt: {}", e),
        }
    }
}
