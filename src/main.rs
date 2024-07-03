use std::{path::Path, time::Duration};

use aranet4::{
    error::SensorError,
    readings::SensorReadings,
    sensor::{Sensor, SensorManager},
};
use sonnerie::CreateTx;

async fn connecting() -> Result<Sensor, SensorError> {
    let mut wait = Duration::from_secs(1);
    let mut tries = 100;
    loop {
        match SensorManager::init(None).await {
            Ok(sensor) => return Ok(sensor),
            Err(e) if tries == 0 => return Err(e),
            Err(_) => (),
        }
        println!("Not connected waiting {wait:?} before next connection...");
        tokio::time::sleep(wait).await;
        wait *= 2;
        tries -= 1;
    }
}

#[tokio::main]
async fn main() {
    let path: &Path = "measurements.son".as_ref();

    for _ in 0..100 {
        let sensor = connecting().await.unwrap();
        println!("Connected 🎉");
        println!("measuring!");
        let SensorReadings {
            co2_level,
            temperature,
            pressure,
            humidity,
            battery,
            // status_color,
            ..
        } = sensor.read_current_values().await.unwrap();
        let celsius = (temperature - 32.0) / 1.8;

        eprintln!("opening txn...");

        tokio::task::spawn_blocking(move || {
            let mut txn = CreateTx::new(path).unwrap();
            println!("txn opened");
            let timestamp = chrono::Utc::now().naive_local();
            // temperature, pressure, CO₂, humidity, battery, ~status color~
            txn.add_record(
                "aranet4",
                timestamp,
                sonnerie::record(celsius)
                    .add(pressure)
                    .add(co2_level as u32)
                    .add(humidity as u32)
                    .add(battery as u32), // .add(status_color as u32),
            )
            .unwrap();
            println!("commiting");
            txn.commit().unwrap();
        })
        .await
        .unwrap();

        let wait = Duration::from_secs(300); // 5 min
        println!("awaiting {wait:?} before next measure...");
        tokio::time::sleep(wait).await;
    }
}
