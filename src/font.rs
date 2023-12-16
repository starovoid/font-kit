// font-kit/src/font.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A font face loaded into memory.
//!
//! The Font type in this crate represents the default loader.

//pub use crate::loaders::default::Font;

use crate::error::GlyphLoadingError;
use crate::handle::Handle;
use crate::hinting::HintingOptions;
use crate::outline::OutlineSink;
use crate::{
    canvas::{Canvas, Format, RasterizationOptions},
    error::FontLoadingError,
    file_type::FileType,
    loader::{FallbackResult, Loader},
    metrics::Metrics,
    properties::{Properties, Stretch, Style, Weight},
};
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2F, Vector2I};
use std::fs::File;
use std::{path::Path, sync::Arc};
use ttf_parser::{Face, GlyphId};

static ARIAL: &'static [u8] = include_bytes!("../resources/DejaVuSansMono.ttf");

#[derive(Debug, Clone)]
pub struct Font {
    font_data: Arc<Vec<u8>>,
    face: Face<'static>,
}

impl Font {
    pub fn from_handle(handle: &Handle) -> Result<Self, FontLoadingError> {
        <Self as Loader>::from_handle(handle)
    }

    pub fn analyze_file(file: &mut File) -> Result<FileType, FontLoadingError> {
        Ok(FileType::Collection(1))
    }
}

impl Loader for Font {
    type NativeFont = u8;

