use std::{collections::HashMap, fs::File, io::Read, path::Path, str::FromStr};

use axum::body::Bytes;
use error_stack::ResultExt;
use flate2::read::GzDecoder;
use headers::ContentType;
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;
use tar::{Archive, EntryType};
use tracing::warn;

#[derive(thiserror::Error, Debug)]
pub enum FilePackageError {
    #[error("File package loading failed")]
    PackageLoading,
    #[error("File package contains invalid UTF-8")]
    InvalidUtf8,
    #[error("File package contains unknown file type")]
    PackageContainsUnknonwFileType,
    #[error("Invalid MIME type string in source code")]
    InvalidMimeType,
}

pub struct FilePackageManager {
    file_path_and_data: HashMap<String, (ContentType, Bytes)>,
}

impl FilePackageManager {
    pub async fn new(config: &SimpleBackendConfig) -> error_stack::Result<Self, FilePackageError> {
        let mime_types = ExtraMimeTypes::new().change_context(FilePackageError::InvalidMimeType)?;
        let package_path = if let Some(c) = config.file_package() {
            c.clone()
        } else {
            return Ok(Self {
                file_path_and_data: HashMap::new(),
            });
        };

        let result: error_stack::Result<Self, FilePackageError> =
            tokio::task::spawn_blocking(move || {
                let file_path = Path::new(&package_path.package);
                if !file_path.exists() {
                    warn!(
                        "Static file hosting package does not exist at location {}",
                        package_path.package.display()
                    );
                    return Ok(Self {
                        file_path_and_data: HashMap::new(),
                    });
                }
                let mut read_files_only_from_dir = package_path.read_from_dir.clone();
                if let Some(d) = &mut read_files_only_from_dir {
                    d.push('/');
                }

                let mut file_path_and_data = HashMap::new();
                let file = File::open(package_path.package)
                    .change_context(FilePackageError::PackageLoading)?;
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);
                let entries = archive
                    .entries()
                    .change_context(FilePackageError::PackageLoading)?;
                for e in entries {
                    let mut e = e.change_context(FilePackageError::PackageLoading)?;
                    if e.header().entry_type() == EntryType::Directory {
                        continue;
                    }
                    let path = e.path().change_context(FilePackageError::PackageLoading)?;
                    // Skip hidden files
                    if path
                        .file_name()
                        .and_then(|v| v.to_str())
                        .map(|v| v.starts_with('.'))
                        .unwrap_or_default()
                    {
                        continue;
                    }
                    let path_string = path
                        .to_str()
                        .ok_or(FilePackageError::InvalidUtf8)?
                        .to_string();
                    // Remove root directory from paths if needed
                    let path_string = if let Some(files_from_dir) = &read_files_only_from_dir {
                        if path_string.starts_with(files_from_dir) {
                            path_string.trim_start_matches(files_from_dir).to_string()
                        } else {
                            continue;
                        }
                    } else {
                        path_string
                    };
                    let mut data = vec![];
                    e.read_to_end(&mut data)
                        .change_context(FilePackageError::PackageLoading)?;
                    let data_bytes: Bytes = data.into();
                    let content_type =
                        Self::path_string_to_content_type(&mime_types, &path_string)?;
                    file_path_and_data.insert(path_string, (content_type, data_bytes));
                }
                Ok(Self { file_path_and_data })
            })
            .await
            .change_context(FilePackageError::PackageLoading)?;

        result
    }

    fn path_string_to_content_type(
        mime_types: &ExtraMimeTypes,
        path: &str,
    ) -> error_stack::Result<ContentType, FilePackageError> {
        let content_type = if path.ends_with(".html") {
            ContentType::html()
        } else if path.ends_with(".js") || path.ends_with(".mjs") {
            mime::APPLICATION_JAVASCRIPT_UTF_8.into()
        } else if path.ends_with(".json") {
            ContentType::json()
        } else if path.ends_with(".png") {
            ContentType::png()
        } else if path.ends_with(".pem") {
            mime::TEXT_PLAIN_UTF_8.into()
        } else if path.ends_with(".otf") {
            mime_types.otf.clone()
        } else if path.ends_with(".frag") {
            mime::TEXT_PLAIN_UTF_8.into()
        } else if path.ends_with(".bin") {
            ContentType::octet_stream()
        } else if path.ends_with(".symbols") {
            mime::TEXT_PLAIN_UTF_8.into()
        } else if path.ends_with(".wasm") {
            mime_types.wasm.clone()
        } else if path.ends_with("/NOTICES") {
            mime::TEXT_PLAIN_UTF_8.into()
        } else {
            return Err(FilePackageError::PackageContainsUnknonwFileType
                .report()
                .attach_printable(path.to_string()));
        };

        Ok(content_type)
    }

    pub fn data(&self, path: &str) -> Option<(ContentType, Bytes)> {
        self.file_path_and_data.get(path).cloned()
    }
}

struct ExtraMimeTypes {
    otf: ContentType,
    wasm: ContentType,
}

impl ExtraMimeTypes {
    pub fn new() -> Result<Self, headers::Error> {
        Ok(Self {
            otf: ContentType::from_str("font/otf")?,
            wasm: ContentType::from_str("application/wasm")?,
        })
    }
}
