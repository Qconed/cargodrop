pub fn setup_logger() {
    env_logger::Builder::from_default_env()
        .format_module_path(false)
        .format_timestamp_millis()
        .init();
}