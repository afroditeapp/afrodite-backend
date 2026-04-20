use crate::{ContentInfo, GetProfileContentResult, ProfileContent, ProfileContentVersion};

const RESULT_VARIANT_EMPTY: u8 = 0;
const RESULT_VARIANT_VERSION_ONLY: u8 = 1;
const RESULT_VARIANT_CONTENT_WITH_VERSION: u8 = 2;

impl GetProfileContentResult {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        match (&self.content, &self.version) {
            (None, None) => {
                buffer.push(RESULT_VARIANT_EMPTY);
            }
            (None, Some(version)) => {
                buffer.push(RESULT_VARIANT_VERSION_ONLY);
                append_profile_content_version(&mut buffer, version);
            }
            (Some(content), Some(version)) => {
                buffer.push(RESULT_VARIANT_CONTENT_WITH_VERSION);
                append_profile_content_version(&mut buffer, version);
                append_profile_content(&mut buffer, content);
            }
            (Some(_), None) => {
                // Keep wire format consistent: content payload requires version.
                buffer.push(RESULT_VARIANT_EMPTY);
            }
        }

        buffer
    }
}

fn append_profile_content_version(buffer: &mut Vec<u8>, version: &ProfileContentVersion) {
    buffer.extend_from_slice(version.as_ref().as_bytes());
}

fn append_profile_content(buffer: &mut Vec<u8>, content: &ProfileContent) {
    buffer.push((content.verification_status.v as u16 & 0x00ff) as u8);

    // Current server-side profile content has max 6 items.
    let content_count = content.content.len().min(6);
    buffer.push(content_count as u8);

    for item in content.content.iter().take(content_count) {
        buffer.extend_from_slice(item.cid.as_ref().as_bytes());
        buffer.push(pack_content_info(item));
    }

    let mut crop_presence_mask = 0u8;
    if content.grid_crop_size.is_some() {
        crop_presence_mask |= 0x01;
    }
    if content.grid_crop_x.is_some() {
        crop_presence_mask |= 0x02;
    }
    if content.grid_crop_y.is_some() {
        crop_presence_mask |= 0x04;
    }
    buffer.push(crop_presence_mask);

    if let Some(value) = content.grid_crop_size {
        buffer.extend_from_slice(&(value as f32).to_le_bytes());
    }
    if let Some(value) = content.grid_crop_x {
        buffer.extend_from_slice(&(value as f32).to_le_bytes());
    }
    if let Some(value) = content.grid_crop_y {
        buffer.extend_from_slice(&(value as f32).to_le_bytes());
    }
}

fn pack_content_info(info: &ContentInfo) -> u8 {
    let ctype_bits = ((info.ctype as i16 as u8) & 0x07) << 5;
    let accepted_bit = u8::from(info.accepted) << 4;
    let face_detected_bit = u8::from(info.face_detected) << 3;
    let face_verified_bits = match info.face_verified {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    };

    ctype_bits | accepted_bit | face_detected_bit | face_verified_bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContentId, MediaContentType, MediaVerificationStatus};

    fn test_uuid(value: u8) -> simple_backend_utils::UuidBase64Url {
        simple_backend_utils::UuidBase64Url::from_bytes([value; 16])
    }

    fn test_content_id(value: u8) -> ContentId {
        ContentId::try_from(test_uuid(value)).expect("test uuid should convert to content id")
    }

    fn test_profile_content_version(value: u8) -> ProfileContentVersion {
        ProfileContentVersion::new_base_64_url(test_uuid(value))
    }

    #[test]
    fn get_profile_content_result_binary_empty() {
        let data = GetProfileContentResult::empty().to_binary();
        assert_eq!(data, vec![RESULT_VARIANT_EMPTY]);
    }

    #[test]
    fn get_profile_content_result_binary_version_only() {
        let version = test_profile_content_version(7);
        let data = GetProfileContentResult::current_version_latest_response(version).to_binary();

        assert_eq!(data[0], RESULT_VARIANT_VERSION_ONLY);
        assert_eq!(data.len(), 1 + 16);
        assert_eq!(&data[1..17], test_uuid(7).as_bytes());
    }

    #[test]
    fn get_profile_content_result_binary_content_with_version() {
        let version = test_profile_content_version(9);
        let content = ProfileContent {
            content: vec![
                ContentInfo {
                    cid: test_content_id(1),
                    ctype: MediaContentType::JpegImage,
                    accepted: true,
                    face_detected: true,
                    face_verified: Some(true),
                },
                ContentInfo {
                    cid: test_content_id(2),
                    ctype: MediaContentType::JpegImage,
                    accepted: false,
                    face_detected: false,
                    face_verified: Some(false),
                },
            ],
            verification_status: MediaVerificationStatus { v: 0x0102 },
            grid_crop_size: Some(1.5),
            grid_crop_x: None,
            grid_crop_y: Some(-2.0),
        };

        let data = GetProfileContentResult::content_with_version(content, version).to_binary();

        assert_eq!(data[0], RESULT_VARIANT_CONTENT_WITH_VERSION);
        assert_eq!(&data[1..17], test_uuid(9).as_bytes());
        assert_eq!(data[17], 0x02); // truncated low byte of 0x0102
        assert_eq!(data[18], 2); // content_count

        // First content packed byte: ctype=0, accepted=1, face_detected=1, face_verified=Some(true)=2
        // => 0001_1010b = 0x1A
        assert_eq!(data[35], 0x1A);

        // Second content packed byte: ctype=0, accepted=0, face_detected=0, face_verified=Some(false)=1
        // => 0000_0001b = 0x01
        assert_eq!(data[52], 0x01);

        // Crop mask: size + y
        assert_eq!(data[53], 0x05);

        let size_bytes = f32::to_le_bytes(1.5);
        let y_bytes = f32::to_le_bytes(-2.0);
        assert_eq!(&data[54..58], size_bytes);
        assert_eq!(&data[58..62], y_bytes);
    }

    #[test]
    fn pack_content_info_face_verified_mapping() {
        let mut info = ContentInfo {
            cid: test_content_id(3),
            ctype: MediaContentType::JpegImage,
            accepted: false,
            face_detected: false,
            face_verified: None,
        };

        assert_eq!(pack_content_info(&info), 0);

        info.face_verified = Some(false);
        assert_eq!(pack_content_info(&info), 1);

        info.face_verified = Some(true);
        assert_eq!(pack_content_info(&info), 2);
    }
}
