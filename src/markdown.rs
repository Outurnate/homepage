use std::{collections::HashMap, sync::LazyLock};

use chrono::{DateTime, NaiveDate};
use chrono_tz::Tz;
use color_eyre::eyre::{ContextCompat, Result};
use comrak::{
    Arena, ExtensionOptions, Options, ParseOptions, Plugins, RenderOptions, RenderPlugins,
    adapters::SyntaxHighlighterAdapter, format_html_with_plugins, html, nodes::NodeValue,
    parse_document,
};
use mathemascii::render_mathml;
use regex::{Captures, Regex, RegexBuilder};
use syntect::{
    dumps::from_uncompressed_data,
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};
use tracing::warn;

pub enum Metadata {
    Article {
        date: DateTime<Tz>,
        modified: Option<DateTime<Tz>>,
        category: String,
    },
    Page {
        description: String,
    },
}

pub enum ListingImage {
    ImageUrl(String),
    CodeBlock(String),
}

pub struct MarkdownDocument {
    pub content: String,
    pub title: String,
    pub listing_image: Option<ListingImage>,
    pub metadata: Metadata,
}

static SS: LazyLock<SyntaxSet> = LazyLock::new(|| {
    from_uncompressed_data(include_bytes!("../syntaxes/newlines.packdump"))
        .expect("pre-compiled packdump is invalid")
});

fn syntax_highligh_svg(syntax: &SyntaxReference, input: &str) -> Result<String> {
    let mut lines = format!(
        r#"<svg width="640" height="480" viewBox="0 0 640 480" xmlns="http://www.w3.org/2000/svg"><style>{}</style>"#,
        include_str!("syntax_highlighting.css")
    );
    let padding = 5;
    for (i, line) in LinesWithEndings::from(input).enumerate() {
        let mut html_generator =
            ClassedHTMLGenerator::new_with_class_style(syntax, &SS, ClassStyle::Spaced);
        html_generator.parse_html_for_line_which_includes_newline(line)?;
        lines.push_str(&format!(
            r#"<text class="source" x="{}" y="{}">"#,
            padding,
            padding + ((i + 1) * 45)
        ));
        lines.push_str(
            &html_generator
                .finalize()
                .replace("<span", r#"<tspan xml:space="preserve""#)
                .replace("</span", r#"</tspan"#),
        );
        lines.push_str("</text>\n");
    }
    lines.push_str("</svg>");
    Ok(lines)
}

fn syntax_highligh_html(syntax: &SyntaxReference, input: &str) -> Result<String> {
    let mut html_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, &SS, ClassStyle::Spaced);
    for line in LinesWithEndings::from(input) {
        html_generator.parse_html_for_line_which_includes_newline(line)?;
    }
    Ok(html_generator.finalize())
}

enum OutputFormat {
    Html,
    Svg,
}

fn syntax_highlight_safe(token: Option<&str>, input: &str, format: OutputFormat) -> String {
    let syntax = token
        .and_then(|token| SS.find_syntax_by_token(token))
        .or_else(|| SS.find_syntax_by_first_line(input));
    if let Some(syntax) = syntax {
        match match format {
            OutputFormat::Html => syntax_highligh_html(syntax, input),
            OutputFormat::Svg => syntax_highligh_svg(syntax, input),
        } {
            Ok(output) => output,
            Err(err) => {
                warn!("syntax highlighting fault: {err}; sending plain text");
                input.to_owned()
            }
        }
    } else {
        if let Some(token) = token
            && !token.is_empty()
        {
            warn!("syntax {token} not found");
        }
        input.to_owned()
    }
}

struct SyntectAdapter;

impl SyntaxHighlighterAdapter for SyntectAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn std::io::Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::io::Result<()> {
        output.write_all(syntax_highlight_safe(lang, code, OutputFormat::Html).as_bytes())
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn std::io::Write,
        attributes: std::collections::HashMap<String, String>,
    ) -> std::io::Result<()> {
        html::write_opening_tag(output, "pre", attributes)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn std::io::Write,
        attributes: std::collections::HashMap<String, String>,
    ) -> std::io::Result<()> {
        html::write_opening_tag(output, "code", attributes)
    }
}

