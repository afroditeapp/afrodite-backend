/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(rename = "admin_moderate_images", skip_serializing_if = "Option::is_none")]
    pub admin_moderate_images: Option<bool>,
    #[serde(rename = "admin_moderate_profiles", skip_serializing_if = "Option::is_none")]
    pub admin_moderate_profiles: Option<bool>,
    #[serde(rename = "admin_modify_capabilities", skip_serializing_if = "Option::is_none")]
    pub admin_modify_capabilities: Option<bool>,
    #[serde(rename = "admin_server_maintenance_reboot_backend", skip_serializing_if = "Option::is_none")]
    pub admin_server_maintenance_reboot_backend: Option<bool>,
    #[serde(rename = "admin_server_maintenance_reset_data", skip_serializing_if = "Option::is_none")]
    pub admin_server_maintenance_reset_data: Option<bool>,
    #[serde(rename = "admin_server_maintenance_save_backend_config", skip_serializing_if = "Option::is_none")]
    pub admin_server_maintenance_save_backend_config: Option<bool>,
    #[serde(rename = "admin_server_maintenance_update_software", skip_serializing_if = "Option::is_none")]
    pub admin_server_maintenance_update_software: Option<bool>,
    #[serde(rename = "admin_server_maintenance_view_backend_config", skip_serializing_if = "Option::is_none")]
    pub admin_server_maintenance_view_backend_config: Option<bool>,
    /// View server infrastructure related info like logs and software versions.
    #[serde(rename = "admin_server_maintenance_view_info", skip_serializing_if = "Option::is_none")]
    pub admin_server_maintenance_view_info: Option<bool>,
    /// View public and private profiles.
    #[serde(rename = "admin_view_all_profiles", skip_serializing_if = "Option::is_none")]
    pub admin_view_all_profiles: Option<bool>,
    #[serde(rename = "admin_view_private_info", skip_serializing_if = "Option::is_none")]
    pub admin_view_private_info: Option<bool>,
    #[serde(rename = "admin_view_profile_history", skip_serializing_if = "Option::is_none")]
    pub admin_view_profile_history: Option<bool>,
    /// View public profiles. Automatically enabled once initial image moderation is complete.
    #[serde(rename = "user_view_public_profiles", skip_serializing_if = "Option::is_none")]
    pub user_view_public_profiles: Option<bool>,
}

impl Capabilities {
    pub fn new() -> Capabilities {
        Capabilities {
            admin_moderate_images: None,
            admin_moderate_profiles: None,
            admin_modify_capabilities: None,
            admin_server_maintenance_reboot_backend: None,
            admin_server_maintenance_reset_data: None,
            admin_server_maintenance_save_backend_config: None,
            admin_server_maintenance_update_software: None,
            admin_server_maintenance_view_backend_config: None,
            admin_server_maintenance_view_info: None,
            admin_view_all_profiles: None,
            admin_view_private_info: None,
            admin_view_profile_history: None,
            user_view_public_profiles: None,
        }
    }
}


