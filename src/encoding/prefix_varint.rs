//! # Prefix-varint encoding and decoding
//!
//! This file implements the encoding described here:
//! <https://github.com/WebAssembly/design/issues/601#issuecomment-196022303>

fn unaligned_load_u64(p: &[u8]) -> u64 {
  let mut array = [0u8; 8];
  let len = p.len().min(8);
  array[..len].copy_from_slice(&p[..len]);
  u64::from_le_bytes(array)
}

fn length(initial: u8) -> u32 {
  1 + (initial as u32 | 0x100).trailing_zeros()
}

/// Decodes an unsigned 64-bit integer from a byte slice, assuming the prefix-varint format.
pub fn decode(p: &[u8]) -> u64 {
  let length = length(*p.first().unwrap());
  if length < 9 {
    let unused = 64 - 8 * length;
    unaligned_load_u64(p) << unused >> (unused + length)
  } else {
    unaligned_load_u64(&p[1..])
  }
}

/// Encodes an unsigned 64-bit integer into a byte vector, using the prefix-varint format.
pub fn encode(x: u64, output: &mut Vec<u8>) {
  let bits = 64 - (x | 1).leading_zeros();
  let mut bytes = 1 + (bits - 1) / 7;
  let mut x = x;
  if bits > 56 {
    output.push(0);
    bytes = 8;
  } else {
    x = (x << bytes) | (1 << (bytes - 1));
  }
  for _ in 0..bytes {
    output.push((x & 0xff) as u8);
    x >>= 8;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::Rng;

  #[test]
  fn test_unaligned_load_u64() {
    // Test loading less than 8 bytes.
    assert_eq!(unaligned_load_u64(&[0x01, 0x02, 0x03]), 0x30201);
    // Test loading exactly 8 bytes.
    assert_eq!(unaligned_load_u64(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]), 0x807060504030201);
    // Test loading more than 8 bytes (should only load first 8).
    assert_eq!(unaligned_load_u64(&[0xFF; 16]), 0xFFFFFFFFFFFFFFFF);
    // Test loading empty slice (should return 0).
    assert_eq!(unaligned_load_u64(&[]), 0);
  }

  #[test]
  fn test_length() {
    // Test various initial bytes and expected lengths.
    let test_cases = vec![
      (0b00000001, 1),
      (0b00000010, 2),
      (0b00000100, 3),
      (0b00001000, 4),
      (0b00010000, 5),
      (0b00100000, 6),
      (0b01000000, 7),
      (0b10000000, 8),
      (0b00000000, 9),
    ];
    for (initial, expected) in test_cases {
      assert_eq!(length(initial), expected, "Initial byte: {:08b}", initial);
    }
  }

  #[test]
  fn test_specific_encode() {
    // Test specific known encodings.
    let test_cases = vec![
      (0, vec![0x01]),
      (1, vec![0x03]),
      (127, vec![0xFF]),
      (128, vec![0x02, 0x02]),
      (255, vec![0xFE, 0x03]),
      (8192, vec![0x02, 0x80]),
      (16383, vec![0xFE, 0xFF]),
      (16384, vec![0x04, 0x00, 0x02]),
      (u64::MAX, vec![0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
    ];
    for (decoded, encoded) in test_cases {
      let mut output = Vec::new();
      encode(decoded, &mut output);
      assert_eq!(output, encoded, "Failed for decoded value: {}", decoded);
    }
  }

  #[test]
  fn test_specific_decode() {
    // Test specific known decodings.
    let test_cases = vec![
      (0, vec![0x01]),
      (1, vec![0x03]),
      (127, vec![0xFF]),
      (128, vec![0x02, 0x02]),
      (255, vec![0xFE, 0x03]),
      (8192, vec![0x02, 0x80]),
      (16383, vec![0xFE, 0xFF]),
      (16384, vec![0x04, 0x00, 0x02]),
      (u64::MAX, vec![0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
    ];
    for (decoded, encoded) in test_cases {
      let result = decode(&encoded);
      assert_eq!(result, decoded, "Failed for encoded bytes: {:?}", encoded);
    }
  }

  #[test]
  fn test_round_trip_boundary_values() {
    // Test round-trip encoding and decoding using boundary values.
    let test_values = vec![
      0u64,
      1,
      (1 << 7) - 1,
      1 << 7,
      (1 << 14) - 1,
      1 << 14,
      (1 << 21) - 1,
      1 << 21,
      (1 << 28) - 1,
      1 << 28,
      (1 << 35) - 1,
      1 << 35,
      (1 << 42) - 1,
      1 << 42,
      (1 << 49) - 1,
      1 << 49,
      (1 << 56) - 1,
      1 << 56,
      u64::MAX,
    ];
    for &value in &test_values {
      let mut encoded = Vec::new();
      encode(value, &mut encoded);
      let decoded = decode(&encoded);
      assert_eq!(decoded, value, "Round-trip failed for value: {}. Encoded bytes: {:?}", value, encoded);
    }
  }

  #[test]
  fn test_round_trip_random_values() {
    // Test round-trip encoding and decoding using random values.
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
      let value: u64 = rng.gen();
      let mut encoded = Vec::new();
      encode(value, &mut encoded);
      let decoded = decode(&encoded);
      assert_eq!(decoded, value, "Round-trip failed for value: {}. Encoded bytes: {:?}", value, encoded);
    }
  }
}