static BLOCK_MATH: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\$\$\n(?P<expr>[\w\W]+)\n\$\$")
        .multi_line(true)
        .build()
        .expect("Unreachable - will only panic if the regex is invalid")
});

fn render_markdown_to_html(md: &str) -> Result<(String, Option<ListingImage>)> {
    let md = BLOCK_MATH.replace(md, |caps: &Captures| {
        render_mathml(mathemascii::parse(
            caps.name("expr")
                .expect(
                    "Unreachable - will only panic if regex doesn't contain a group called 'expr'",
                )
                .as_str(),
        ))
    });

    let arena = Arena::new();

    let extension = ExtensionOptions {
        strikethrough: true,
        tagfilter: false,
        table: true,
        autolink: false,
        tasklist: false,
        superscript: true,
        footnotes: true,
        description_lists: true,
        front_matter_delimiter: None,
        multiline_block_quotes: false,
        alerts: true,
        math_dollars: false,
        math_code: false,
        wikilinks_title_after_pipe: false,
        wikilinks_title_before_pipe: false,
        underline: true,
        subscript: true,
        spoiler: false,
        greentext: false,
        ..Default::default()
    };
    let parse = ParseOptions {
        smart: true,
        ..Default::default()
    };
    let render = RenderOptions {
        unsafe_: true,
        sourcepos: false,
        figure_with_caption: true,
        ..Default::default()
    };
    let options = Options {
        extension,
        parse,
        render,
    };
    let render = RenderPlugins {
        codefence_syntax_highlighter: Some(&SyntectAdapter {}),
        ..Default::default()
    };
    let plugins = Plugins { render };

    let doc = parse_document(&arena, &md, &options);

    let first_image_url = doc
        .descendants()
        .filter_map(|node| match &node.data.borrow().value {
            NodeValue::Image(link) => Some(ListingImage::ImageUrl(link.url.to_string())),
            _ => None,
        })
        .next();

    let first_codeblock = doc
        .descendants()
        .filter_map(|node| match &node.data.borrow().value {
            NodeValue::CodeBlock(code) => Some(ListingImage::CodeBlock(syntax_highlight_safe(
                Some(&code.info),
                &code.literal,
                OutputFormat::Svg,
            ))),
            _ => None,
        })
        .next();

    let listing_image = first_image_url.or(first_codeblock);

    let mut html = vec![];
    format_html_with_plugins(doc, &options, &mut html, &plugins)?;

    Ok((String::from_utf8(html)?, listing_image))
}

fn sloppy_date_parser(s: &str) -> Option<DateTime<Tz>> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .and_then(|d| d.and_local_timezone(chrono_tz::Canada::Eastern).earliest())
}

impl MarkdownDocument {
    pub fn new(content: String) -> Result<Self> {
        let (frontmatter, md) = content
            .split_once("\n\n")
            .wrap_err("Document missing frontmatter")?;
        let properties: HashMap<_, _> = frontmatter
            .lines()
            .filter_map(|line| {
                line.split_once(":")
                    .map(|(key, value)| (key.trim().to_lowercase(), value.trim()))
            })
            .collect();
        let (content, image_urls) = render_markdown_to_html(md)?;
        let title = properties
            .get("title")
            .wrap_err("Document missing title")?
            .to_string();
        let metadata = if let Some(date) =
            properties.get("date").and_then(|d| sloppy_date_parser(d))
            && let Some(category) = properties.get("category").map(|x| x.to_string())
        {
            Metadata::Article {
                date,
                modified: properties
                    .get("modified")
                    .and_then(|d| sloppy_date_parser(d)),
                category,
            }
        } else if let Some(description) = properties.get("description").map(|x| x.to_string()) {
            Metadata::Page { description }
        } else {
            panic!("too much missing metadata");
        };
        Ok(Self {
            content,
            title,
            listing_image: image_urls,
            metadata,
        })
    }
}
