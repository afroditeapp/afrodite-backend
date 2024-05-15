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
pub struct DemoModeToken {
    #[serde(rename = "token")]
    pub token: String,
}

impl DemoModeToken {
    pub fn new(token: String) -> DemoModeToken {
        DemoModeToken {
            token,
        }
    }
}


