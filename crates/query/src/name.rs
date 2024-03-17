use std::{
    fmt,
    hash::{Hash, Hasher},
    mem::size_of,
};

/// Pre-hashed, compile-time known name.
///
/// # Note about collisions
///
/// It's generally good to have a mechanism that checks for hash collisions against names wherever
/// you use them. This is because names do not check for collisions between themselves, as there is
/// no way to do that in Rust.
#[derive(Clone, Copy, Eq)]
pub struct Name {
    hash: u64,
    name: &'static str,
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Name {
    pub const fn new(name: &'static str) -> Self {
        Self {
            hash: Self::fxhash(name.as_bytes()),
            name,
        }
    }

    /// Compile-time fx hash implementation.
    const fn fxhash(bytes: &[u8]) -> u64 {
        const K: u64 = 0x517cc1b727220a95;

        let mut i = 0;
        let mut hash = 0_u64;
        macro_rules! add_to_hash {
            ($i:expr) => {
                hash = (hash.rotate_left(5) ^ $i).wrapping_mul(K);
            };
        }

        assert!(size_of::<usize>() <= 8);
        while bytes.len() >= size_of::<usize>() {
            add_to_hash!(u64::from_le_bytes([
                bytes[i],
                bytes[1 + i],
                bytes[2 + i],
                bytes[3 + i],
                bytes[4 + i],
                bytes[5 + i],
                bytes[6 + i],
                bytes[7 + i],
            ]));
            i += size_of::<usize>();
        }
        if (size_of::<usize>() > 4) && (bytes.len() >= 4) {
            add_to_hash!(
                u32::from_ne_bytes([bytes[i], bytes[1 + i], bytes[2 + i], bytes[3 + i]]) as u64
            );
            i += 4;
        }
        if (size_of::<usize>() > 2) && bytes.len() >= 2 {
            add_to_hash!(u16::from_ne_bytes([bytes[i], bytes[1 + i]]) as u64);
            i += 2;
        }
        if (size_of::<usize>() > 1) && !bytes.is_empty() {
            add_to_hash!(bytes[i] as u64);
        }

        hash
    }
}
