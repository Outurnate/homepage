use std::{
    fs::{File, OpenOptions},
    io::{Cursor, Read},
    num::NonZero,
};

use color_eyre::eyre::Result;
use image::{
    ExtendedColorType, ImageFormat,
    codecs::ico::{IcoEncoder, IcoFrame},
};
use oxipng::{Deflaters, optimize_from_memory};

use crate::{content::SiteConfiguration, util::render_svg};

pub fn render_favicon(config: &SiteConfiguration) -> Result<()> {
    let data = {
        let mut data = Vec::new();
        File::open(config.get_content_root().join("favicon.svg"))?.read_to_end(&mut data)?;
        data
    };

    let sizes = [16, 32, 64, 128];
    let frames = sizes
        .into_iter()
        .map(|size| -> Result<IcoFrame<'_>> {
            let encoded_unopt = {
                let image = render_svg(config.get_fontbook(), &data, |_, _| (size, size), None)?;
                let mut buffer = Cursor::new(Vec::new());
                image.write_to(&mut buffer, ImageFormat::Png)?;
                buffer.into_inner()
            };
            let encoded_image = optimize_from_memory(
                &encoded_unopt,
                &oxipng::Options {
                    deflate: Deflaters::Zopfli {
                        iterations: NonZero::new(15).expect("15 is not 0"),
                    },
                    ..Default::default()
                },
            )?;
            Ok(IcoFrame::with_encoded(
                encoded_image,
                size,
                size,
                ExtendedColorType::Rgba8,
            )?)
        })
        .collect::<Result<Vec<_>>>()?;
    IcoEncoder::new(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(config.get_output_root().join("favicon.ico"))?,
    )
    .encode_images(&frames)?;
    Ok(())
}
