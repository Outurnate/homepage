use chrono::{DateTime, Utc};
use chrono_tz::{Canada, Tz};
use color_eyre::eyre::Result;
use itertools::Itertools;
use std::{
    fmt::Display,
    fs::read_to_string,
    path::{Path, PathBuf, StripPrefixError},
};

use crate::{markdown::Metadata, util::FontBook};

pub struct SiteConfiguration {
    root_content_path: PathBuf,
    root_output_path: PathBuf,
    root_url: String,
    d2: PathBuf,
    fontbook: FontBook,
}

impl SiteConfiguration {
    pub fn new(
        root_content_path: PathBuf,
        root_output_path: PathBuf,
        root_url: String,
        d2: PathBuf,
        fontbook: FontBook,
    ) -> Self {
        Self {
            root_content_path,
            root_output_path,
            root_url,
            d2,
            fontbook,
        }
    }

    pub fn get_d2_executable(&self) -> &Path {
        &self.d2
    }

    pub fn get_site_url(&self) -> &str {
        &self.root_url
    }

    pub fn get_content_root(&self) -> &Path {
        &self.root_content_path
    }

    pub fn get_output_root(&self) -> &Path {
        &self.root_output_path
    }

    pub fn resolve_relative_url(&self, s: &str) -> String {
        format!("{}{}", self.root_url, s)
    }

    pub fn get_fontbook(&self) -> &FontBook {
        &self.fontbook
    }
}

pub struct ContentReference {
    content_file_path: PathBuf,
    output_file_path: PathBuf,
}

impl ContentReference {
    pub fn new(
        site_config: &SiteConfiguration,
        content_file_full_path: PathBuf,
    ) -> Result<Self, StripPrefixError> {
        let output_file_path = content_file_full_path
            .strip_prefix(&site_config.root_content_path)?
            .to_path_buf();

        Ok(Self {
            content_file_path: output_file_path.clone(),
            output_file_path,
        })
    }

    pub fn get_content_extension(&self) -> Option<&str> {
        self.content_file_path.extension().and_then(|x| x.to_str())
    }

    pub fn set_output_extension(&mut self, ext: &str) {
        self.output_file_path.set_extension(ext);
    }

    pub fn get_relative_url(&self) -> String {
        self.output_file_path.to_string_lossy().to_string()
    }

    pub fn get_full_url(&self, site_config: &SiteConfiguration) -> String {
        format!(
            "{}{}",
            site_config.root_url,
            self.output_file_path.to_string_lossy()
        )
    }

    pub fn get_slug(&self) -> String {
        self.content_file_path
            .file_stem()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn resolve_relative_path(
        &self,
        site_config: &SiteConfiguration,
        p: impl AsRef<Path>,
    ) -> PathBuf {
        site_config
            .root_output_path
            .join(self.output_file_path.parent().expect(
                "output_file_path should always contain a file path, so parent is always findable",
            ))
            .join(p)
    }

    pub fn process_with_output<F, R>(
        &mut self,
        site_config: &SiteConfiguration,
        processor: F,
    ) -> Result<R>
    where
        F: FnOnce(String, &SiteConfiguration, &Self) -> Result<(String, R)>,
    {
        let input = read_to_string(site_config.root_content_path.join(&self.content_file_path))?;
        let (output, result) = processor(input, site_config, self)?;
        std::fs::write(
            site_config.root_output_path.join(&self.output_file_path),
            output,
        )?;
        Ok(result)
    }

    pub fn process<F>(&mut self, site_config: &SiteConfiguration, processor: F) -> Result<()>
    where
        F: FnOnce(String, &SiteConfiguration, &Self) -> Result<String>,
    {
        self.process_with_output(site_config, |source, site_config, content_reference| {
            Ok((processor(source, site_config, content_reference)?, ()))
        })
    }

    pub fn copy(&mut self, site_config: &SiteConfiguration) -> Result<()> {
        std::fs::copy(
            site_config.root_content_path.join(&self.content_file_path),
            site_config.root_output_path.join(&self.output_file_path),
        )?;
        Ok(())
    }
}

impl Display for ContentReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .content_file_path
                .as_path()
                .as_os_str()
                .to_string_lossy(),
        )
    }
}

pub struct Sitemap {
    entries: Vec<(ContentReference, String, Metadata)>,
    buildstamp: DateTime<Tz>,
}

impl Sitemap {
    pub fn new(entries: Vec<(ContentReference, String, Metadata)>) -> Self {
        Self {
            entries,
            buildstamp: Utc::now().with_timezone(&Canada::Eastern),
        }
    }

    pub fn get_article_entries<'a>(
        &'a self,
        site_config: &SiteConfiguration,
    ) -> impl Iterator<Item = ArticleEntry<'a>> {
        self.entries
            .iter()
            .filter_map(|(content_reference, title, metadata)| {
                if let Metadata::Article {
                    date,
                    modified: _,
                    category,
                } = metadata
                {
                    Some(ArticleEntry {
                        title,
                        relative_url: content_reference.get_relative_url(),
                        listing_url: format!(
                            "/images/listing_{}.jpeg",
                            content_reference.get_slug()
                        ),
                        full_url: content_reference.get_full_url(site_config),
                        date,
                        category,
                    })
                } else {
                    None
                }
            })
            .sorted_by(|x, y| Ord::cmp(&y.date, &x.date))
    }

    pub fn get_map_entries(
        &self,
        site_config: &SiteConfiguration,
    ) -> impl Iterator<Item = MapEntry> {
        self.entries
            .iter()
            .map(|(content_reference, _title, metadata)| match metadata {
                Metadata::Article {
                    date,
                    modified,
                    category: _,
                } => MapEntry {
                    location: content_reference.get_full_url(site_config),
                    last_modified: modified.unwrap_or(*date).to_owned(),
                    change_frequency: String::from("monthly"),
                },
                Metadata::Page { description: _ } => MapEntry {
                    location: content_reference.get_full_url(site_config),
                    last_modified: self.buildstamp,
                    change_frequency: String::from("yearly"),
                },
            })
    }

    pub fn get_buildstamp(&self) -> &DateTime<Tz> {
        &self.buildstamp
    }
}

pub struct MapEntry {
    pub location: String,
    pub last_modified: DateTime<Tz>,
    pub change_frequency: String,
}

pub struct ArticleEntry<'a> {
    pub title: &'a str,
    pub relative_url: String,
    pub listing_url: String,
    pub full_url: String,
    pub date: &'a DateTime<Tz>,
    pub category: &'a String,
}
