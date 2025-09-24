use std::{
    collections::HashMap,
    fs::{File, ReadDir},
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use axum::body::Bytes;
use error_stack::ResultExt;
use flate2::read::GzDecoder;
use headers::{ContentEncoding, ContentType};
use regex::Regex;
use simple_backend_config::SimpleBackendConfig;
use simple_backend_model::VersionNumber;
use simple_backend_utils::{ContextExt, IntoReportFromString};
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
    #[error("Package contains multiple index.html files")]
    MultipleIndexHtmlFiles,
}

#[derive(Clone)]
pub struct StaticFile {
    pub content_type: ContentType,
    pub content_encoding: Option<ContentEncoding>,
    pub data: Bytes,
}

impl StaticFile {
    fn new(
        mime_types: &ExtraMimeTypes,
        path_string: String,
        data: Vec<u8>,
    ) -> error_stack::Result<(String, Self), FilePackageError> {
        if let Some(path_string) = path_string.strip_suffix(".gz").map(ToString::to_string) {
            let static_file = Self {
                content_type: FilePackageManager::path_string_to_content_type(
                    mime_types,
                    &path_string,
                )?,
                content_encoding: Some(ContentEncoding::gzip()),
                data: data.into(),
            };
            Ok((path_string, static_file))
        } else {
            let static_file = Self {
                content_type: FilePackageManager::path_string_to_content_type(
                    mime_types,
                    &path_string,
                )?,
                content_encoding: None,
                data: data.into(),
            };
            Ok((path_string, static_file))
        }
    }
}

pub struct FilePackageManager {
    index_html: Option<StaticFile>,
    file_path_and_data: HashMap<String, StaticFile>,
}

impl FilePackageManager {
    fn new_empty() -> Self {
        Self {
            index_html: None,
            file_path_and_data: HashMap::new(),
        }
    }

    pub async fn new(config: &SimpleBackendConfig) -> error_stack::Result<Self, FilePackageError> {
        let mime_types = ExtraMimeTypes::new().change_context(FilePackageError::InvalidMimeType)?;
        let package_config = if let Some(c) = config.file_package() {
            c.clone()
        } else {
            return Ok(Self::new_empty());
        };

        let result: error_stack::Result<Self, FilePackageError> =
            tokio::task::spawn_blocking(move || {
                let mut manager = Self::new_empty();

                if let Some(single_package) = package_config.package {
                    manager.handle_single_package_mode(&mime_types, &single_package)?
                } else if let Some(dir) = package_config.package_dir {
                    manager.handle_package_dir_mode(&mime_types, &dir)?
                }

                Ok(manager)
            })
            .await
            .change_context(FilePackageError::PackageLoading)?;

        result
    }

    fn handle_single_package_mode(
        &mut self,
        mime_types: &ExtraMimeTypes,
        package_path: &Path,
    ) -> error_stack::Result<(), FilePackageError> {
        if !package_path.exists() {
            warn!(
                "Static file hosting package does not exist at location {}",
                package_path.display()
            );
            return Ok(());
        }

        self.handle_package(mime_types, package_path, true)
    }

    fn handle_package_dir_mode(
        &mut self,
        mime_types: &ExtraMimeTypes,
        package_dir: &Path,
    ) -> error_stack::Result<(), FilePackageError> {
        if !package_dir.exists() {
            warn!(
                "Static file hosting package dir does not exist at location {}",
                package_dir.display()
            );
            return Ok(());
        }

        let dir =
            std::fs::read_dir(package_dir).change_context(FilePackageError::PackageLoading)?;
        let manager = PackageDirManager::new(dir)?;

        let mut latest_package = true;
        for (_, package_path) in manager.sorted_packages() {
            self.handle_package(mime_types, &package_path, latest_package)?;
            latest_package = false;
        }

        Ok(())
    }

    fn handle_package(
        &mut self,
        mime_types: &ExtraMimeTypes,
        package_path: &Path,
        find_index_html: bool,
    ) -> error_stack::Result<(), FilePackageError> {
        if !package_path.to_string_lossy().ends_with(".tar.gz") {
            return Err(FilePackageError::PackageLoading.report())
                .attach_printable("File name does not end with '.tar.gz'")
                .attach_printable(package_path.display().to_string());
        }
        let file = File::open(package_path).change_context(FilePackageError::PackageLoading)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        let entries = archive
            .entries()
            .change_context(FilePackageError::PackageLoading)?;
        let mut index_html_detected = false;
        for e in entries {
            let mut e = e.change_context(FilePackageError::PackageLoading)?;
            let Some(path_string) = Self::get_path_string(&e)? else {
                continue;
            };
            let mut data = vec![];
            e.read_to_end(&mut data)
                .change_context(FilePackageError::PackageLoading)?;
            let (path_string, static_file) = StaticFile::new(mime_types, path_string, data)?;
            if path_string.ends_with("/index.html") {
                if index_html_detected {
                    return Err(FilePackageError::MultipleIndexHtmlFiles.report())
                        .attach_printable(package_path.display().to_string());
                } else {
                    index_html_detected = true;
                    if find_index_html {
                        self.index_html = Some(static_file.clone());
                    }
                }
            }
            self.file_path_and_data.insert(path_string, static_file);
        }

        Ok(())
    }

    fn get_path_string(
        e: &tar::Entry<GzDecoder<File>>,
    ) -> error_stack::Result<Option<String>, FilePackageError> {
        if e.header().entry_type() == EntryType::Directory {
            return Ok(None);
        }
        let path = e.path().change_context(FilePackageError::PackageLoading)?;
        // Skip hidden files
        if path
            .file_name()
            .and_then(|v| v.to_str())
            .map(|v| v.starts_with('.'))
            .unwrap_or_default()
        {
            return Ok(None);
        }
        let path_string = path
            .to_str()
            .ok_or(FilePackageError::InvalidUtf8)?
            .to_string();
        Ok(Some(path_string))
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

    pub fn static_file(&self, path: &str) -> Option<StaticFile> {
        self.file_path_and_data.get(path).cloned()
    }

    pub fn index_html(&self) -> Option<StaticFile> {
        self.index_html.clone()
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

struct PackageDirManager {
    packages: HashMap<VersionNumber, PathBuf>,
}

impl PackageDirManager {
    fn new(dir: ReadDir) -> error_stack::Result<Self, FilePackageError> {
        let regex = Regex::new(r"v\d+\.\d+\.\d+").unwrap();

        let mut packages = HashMap::new();
        for d in dir {
            let d = d.change_context(FilePackageError::PackageLoading)?;
            if !d.path().is_file() {
                continue;
            }
            let name = d.file_name();
            let name = name.to_string_lossy();
            let Some(version) = regex
                .find(&name)
                .map(|m| m.as_str().trim_start_matches('v').to_string())
            else {
                warn!("{name} does not contain version number like v0.0.0");
                continue;
            };
            let version = TryInto::<VersionNumber>::try_into(version)
                .into_error_string(FilePackageError::PackageLoading)?;
            packages.insert(version, d.path());
        }

        Ok(Self { packages })
    }

    /// The first package is the latest package
    fn sorted_packages(self) -> Vec<(VersionNumber, PathBuf)> {
        let mut packages = self.packages.into_iter().collect::<Vec<_>>();
        packages.sort_by(|a, b| b.0.cmp(&a.0));
        packages
    }
}
