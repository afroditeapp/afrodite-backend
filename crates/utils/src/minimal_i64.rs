pub fn add_minimal_i64(buffer: &mut Vec<u8>, value: i64) {
    let bytes = value.to_le_bytes();

    let mut marker = 8_usize;
    while marker > 1 {
        let next_msb = bytes[marker - 2];
        let sign_extension = if next_msb & 0x80 == 0 { 0x00 } else { 0xFF };

        if bytes[marker - 1] == sign_extension {
            marker -= 1;
        } else {
            break;
        }
    }

    buffer.push(marker as u8);
    buffer.extend_from_slice(&bytes[..marker]);
}

pub fn parse_minimal_i64_from_iter(iter: &mut impl Iterator<Item = u8>) -> Option<i64> {
    let marker = iter.next()?;
    parse_minimal_i64_from_iter_with_marker(marker, iter)
}

fn parse_minimal_i64_from_iter_with_marker(
    marker: u8,
    iter: &mut impl Iterator<Item = u8>,
) -> Option<i64> {
    if !(1..=8).contains(&marker) {
        return None;
    }

    let mut bytes = [0_u8; 8];
    for b in bytes.iter_mut().take(marker as usize) {
        *b = iter.next()?;
    }

    if bytes[marker as usize - 1] & 0x80 != 0 {
        bytes[marker as usize..].fill(0xFF);
    }

    let value = i64::from_le_bytes(bytes);

    Some(value)
}

#[cfg(test)]
mod tests {
    use super::{add_minimal_i64, parse_minimal_i64_from_iter};

    #[test]
    fn minimal_i64_roundtrip_with_iter_parser() {
        let values = [
            0,
            1,
            -1,
            i8::MAX as i64,
            i8::MIN as i64,
            8_388_607,
            -8_388_608,
            i32::MAX as i64,
            -2_147_483_648,
            549_755_813_887,
            -549_755_813_888,
            140_737_488_355_327,
            -140_737_488_355_328,
            i64::MIN,
        ];

        let mut bytes = Vec::new();
        for value in values {
            add_minimal_i64(&mut bytes, value);
        }

        let mut iter = bytes.into_iter();
        for expected in values {
            assert_eq!(parse_minimal_i64_from_iter(&mut iter), Some(expected));
        }
        assert_eq!(parse_minimal_i64_from_iter(&mut iter), None);
    }

    #[test]
    fn minimal_i64_uses_all_supported_lengths() {
        let values_and_markers = [
            (0_i64, 1_u8),
            (128_i64, 2_u8),
            (8_388_607_i64, 3_u8),
            (2_147_483_647_i64, 4_u8),
            (549_755_813_887_i64, 5_u8),
            (140_737_488_355_327_i64, 6_u8),
            (36_028_797_018_963_967_i64, 7_u8),
            (i64::MAX, 8_u8),
        ];

        for (value, expected_marker) in values_and_markers {
            let mut bytes = Vec::new();
            add_minimal_i64(&mut bytes, value);
            assert_eq!(bytes.first().copied(), Some(expected_marker));
        }
    }

    #[test]
    fn minimal_i64_invalid_marker_returns_none() {
        let mut payload = [9].into_iter();
        assert_eq!(parse_minimal_i64_from_iter(&mut payload), None);
    }
}
