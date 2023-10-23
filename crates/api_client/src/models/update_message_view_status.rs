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
pub struct UpdateMessageViewStatus {
    #[serde(rename = "account_id_sender")]
    pub account_id_sender: Box<crate::models::AccountId>,
    #[serde(rename = "message_number")]
    pub message_number: Box<crate::models::MessageNumber>,
}

impl UpdateMessageViewStatus {
    pub fn new(account_id_sender: crate::models::AccountId, message_number: crate::models::MessageNumber) -> UpdateMessageViewStatus {
        UpdateMessageViewStatus {
            account_id_sender: Box::new(account_id_sender),
            message_number: Box::new(message_number),
        }
    }
}


