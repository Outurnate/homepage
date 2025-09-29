use std::io::Cursor;

use askama::Template;
use chrono::{DateTime, Datelike};
use chrono_tz::Tz;
use color_eyre::eyre::{Context, Result};
use image::{DynamicImage, ImageReader, Rgb, RgbImage, buffer::ConvertBuffer, imageops};
use ntscrs::ntsc::{
    FbmNoiseSettings, NtscEffect, TrackingNoiseSettings, VHSEdgeWaveSettings, VHSSettings,
    VHSTapeSpeed,
};
use ntscrs::yiq_fielding::Rgb8;
use ordinal::ToOrdinal;
use tracing::info;

use crate::{
    ContentReference,
    content::ArticleEntry,
    markdown::{ListingImage, MarkdownDocument, Metadata},
    util::render_svg,
};
use crate::{SiteConfiguration, content::Sitemap};

fn human_date(date: &DateTime<Tz>) -> String {
    format!(
        "{} {}{}, {}",
        date.format("%A %B"),
        date.day(),
        date.day().suffix(),
        date.format("%Y")
    )
}

fn iso_date(date: &DateTime<Tz>) -> String {
    format!("{}", date.format("%+"))
}

#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    config: &'a SiteConfiguration,
    content_reference: &'a ContentReference,
    document: &'a MarkdownDocument,
    opengraph_image_url: String,
}

fn render_og_bg(
    content_index: &SiteConfiguration,
    content_reference: &ContentReference,
    document: &MarkdownDocument,
) -> Result<RgbImage> {
    let width = 640;
    let height = 480;
    match &document.listing_image {
        Some(ListingImage::ImageUrl(image_url)) => {
            info!("selected {image_url:?} as opengraph image");
            let image_url = content_reference.resolve_relative_path(content_index, image_url);
            let bytes = std::fs::read(&image_url)
                .wrap_err(format!("Error opening {}", image_url.to_string_lossy()))?;
            let image = match image_url.extension().and_then(|ext| ext.to_str()) {
                Some("svg") => DynamicImage::ImageRgba8(render_svg(
                    content_index.get_fontbook(),
                    &bytes,
                    |original_width, original_height| {
                        (width, (width * original_height) / original_width)
                    },
                    None,
                )?),
                Some(_) | None => ImageReader::new(Cursor::new(bytes))
                    .with_guessed_format()?
                    .decode()?,
            };
            Ok(image
                .resize_to_fill(width, height, imageops::FilterType::Lanczos3)
                .into_rgb8())
        }
        Some(ListingImage::CodeBlock(code)) => Ok(render_svg(
            content_index.get_fontbook(),
            code.as_bytes(),
            |_, _| (width, height),
            None,
        )?
        .convert()),
        _ => Ok(RgbImage::from_pixel(width, height, Rgb::from([0, 0, 0]))),
    }
}

fn render_og_bar(content_index: &SiteConfiguration) -> Result<RgbImage> {
    Ok(render_svg(
        content_index.get_fontbook(),
        include_bytes!("og-image.svg"),
        |_width, _height| (640, 103),
        Some(content_index.get_content_root().to_path_buf()),
    )?
    .convert())
}

fn apply_effect(image: RgbImage, wiggle: usize) -> RgbImage {
    let width = image.width();
    let height = image.height();
    let mut raw_image = image.into_raw();
    let effect = {
        let mut effect = NtscEffect::default();
        effect.snow_intensity = 0.6;
        effect.snow_anisotropy = 0.6;
        effect.luma_smear = 0.75;
        effect.tracking_noise = Some(TrackingNoiseSettings {
            height: 12,
            wave_intensity: 15.0,
            snow_intensity: 0.025,
            snow_anisotropy: 0.25,
            noise_intensity: 0.25,
        });
        effect.chroma_noise = Some(FbmNoiseSettings {
            intensity: 0.5,
            frequency: 0.05,
            detail: 2,
        });
        effect.vhs_settings = Some(VHSSettings {
            tape_speed: VHSTapeSpeed::LP,
            chroma_loss: 0.0025,
            sharpen: None,
            edge_wave: Some(VHSEdgeWaveSettings {
                intensity: 2.5,
                speed: 10.0,
                frequency: 0.05,
                detail: 2,
            }),
        });
        effect
    };
    effect.apply_effect_to_buffer::<Rgb8>(
        (width as usize, height as usize),
        &mut raw_image,
        wiggle,
        [1.0, 1.0],
    );
    RgbImage::from_raw(width, height, raw_image)
        .expect("reconstructing image from original size - infallible")
}

