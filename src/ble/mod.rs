pub mod advertise;
pub mod discover;

// The uniquely generated UUID used to identify CargoDrop's BLE service.
// This allows filtering out other random BLE devices nearby.
pub const APP_SERVICE_UUID: &str = "d59218d6-6b22-4a0b-9ba7-70e28148b488";
