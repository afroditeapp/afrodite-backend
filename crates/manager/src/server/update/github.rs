use std::path::Path;

use error_stack::{Result, ResultExt, report};
use futures::StreamExt;
use manager_config::file::SoftwareUpdateConfig;
use reqwest::{
    Client, StatusCode,
    header::{ACCEPT, USER_AGENT},
};
use serde_json::Value;
use tokio::io::AsyncWriteExt;

use super::UpdateError;

const GITHUB_API_VERSION: &str = "2022-11-28";

pub struct ReleaseAsset {
    pub name: String,
    pub id: i64,
}

pub struct GitHubApi<'a> {
    pub updater_config: &'a SoftwareUpdateConfig,
    pub client: &'a Client,
    pub user_agent: &'a str,
}

impl GitHubApi<'_> {
    pub async fn get_latest_release_asset(&self) -> Result<Option<ReleaseAsset>, UpdateError> {
        let config = self.updater_config;

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            config.github.owner, config.github.repository
        );

        let request = self
            .client
            .get(url)
            .header(ACCEPT, "application/vnd.github+json")
            .header(USER_AGENT, self.user_agent)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION);

        let request = if let Some(token) = config.github.token.clone() {
            request.bearer_auth(token)
        } else {
            request
        };

        let request = request.build().change_context(UpdateError::GitHubApi)?;

        let response = self
            .client
            .execute(request)
            .await
            .change_context(UpdateError::GitHubApi)?;

        let status = response.status();
        if status != StatusCode::OK {
            let text = response
                .text()
                .await
                .change_context(UpdateError::GitHubApi)?;

            return Err(report!(UpdateError::GitHubApi)
                .attach_printable(status)
                .attach_printable(text));
        }

        let json: Value = response
            .json()
            .await
            .change_context(UpdateError::GitHubApi)?;

        let assets = json
            .get("assets")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or_default();

        let mut selected_asset: Option<ReleaseAsset> = None;
        for a in assets {
            let Some(name) = a
                .as_object()
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
            else {
                return Err(report!(UpdateError::GitHubApi));
            };
            let Some(id) = a
                .as_object()
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_i64())
            else {
                return Err(report!(UpdateError::GitHubApi));
            };
            let Some(uploader) = a
                .as_object()
                .and_then(|v| v.get("uploader"))
                .and_then(|v| v.get("login"))
                .and_then(|v| v.as_str())
            else {
                return Err(report!(UpdateError::GitHubApi));
            };

            if name.ends_with(&config.github.file_name_ending) {
                if let Some(selected) = selected_asset {
                    return Err(report!(UpdateError::SotwareDownloadFailedAmbiguousFileName)
                        .attach_printable(selected.name.to_string())
                        .attach_printable(name.to_string()));
                } else {
                    if let Some(required_uploader) = &config.github.uploader
                        && uploader != required_uploader
                    {
                        return Err(
                            report!(UpdateError::SotwareDownloadFailedUnknownFileUploader)
                                .attach_printable(format!(
                                    "uploader: {uploader}, expected: {required_uploader}"
                                )),
                        );
                    }
                    selected_asset = Some(ReleaseAsset {
                        name: name.to_string(),
                        id,
                    });
                }
            }
        }

        Ok(selected_asset)
    }

    pub async fn download_asset(
        &self,
        asset: &ReleaseAsset,
        download_location: impl AsRef<Path>,
    ) -> Result<(), UpdateError> {
        let config = self.updater_config;

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/assets/{}",
            config.github.owner, config.github.repository, asset.id,
        );

        let request = self
            .client
            .get(url)
            .header(ACCEPT, "application/octet-stream")
            .header(USER_AGENT, self.user_agent)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION);

        let request = if let Some(token) = config.github.token.clone() {
            request.bearer_auth(token)
        } else {
            request
        };

        let request = request.build().change_context(UpdateError::GitHubApi)?;

        let response = self
            .client
            .execute(request)
            .await
            .change_context(UpdateError::GitHubApi)?;

        let status = response.status();
        if status != StatusCode::OK {
            return Err(report!(UpdateError::GitHubApi).attach_printable(status));
        }

        let mut file = tokio::fs::File::create(download_location)
            .await
            .change_context(UpdateError::FileWritingFailed)?;

        let mut stream = response.bytes_stream();
        while let Some(bytes) = stream.next().await {
            let bytes = bytes.change_context(UpdateError::GitHubApi)?;
            file.write_all(&bytes)
                .await
                .change_context(UpdateError::FileWritingFailed)?;
        }

        Ok(())
    }
}
