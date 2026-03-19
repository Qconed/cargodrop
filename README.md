# cargodrop
An airdrop application fully written in Rust for effiicency ! Enables file transfers to others close by, withouth the need of a Internet or Wifi connection between devices.





# Turning on bluetooth advertising and discovering raspberry pi

sudo bluetoothctl system-alias "RaspberryPi-CargoDrop"
sudo bluetoothctl
menu advertise
name on
back
advertise on

then cargo run and raspberry will be detected.

sudo bluetoothctl advertise off # to turn off advertising

# Generating a Custom UUID

If you want to change the application's unique identifier (UUID) used for Bluetooth Service Discovery:

1. **Using Terminal**: Run `uuidgen` in your Linux/macOS terminal to generate a random 128-bit Version 4 UUID.
2. **Programmatically**: Use the `uuid` Rust crate: `uuid::Uuid::new_v4().to_string()`.

Once generated, replace the existing `APP_SERVICE_UUID` constant in `src/ble/mod.rs` with your new 128-bit UUID string.