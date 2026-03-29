pub fn add_minimal_i64(buffer: &mut Vec<u8>, value: i64) {
    if let Ok(value) = TryInto::<i8>::try_into(value) {
        buffer.push(1);
        buffer.extend_from_slice(&value.to_le_bytes());
    } else if let Ok(value) = TryInto::<i16>::try_into(value) {
        buffer.push(2);
        buffer.extend_from_slice(&value.to_le_bytes());
    } else if let Ok(value) = TryInto::<i32>::try_into(value) {
        buffer.push(4);
        buffer.extend_from_slice(&value.to_le_bytes());
    } else {
        buffer.push(8);
        buffer.extend_from_slice(&value.to_le_bytes());
    }
}

pub fn parse_minimal_i64_from_iter(iter: &mut impl Iterator<Item = u8>) -> Option<i64> {
    let marker = iter.next()?;
    parse_minimal_i64_from_iter_with_marker(marker, iter)
}

fn parse_minimal_i64_from_iter_with_marker(
    marker: u8,
    iter: &mut impl Iterator<Item = u8>,
) -> Option<i64> {
    let value = match marker {
        1 => i8::from_le_bytes([iter.next()?]).into(),
        2 => i16::from_le_bytes([iter.next()?, iter.next()?]).into(),
        4 => i32::from_le_bytes([iter.next()?, iter.next()?, iter.next()?, iter.next()?]).into(),
        8 => i64::from_le_bytes([
            iter.next()?,
            iter.next()?,
            iter.next()?,
            iter.next()?,
            iter.next()?,
            iter.next()?,
            iter.next()?,
            iter.next()?,
        ]),
        _ => return None,
    };

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
            i32::MAX as i64,
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
    fn minimal_i64_invalid_marker_returns_none() {
        let mut payload = [9].into_iter();
        assert_eq!(parse_minimal_i64_from_iter(&mut payload), None);
    }
}
