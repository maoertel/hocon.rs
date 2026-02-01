use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use crate::internals::HoconInternal;
use crate::parser;
use crate::Error;
use crate::Result;

#[derive(Debug, Clone)]
pub(crate) enum FileType {
    Properties,
    Hocon,
    Json,
    All,
}

#[derive(Default, Debug)]
pub(crate) struct FileRead {
    pub(crate) properties: Option<String>,
    pub(crate) json: Option<String>,
    pub(crate) hocon: Option<String>,
}
impl FileRead {
    fn from_file_type(ft: &FileType, s: String) -> Self {
        match ft {
            FileType::Properties => Self {
                properties: Some(s),
                ..Default::default()
            },
            FileType::Json => Self {
                json: Some(s),
                ..Default::default()
            },
            FileType::Hocon => Self {
                hocon: Some(s),
                ..Default::default()
            },
            FileType::All => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ConfFileMeta {
    path: PathBuf,
    full_path: PathBuf,
    file_type: FileType,
}
impl ConfFileMeta {
    pub(crate) fn from_path(path: PathBuf) -> Self {
        let file = path
            .file_name()
            .expect("got a path without a filename")
            .to_str()
            .expect("got invalid UTF-8 path");
        let mut parent_path = path.clone();
        parent_path.pop();

        Self {
            path: parent_path,
            full_path: path.clone(),
            file_type: match Path::new(file).extension().and_then(OsStr::to_str) {
                Some("properties") => FileType::Properties,
                Some("json") => FileType::Json,
                Some("conf") => FileType::Hocon,
                _ => FileType::All,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HoconLoaderConfig {
    pub(crate) include_depth: u8,
    pub(crate) file_meta: Option<ConfFileMeta>,
    pub(crate) system: bool,
    #[cfg(feature = "url-support")]
    pub(crate) external_url: bool,
    pub(crate) strict: bool,
    pub(crate) max_include_depth: u8,
}

impl Default for HoconLoaderConfig {
    fn default() -> Self {
        Self {
            include_depth: 0,
            file_meta: None,
            system: true,
            #[cfg(feature = "url-support")]
            external_url: true,
            strict: false,
            max_include_depth: 10,
        }
    }
}

impl HoconLoaderConfig {
    pub(crate) fn included_from(&self) -> Self {
        Self {
            include_depth: self.include_depth + 1,
            ..self.clone()
        }
    }

    pub(crate) fn with_file(&self, path: PathBuf) -> Self {
        match self.file_meta.as_ref() {
            Some(file_meta) => Self {
                file_meta: Some(ConfFileMeta::from_path(file_meta.clone().path.join(path))),
                ..self.clone()
            },
            None => Self {
                file_meta: Some(ConfFileMeta::from_path(path)),
                ..self.clone()
            },
        }
    }

    pub(crate) fn parse_str_to_internal(&self, s: FileRead) -> Result<HoconInternal> {
        let mut internal = HoconInternal::empty();
        if let Some(properties) = s.properties {
            internal = internal.add(
                java_properties::read(properties.as_bytes())
                    .map(HoconInternal::from_properties)
                    .map_err(|_| Error::Parse)?,
            );
        };
        if let Some(json) = s.json {
            let input = format!("{}\n\0", json.replace('\r', "\n"));
            internal = internal.add(
                parser::root(self)(&input)
                    .map_err(|_| Error::Parse)
                    .and_then(|(remaining, parsed)| {
                        if Self::remaining_only_whitespace(remaining) {
                            parsed
                        } else if self.strict {
                            Err(Error::Deserialization {
                                message: String::from("file could not be parsed completely"),
                            })
                        } else {
                            parsed
                        }
                    })?,
            );
        };
        if let Some(hocon) = s.hocon {
            let input = format!("{}\n\0", hocon.replace('\r', "\n"));
            internal = internal.add(
                parser::root(self)(&input)
                    .map_err(|_| Error::Parse)
                    .and_then(|(remaining, parsed)| {
                        if Self::remaining_only_whitespace(remaining) {
                            parsed
                        } else if self.strict {
                            Err(Error::Deserialization {
                                message: String::from("file could not be parsed completely"),
                            })
                        } else {
                            parsed
                        }
                    })?,
            );
        };

        Ok(internal)
    }

    fn remaining_only_whitespace(remaining: &str) -> bool {
        remaining
            .chars()
            .all(|c| c == '\n' || c == '\r' || c == '\0')
    }

    pub(crate) fn read_file_to_string(path: PathBuf) -> Result<String> {
        let mut file = File::open(path.as_os_str())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub(crate) fn read_file(&self) -> Result<FileRead> {
        let full_path = self
            .file_meta
            .clone()
            .expect("missing file metadata")
            .full_path;
        match self.file_meta.as_ref().map(|fm| &fm.file_type) {
            Some(FileType::All) => Ok(FileRead {
                hocon: Self::read_file_to_string({
                    let mut path = full_path.clone();
                    if !path.exists() {
                        path.set_extension("conf");
                    }
                    path
                })
                .ok(),
                json: Self::read_file_to_string({
                    let mut path = full_path.clone();
                    path.set_extension("json");
                    path
                })
                .ok(),
                properties: Self::read_file_to_string({
                    let mut path = full_path;
                    path.set_extension("properties");
                    path
                })
                .ok(),
            }),
            Some(ft) => Ok(FileRead::from_file_type(
                ft,
                Self::read_file_to_string(full_path)?,
            )),
            _ => unimplemented!(),
        }
    }

    #[cfg(feature = "url-support")]
    pub(crate) fn load_url(&self, url: &str) -> Result<HoconInternal> {
        if let Ok(parsed_url) = reqwest::Url::parse(url) {
            if parsed_url.scheme() == "file" {
                if let Ok(path) = parsed_url.to_file_path() {
                    let include_config = self.included_from().with_file(path);
                    let s = include_config.read_file()?;
                    Ok(include_config
                        .parse_str_to_internal(s)
                        .map_err(|_| Error::Include {
                            path: String::from(url),
                        })?)
                } else {
                    Err(Error::Include {
                        path: String::from(url),
                    })
                }
            } else if self.external_url {
                let body = reqwest::blocking::get(parsed_url)
                    .and_then(reqwest::blocking::Response::text)
                    .map_err(|_| Error::Include {
                        path: String::from(url),
                    })?;

                Ok(self.parse_str_to_internal(FileRead {
                    hocon: Some(body),
                    ..Default::default()
                })?)
            } else {
                Err(Error::Include {
                    path: String::from(url),
                })
            }
        } else {
            Err(Error::Include {
                path: String::from(url),
            })
        }
    }
}
