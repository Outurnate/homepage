use color_eyre::{
    Result,
    eyre::{OptionExt, eyre},
};
use regex::{Captures, Regex};
use std::{
    ffi::{OsStr, OsString},
    io::{Cursor, Read, Write},
    process::{Command, Stdio},
    sync::LazyLock,
};
use tracing::{error, info};
use xmltree::{Element, XMLNode};

use crate::content::SiteConfiguration;

static PATCH_FONT_STYLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""?d2-\d+-font-(?P<style>regular|bold|italic|semibold)"?;"#)
        .expect("compile time regex invalid")
});

fn patch_css(source: &str) -> String {
    let source = PATCH_FONT_STYLE.replace_all(source, |captures: &Captures<'_>| {
        match captures.name("style").map(|m| m.as_str()) {
            Some("regular") => "Fira Sans;",
            Some("bold") => "Fira Sans;font-weight:bold;",
            Some("italic") => "Fira Sans;font-style:italic;",
            Some("semibold") => "Fira Sans;font-weight:600;",
            None | Some(_) => todo!(),
        }
    });

    source.to_string()
}

fn walk_to_styles(node: &mut XMLNode) -> Result<()> {
    if let XMLNode::Element(element) = node {
        if element.matches("style") {
            let new_css = patch_css(&element.get_text().unwrap_or_default());
            element.children.clear();
            element.children.push(XMLNode::Text(new_css));
        } else {
            for child in &mut element.children {
                walk_to_styles(child)?;
            }
        }
    }
    Ok(())
}

fn postprocess_svg_css<R, W>(input: R, output: W) -> Result<()>
where
    R: Read,
    W: Write,
{
    let mut root = Element::parse(input)?;
    for node in &mut root.children {
        walk_to_styles(node)?;
    }
    root.write(output)?;
    Ok(())
}

pub fn compile_d2(config: &SiteConfiguration, source: &[u8]) -> Result<String> {
    let mut d2 = Command::new(config.get_d2_executable())
        .arg("--theme=1")
        .arg("--dark-theme=1")
        .arg(OsString::from_iter([
            OsStr::new("--font-regular="),
            config.get_fontbook().regular.as_os_str(),
        ]))
        .arg(OsString::from_iter([
            OsStr::new("--font-italic="),
            config.get_fontbook().italic.as_os_str(),
        ]))
        .arg(OsString::from_iter([
            OsStr::new("--font-bold="),
            config.get_fontbook().bold.as_os_str(),
        ]))
        .arg(OsString::from_iter([
            OsStr::new("--font-semibold="),
            config.get_fontbook().semibold.as_os_str(),
        ]))
        .arg("--pad=0")
        .arg("-")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let mut stdin = d2
            .stdin
            .take()
            .ok_or_eyre("d2 closed stdin before we could open it")?;
        stdin.write_all(include_bytes!("preamble.d2"))?;
        stdin.write_all(source)?;
    }
    let output = d2.wait_with_output()?;
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        match line.split_once(":") {
            Some(("info", msg)) => info!("{}", msg.trim()),
            Some(("err", msg)) => error!("{}", msg.trim()),
            Some((_, _)) | None => info!("{}", line.trim()),
        }
    }
    if output.status.success() {
        let mut destination = Vec::new();
        postprocess_svg_css(Cursor::new(output.stdout), &mut destination)?;
        Ok(String::from_utf8(destination)?)
    } else {
        Err(eyre!("d2 exit status indicated failure"))
    }
}
