// === src/command.rs ===
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub enum Comando {
    Uno,
    W,
}

static RE_CMD_1: Lazy<Regex> = Lazy::new(|| Regex::new(r"^1+\s*$").unwrap());
static RE_CMD_W: Lazy<Regex> = Lazy::new(|| Regex::new(r"^W+\s*$").unwrap());

impl Comando {
    pub fn parse(input: &str) -> Option<Self> {
        if RE_CMD_1.is_match(input) {
            Some(Comando::Uno)
        } else if RE_CMD_W.is_match(input) {
            Some(Comando::W)
        } else {
            None
        }
    }
}