    fn from_bytes(_font_data: Arc<Vec<u8>>, font_index: u32) -> Result<Self, FontLoadingError> {
        let face = Face::parse(ARIAL, font_index).map_err(|_| FontLoadingError::UnknownFormat)?;
        Ok(Font {
            font_data: Arc::new(ARIAL.to_owned()),
            face,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn from_file(_file: &mut File, _font_index: u32) -> Result<Self, FontLoadingError> {
        /*let mut reader = BufReader::new(file);
        let mut font_data =
            Vec::with_capacity(file.metadata().map(|md| md.len() as usize).unwrap_or(0));
        reader.read_to_end(&mut font_data);*/

        let face = Face::parse(ARIAL, 0).map_err(|_| FontLoadingError::UnknownFormat)?;
        Ok(Font {
            font_data: Arc::new(ARIAL.to_owned()),
            face,
        })
    }

    unsafe fn from_native_font(native_font: Self::NativeFont) -> Self {
        let face = Face::parse(ARIAL, 0)
            .map_err(|_| FontLoadingError::UnknownFormat)
            .unwrap();
        Font {
            font_data: Arc::new(ARIAL.to_owned()),
            face,
        }
    }

    fn analyze_bytes(_font_data: Arc<Vec<u8>>) -> Result<FileType, FontLoadingError> {
        Ok(FileType::Collection(1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn analyze_file(_file: &mut File) -> Result<FileType, FontLoadingError> {
        Ok(FileType::Collection(1))
    }

    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    fn analyze_path<P>(_path: P) -> Result<FileType, FontLoadingError>
    where
        P: AsRef<Path>,
    {
        Ok(FileType::Collection(1))
    }

    fn native_font(&self) -> Self::NativeFont {
        0u8
    }

    fn postscript_name(&self) -> Option<String> {
        None
    }

    fn full_name(&self) -> String {
        "Arial".to_owned()
    }

    fn family_name(&self) -> String {
        "Arail".to_owned()
    }

    fn is_monospace(&self) -> bool {
        self.face.is_monospaced()
    }

    fn glyph_count(&self) -> u32 {
        self.face.number_of_glyphs() as u32
    }

    fn properties(&self) -> Properties {
        use ttf_parser::Weight as W;
        Properties {
            style: if self.face.is_italic() {
                Style::Italic
            } else if self.face.is_oblique() {
                Style::Oblique
            } else {
                Style::Normal
            },
            weight: match self.face.weight() {
                W::Thin => Weight::THIN,
                W::ExtraLight => Weight::EXTRA_LIGHT,
                W::Light => Weight::LIGHT,
                W::Normal => Weight::NORMAL,
                W::Medium => Weight::MEDIUM,
                W::SemiBold => Weight::SEMIBOLD,
                W::Bold => Weight::BOLD,
                W::ExtraBold => Weight::BOLD,
                W::Black => Weight::BLACK,
                W::Other(val) => Weight(val as f32),
            },
            stretch: Stretch::NORMAL,
        }
    }

    fn glyph_for_char(&self, character: char) -> Option<u32> {
        self.face.glyph_index(character).map(|id| id.0 as u32)
    }

    fn outline<S: OutlineSink>(
        &self,
        glyph_id: u32,
        hinting_mode: HintingOptions,
        sink: &mut S,
    ) -> Result<(), GlyphLoadingError> {
        Ok(())
    }

    fn typographic_bounds(&self, glyph_id: u32) -> Result<RectF, GlyphLoadingError> {
        let rect = self
            .face
            .glyph_bounding_box(ttf_parser::GlyphId(glyph_id as u16))
            .ok_or(GlyphLoadingError::NoSuchGlyph)?;

        let rect = RectF::from_points(
            Vector2F::new(rect.x_min as f32, rect.y_min as f32),
            Vector2F::new(rect.x_max as f32, rect.y_max as f32),
        );
        Ok(rect)
    }

    fn advance(&self, glyph_id: u32) -> Result<Vector2F, GlyphLoadingError> {
        let h = self
            .face
            .glyph_hor_advance(GlyphId(glyph_id as u16))
            .ok_or(GlyphLoadingError::NoSuchGlyph)?;
        let v = self
            .face
            .glyph_ver_advance(GlyphId(glyph_id as u16))
            .ok_or(GlyphLoadingError::NoSuchGlyph)?;
        Ok(Vector2F::new(h as f32, v as f32))
    }

    fn origin(&self, glyph_id: u32) -> Result<Vector2F, GlyphLoadingError> {
        Ok(Vector2F::default())
    }

    fn metrics(&self) -> Metrics {
        Metrics::default()
    }

    fn rasterize_glyph(
        &self,
        canvas: &mut Canvas,
        glyph_id: u32,
        point_size: f32,
        transform: Transform2F,
        hinting_options: HintingOptions,
        rasterization_options: RasterizationOptions,
    ) -> Result<(), GlyphLoadingError> {
        /*let raster = self
            .face
            .glyph_raster_image(GlyphId(self.glyph_for_char('a').unwrap() as u16), 12)
            .unwrap();
        canvas.pixels = raster.data.to_owned();*/
        canvas.pixels = [
            88u8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 140,
            88, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 140, 0,
            0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255,
            232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255,
            232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255,
            232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255,
            232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255,
            232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255,
            232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 255, 232, 0, 0, 0, 0, 0, 0, 0,
        ]
        .to_vec();
        canvas.pixels.extend(
            std::iter::repeat(0u8).take((canvas.size.x() * canvas.size.y()) as usize - 20 * 17),
        ); //canvas.size = Vector2I::new(20, 17);
        canvas.format = Format::A8;
        Ok(())
    }

    fn get_fallbacks(&self, text: &str, locale: &str) -> FallbackResult<Self> {
        FallbackResult {
            fonts: Vec::new(),
            valid_len: text.len(),
        }
    }

    fn load_font_table(&self, table_tag: u32) -> Option<Box<[u8]>> {
        self.face
            .raw_face()
            .table(ttf_parser::Tag(table_tag))
            .map(|t| t.into())
    }

    fn supports_hinting_options(
        &self,
        hinting_options: HintingOptions,
        for_rasterization: bool,
    ) -> bool {
        false
    }

    fn copy_font_data(&self) -> Option<Arc<Vec<u8>>> {
        Some(Arc::clone(&self.font_data))
    }
}
