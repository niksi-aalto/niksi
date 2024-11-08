use std::path::Path;

mod open;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[derive(Debug, Clone, Default)]
pub struct NiksiConfig {
    nix: String,
}

impl NiksiConfig {
    pub fn new(nix: String) -> Self {
        Self { nix }
    }
}

#[derive(Debug, Clone)]
pub struct Niksi {
    pub config: NiksiConfig,
}