fn render_og(
    content_index: &SiteConfiguration,
    content_reference: &ContentReference,
    document: &MarkdownDocument,
) -> Result<(RgbImage, RgbImage)> {
    let mut og = render_og_bg(content_index, content_reference, document)?;
    let bar = render_og_bar(content_index)?;
    let y = (og.height() - bar.height()) as i64;
    imageops::overlay(&mut og, &bar, 0, y);
    let vhs = apply_effect(og, document.title.as_bytes()[0] as usize);
    let mut letterbox = RgbImage::from_pixel(917, 480, Rgb::from([0, 0, 0]));
    imageops::overlay(&mut letterbox, &vhs, (917 - 640) / 2, 0);
    Ok((vhs, letterbox))
}

pub fn render(
    source: String,
    config: &SiteConfiguration,
    content_reference: &ContentReference,
) -> Result<(String, MarkdownDocument)> {
    let document = MarkdownDocument::new(source)?;
    let opengraph_image_url = format!("images/opengraph_{}.jpeg", content_reference.get_slug());
    let (listing_image, opengraph_image) = render_og(config, content_reference, &document)?;
    opengraph_image.save(content_reference.resolve_relative_path(config, &opengraph_image_url))?;
    listing_image.save(content_reference.resolve_relative_path(
        config,
        format!("images/listing_{}.jpeg", content_reference.get_slug()),
    ))?;
    Ok((
        PageTemplate {
            config,
            content_reference,
            document: &document,
            opengraph_image_url: config.resolve_relative_url(&opengraph_image_url),
        }
        .render()?,
        document,
    ))
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage<'a> {
    config: &'a SiteConfiguration,
    latest_article: Option<ArticleEntry<'a>>,
}

pub fn output_index(config: &SiteConfiguration, sitemap: &Sitemap) -> Result<String> {
    Ok(IndexPage {
        config,
        latest_article: sitemap.get_article_entries(config).next(),
    }
    .render()?)
}

#[derive(Template)]
#[template(path = "archives.html")]
struct ArchivePage<'a> {
    config: &'a SiteConfiguration,
    sitemap: &'a Sitemap,
}

pub fn output_archive(config: &SiteConfiguration, sitemap: &Sitemap) -> Result<String> {
    Ok(ArchivePage { config, sitemap }.render()?)
}

#[derive(Template)]
#[template(path = "feed.xml")]
struct RssPage<'a> {
    config: &'a SiteConfiguration,
    sitemap: &'a Sitemap,
}

pub fn output_rss(config: &SiteConfiguration, sitemap: &Sitemap) -> Result<String> {
    Ok(RssPage { config, sitemap }.render()?)
}

#[derive(Template)]
#[template(path = "sitemap.xml")]
struct SitemapPage<'a> {
    config: &'a SiteConfiguration,
    sitemap: &'a Sitemap,
}

pub fn output_sitemap(config: &SiteConfiguration, sitemap: &Sitemap) -> Result<String> {
    Ok(SitemapPage { config, sitemap }.render()?)
}

#[derive(Template)]
#[template(path = "simple-markov-generator.html")]
struct Custom1<'a> {
    config: &'a SiteConfiguration,
}

pub fn output_custom1(config: &SiteConfiguration) -> Result<String> {
    Ok(Custom1 { config }.render()?)
}

#[derive(Template)]
#[template(path = "password-strength-checker.html")]
struct Custom2<'a> {
    config: &'a SiteConfiguration,
}

pub fn output_custom2(config: &SiteConfiguration) -> Result<String> {
    Ok(Custom2 { config }.render()?)
}
