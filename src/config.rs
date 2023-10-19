#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Config {
    pub resilient: bool,
    pub display_errors: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resilient: true,
            display_errors: false,
        }
    }
}
