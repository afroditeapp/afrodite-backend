/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// PendingNotificationToken : PendingNotificationToken is used as a token for pending notification API access.  The token is 256 bit random value which is Base64 encoded. The token lenght in characters is 44.  OWASP recommends at least 128 bit session IDs. https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PendingNotificationToken {
    #[serde(rename = "token")]
    pub token: String,
}

impl PendingNotificationToken {
    /// PendingNotificationToken is used as a token for pending notification API access.  The token is 256 bit random value which is Base64 encoded. The token lenght in characters is 44.  OWASP recommends at least 128 bit session IDs. https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
    pub fn new(token: String) -> PendingNotificationToken {
        PendingNotificationToken {
            token,
        }
    }
}


