use std::error::Error;
use futures::Future;

/// Trait defining the application use cases. 
/// Components like CLI or UI hook onto this abstraction to execute logic.
pub trait AppUseCases {
    // Future: I promise to eventually give you a result (useful to make the UI wait for an async result)
    // Output = ... : When the task finishes, this is the type you get back.
    // "Result<(), Box<dyn Error>>": The task succeeds with nothing (()), 
    // or fails with "any kind of error" (Box<dyn Error>).
    // "+ Send": This task is safe to move to a different CPU thread if needed. Required to make it "Future"
    fn advertise(&self) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn discover(&self) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn send(&self, ip: String, port: u16, file_path: String) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn receive(&self, port: u16) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
}
