//! Deterministic address generation for the game ledger. Nobody signs anything
//! here; the addresses only need to look and classify like real ones.

use bitcoin::address::NetworkUnchecked;
use bitcoin::hashes::{sha256, Hash};
use bitcoin::key::TweakedPublicKey;
use bitcoin::secp256k1::XOnlyPublicKey;
use bitcoin::{Address, Network, PubkeyHash, ScriptBuf, WPubkeyHash};
use serde::{Deserialize, Serialize};

/// The three address families the talk tells apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptKind {
    Legacy,
    Segwit,
    Taproot,
}

impl ScriptKind {
    pub fn of(address: &Address<NetworkUnchecked>) -> Option<ScriptKind> {
        let script = address.clone().assume_checked().script_pubkey();
        if script.is_p2pkh() {
            Some(ScriptKind::Legacy)
        } else if script.is_p2wpkh() {
            Some(ScriptKind::Segwit)
        } else if script.is_p2tr() {
            Some(ScriptKind::Taproot)
        } else {
            None
        }
    }
}

/// Hands out fresh addresses, reproducibly from a seed.
#[derive(Debug, Clone)]
pub struct AddressBook {
    seed: [u8; 32],
    counter: u64,
}

impl AddressBook {
    pub fn new(seed: [u8; 32]) -> Self {
        Self { seed, counter: 0 }
    }

    fn next_hash(&mut self) -> [u8; 32] {
        let mut data = self.seed.to_vec();
        data.extend_from_slice(&self.counter.to_le_bytes());
        self.counter += 1;
        sha256::Hash::hash(&data).to_byte_array()
    }

    pub fn fresh(&mut self, kind: ScriptKind) -> Address<NetworkUnchecked> {
        let checked = match kind {
            ScriptKind::Legacy => {
                let hash = self.next_hash();
                let mut short = [0u8; 20];
                short.copy_from_slice(&hash[..20]);
                Address::p2pkh(PubkeyHash::from_byte_array(short), Network::Bitcoin)
            }
            ScriptKind::Segwit => loop {
                let hash = self.next_hash();
                let mut short = [0u8; 20];
                short.copy_from_slice(&hash[..20]);
                let script = ScriptBuf::new_p2wpkh(&WPubkeyHash::from_byte_array(short));
                // A 20-byte program is always a valid p2wpkh script.
                if let Ok(a) = Address::from_script(&script, Network::Bitcoin) {
                    break a;
                }
            },
            ScriptKind::Taproot => loop {
                // About half of all 32-byte strings are valid x coordinates.
                if let Ok(key) = XOnlyPublicKey::from_slice(&self.next_hash()) {
                    break Address::p2tr_tweaked(
                        TweakedPublicKey::dangerous_assume_tweaked(key),
                        Network::Bitcoin,
                    );
                }
            },
        };
        checked.as_unchecked().clone()
    }

    /// A Silent Payments address. Only its shape matters to the game: the
    /// string never reaches the ledger, payments to it land on fresh Taproot
    /// addresses from [`AddressBook::fresh`].
    pub fn silent_payment(&mut self) -> String {
        let hash = self.next_hash();
        let body: String = hash.iter().map(|b| CHARSET[(*b % 32) as usize]).collect();
        format!("sp1q{body}{}", &body[..5])
    }
}

const CHARSET: [char; 32] = [
    'q', 'p', 'z', 'r', 'y', '9', 'x', '8', 'g', 'f', '2', 't', 'v', 'd', 'w', '0', 's', '3', 'j',
    'n', '5', '4', 'k', 'h', 'c', 'e', '6', 'm', 'u', 'a', '7', 'l',
];

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> AddressBook {
        AddressBook::new([7u8; 32])
    }

    #[test]
    fn each_kind_produces_its_own_prefix() {
        let mut book = book();
        assert!(book
            .fresh(ScriptKind::Legacy)
            .assume_checked()
            .to_string()
            .starts_with('1'));
        assert!(book
            .fresh(ScriptKind::Segwit)
            .assume_checked()
            .to_string()
            .starts_with("bc1q"));
        assert!(book
            .fresh(ScriptKind::Taproot)
            .assume_checked()
            .to_string()
            .starts_with("bc1p"));
    }

    #[test]
    fn classification_round_trips() {
        let mut book = book();
        for kind in [ScriptKind::Legacy, ScriptKind::Segwit, ScriptKind::Taproot] {
            assert_eq!(ScriptKind::of(&book.fresh(kind)), Some(kind));
        }
    }

    #[test]
    fn addresses_never_repeat_but_seeds_reproduce() {
        let mut a = book();
        let mut b = book();
        let first: Vec<_> = (0..50).map(|_| a.fresh(ScriptKind::Segwit)).collect();
        let second: Vec<_> = (0..50).map(|_| b.fresh(ScriptKind::Segwit)).collect();
        assert_eq!(first, second);
        let unique: std::collections::HashSet<_> = first.iter().collect();
        assert_eq!(unique.len(), 50);
    }

    #[test]
    fn silent_payment_address_has_the_sp1_shape() {
        let sp = book().silent_payment();
        assert!(sp.starts_with("sp1q"));
        assert!(sp.len() > 40);
        assert!(sp
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }
}
