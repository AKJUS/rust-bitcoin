// SPDX-License-Identifier: CC0-1.0

use crate::consensus::encode::{ReadExt, WriteExt, MAX_COMPACT_SIZE};
use crate::consensus::{Error, ParseError};
use crate::io::Cursor;

#[kani::unwind(10)] // Unwind recursion for read/write operations
#[kani::proof]
fn check_compact_size_roundtrip() {
    let x: u32 = kani::any();
    kani::assume(x <= MAX_COMPACT_SIZE as u32);
    let mut bytes = [0u8; 9];
    let mut cursor = Cursor::new(&mut bytes[..]);
    cursor.emit_compact_size(x).unwrap();
    cursor.set_position(0);
    let y = cursor.read_compact_size().unwrap();
    assert_eq!(u64::from(x), y);
}

#[kani::unwind(10)]
#[kani::proof]
fn check_oversized_compact_size_is_rejected() {
    let x: u64 = kani::any();
    kani::assume(x > MAX_COMPACT_SIZE as u64);
    let mut bytes = [0u8; 9];
    let mut cursor = Cursor::new(&mut bytes[..]);
    cursor.emit_compact_size(x).unwrap();
    cursor.set_position(0);
    assert!(matches!(
        cursor.read_compact_size(),
        Err(Error::Parse(ParseError::OversizedCompactSize))
    ));
}
