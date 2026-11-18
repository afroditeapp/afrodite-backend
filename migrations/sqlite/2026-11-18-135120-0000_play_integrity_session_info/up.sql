ALTER TABLE login_session_info
    ADD COLUMN app_attestation_android_strong_integrity BOOLEAN;

ALTER TABLE login_session_info
    ADD COLUMN app_attestation_android_sdk_version SMALLINT;
