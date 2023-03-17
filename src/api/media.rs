pub mod data;
pub mod internal;

use axum::{extract::Path};

use hyper::{StatusCode};





use self::{
    super::model::{AccountId},
};

use self::data::ImageFileName;



use super::{
    ReadDatabase,
};

pub const PATH_GET_IMAGE: &str = "/image/:user_id/:image_file";

#[utoipa::path(
    get,
    path = "/image/{user_id}/{image_file}",
    params(AccountId, ImageFileName),
    responses(
        (status = 200, description = "Get image file.", content_type = "image/jpeg"),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_image<S: ReadDatabase>(
    Path(_user_id): Path<AccountId>,
    Path(_image_file): Path<ImageFileName>,
    _state: S,
) -> Result<(), StatusCode> {
    // TODO: Validate user id
    // state
    //     .read_database()
    //     .user_profile(&user_id)
    //     .await
    //     .map(|profile| ()) // TODO: Read and send image.
    //     .map_err(|e| {
    //         error!("Get profile error: {e:?}");
    //         StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
    //     })
    Ok(())
}
