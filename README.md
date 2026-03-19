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