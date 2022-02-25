/// Containers related errors
#[derive(Debug)]
pub enum Error {
}

/// A common result type for our container module.
pub type Result<T> = std::result::Result<T, Error>;

/// The `Container` struct provides a simple way to
/// create and run a container on the host.
#[derive(Default)]
pub struct Quardle {
}

impl Quardle {
    /// Build a new container with the bundle provided in parameters.
    pub fn new() -> Result<Self> {
      Ok(Quardle {})
    }
}