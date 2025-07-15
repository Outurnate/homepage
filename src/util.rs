use std::{fs::File, io::Read, path::PathBuf};

use color_eyre::eyre::{ContextCompat, Result};
use image::RgbaImage;
use resvg::{
    tiny_skia::Pixmap,
    usvg::{self, Transform, fontdb},
};

pub struct FontBook {
    pub regular: PathBuf,
    pub italic: PathBuf,
    pub bold: PathBuf,
    pub semibold: PathBuf,
    pub monospace: PathBuf,
}

impl FontBook {
    pub fn inject_fonts(&self, fontdb: &mut fontdb::Database) -> Result<()> {
        let mut regular = Vec::new();
        File::open(&self.regular)?.read_to_end(&mut regular)?;
        fontdb.load_font_data(regular);
        let mut italic = Vec::new();
        File::open(&self.italic)?.read_to_end(&mut italic)?;
        fontdb.load_font_data(italic);
        let mut bold = Vec::new();
        File::open(&self.bold)?.read_to_end(&mut bold)?;
        fontdb.load_font_data(bold);
        let mut semibold = Vec::new();
        File::open(&self.semibold)?.read_to_end(&mut semibold)?;
        fontdb.load_font_data(semibold);
        let mut monospace = Vec::new();
        File::open(&self.monospace)?.read_to_end(&mut monospace)?;
        fontdb.load_font_data(monospace);
        Ok(())
    }
}

pub fn render_svg(
    fontbook: &FontBook,
    data: &[u8],
    size: impl FnOnce(u32, u32) -> (u32, u32),
    resources_dir: Option<PathBuf>,
) -> Result<RgbaImage> {
    let tree = {
        let mut opt = usvg::Options {
            resources_dir,
            ..Default::default()
        };
        fontbook.inject_fonts(opt.fontdb_mut())?;
        usvg::Tree::from_data(data, &opt)?
    };
    let original_size = tree.size().to_int_size();
    let (width, height) = size(original_size.width(), original_size.height());
    let mut pixmap = Pixmap::new(width, height)
        .wrap_err_with(|| format!("dimensions {width}x{height} invalid"))?;
    resvg::render(
        &tree,
        Transform::from_scale(
            width as f32 / original_size.width() as f32,
            height as f32 / original_size.height() as f32,
        ),
        &mut pixmap.as_mut(),
    );
    RgbaImage::from_raw(width, height, pixmap.take()).wrap_err("pixmap geometry unexpected")
}
