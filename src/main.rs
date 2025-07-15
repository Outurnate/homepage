#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

mod content;
mod diagrams;
mod favicon;
mod markdown;
mod templates;
mod util;

use clap::Parser;
use color_eyre::eyre::Result;
use content::{ContentReference, SiteConfiguration, Sitemap};
use diagrams::compile_d2;
use favicon::render_favicon;
use markdown::Metadata;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs::create_dir, path::PathBuf};
use templates::{
    output_archive, output_custom1, output_custom2, output_index, output_rss, output_sitemap,
    render,
};
use tracing::{info, warn};
use util::FontBook;
use walkdir::WalkDir;

#[tracing::instrument(skip_all, fields(src = content.to_string()))]
fn process_content_first_pass(
    site_config: &SiteConfiguration,
    content: &mut ContentReference,
) -> Result<()> {
    match content.get_content_extension() {
        Some("md") => {
            info!("deferring markdown on first pass");
            Ok(())
        }
        Some("d2") => {
            info!("rendering d2 diagram");
            content.set_output_extension("svg");
            content.process(site_config, |source, _, _| {
                compile_d2(site_config, source.as_bytes())
            })
        }
        _ => {
            info!("copying misc file");
            content.copy(site_config)
        }
    }
}

#[tracing::instrument(skip_all, fields(src = content.to_string()))]
fn process_content_second_pass(
    site_config: &SiteConfiguration,
    mut content: ContentReference,
) -> Result<Option<(ContentReference, String, Metadata)>> {
    if content.get_content_extension() == Some("md") {
        info!("rendering markdown");
        content.set_output_extension("html");
        let document = content.process_with_output(site_config, render)?;
        Ok(Some((content, document.title, document.metadata)))
    } else {
        Ok(None)
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    content: PathBuf,

    #[arg(long)]
    dist: PathBuf,

    #[arg(long)]
    url: String,

    #[arg(long)]
    d2: PathBuf,

    #[arg(long)]
    font_regular: PathBuf,

    #[arg(long)]
    font_italic: PathBuf,

    #[arg(long)]
    font_bold: PathBuf,

    #[arg(long)]
    font_semibold: PathBuf,

    #[arg(long)]
    font_monospace: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let config = SiteConfiguration::new(
        args.content.canonicalize()?,
        args.dist.canonicalize()?,
        args.url,
        args.d2,
        FontBook {
            regular: args.font_regular,
            italic: args.font_italic,
            bold: args.font_bold,
            semibold: args.font_semibold,
            monospace: args.font_monospace,
        },
    );

    let mut contents: Vec<_> = WalkDir::new(config.get_content_root())
        .into_iter()
        .map(|entry| -> Result<Option<ContentReference>> {
            let entry = entry?;
            if entry.file_type().is_dir() {
                let destination_path = config
                    .get_output_root()
                    .join(entry.path().strip_prefix(config.get_content_root())?);
                if !destination_path.exists() {
                    create_dir(destination_path)?;
                }
                Ok(None)
            } else if entry.file_type().is_file() {
                Ok(Some(ContentReference::new(
                    &config,
                    entry.path().to_path_buf(),
                )?))
            } else {
                Ok(None)
            }
        })
        .filter_map(|x| x.transpose())
        .collect::<Result<Vec<_>, _>>()?;

    for content in &mut contents {
        process_content_first_pass(&config, content)?;
    }

    let sitemap_entries = contents
        .into_par_iter()
        .map(|content| process_content_second_pass(&config, content))
        .filter_map(|x| x.transpose())
        .collect::<Result<Vec<_>, _>>()?;

    let sitemap = Sitemap::new(sitemap_entries);

    std::fs::write(
        config.get_output_root().join("index.html"),
        output_index(&config, &sitemap)?,
    )?;
    std::fs::write(
        config.get_output_root().join("archives.html"),
        output_archive(&config, &sitemap)?,
    )?;
    std::fs::write(
        config.get_output_root().join("feeds/feed.xml"),
        output_rss(&config, &sitemap)?,
    )?;
    std::fs::write(
        config.get_output_root().join("sitemap.xml"),
        output_sitemap(&config, &sitemap)?,
    )?;
    std::fs::write(
        config
            .get_output_root()
            .join("simple-markov-generator.html"),
        output_custom1(&config)?,
    )?;
    std::fs::write(
        config
            .get_output_root()
            .join("password-strength-checker.html"),
        output_custom2(&config)?,
    )?;

    render_favicon(&config)?;

    Ok(())
}
