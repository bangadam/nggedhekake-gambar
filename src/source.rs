use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use image::{
    DynamicImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader,
    codecs::webp::WebPDecoder, imageops::FilterType,
};
use thiserror::Error;

const MAX_SOURCE_PIXELS: u64 = 200_000_000;
pub const PREVIEW_MAX_EDGE: u32 = 2_048;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewPixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedSource {
    pub path: PathBuf,
    pub name: String,
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub preview: PreviewPixels,
}

impl LoadedSource {
    pub fn details(&self) -> String {
        format!("{} × {} px · {}", self.width, self.height, self.format)
    }
}

pub trait SourceLoader {
    fn load(&self, path: &Path) -> Result<LoadedSource, SourceError>;
}

#[derive(Default)]
pub struct ImageSourceLoader;

impl SourceLoader for ImageSourceLoader {
    fn load(&self, path: &Path) -> Result<LoadedSource, SourceError> {
        load_source(path)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SourceError {
    #[error("Choose a PNG, JPEG, or WebP image.")]
    UnsupportedFormat,
    #[error("Animated WebP is not supported. Choose a still image.")]
    AnimatedWebp,
    #[error("This image has no pixels. Choose another file.")]
    ZeroSized,
    #[error("This image is too large to preview safely ({width} × {height} px).")]
    TooLarge { width: u32, height: u32 },
    #[error("Could not open this image: {reason}")]
    Open { reason: String },
    #[error("Could not read this image: {reason}")]
    Decode { reason: String },
}

pub fn load_source(path: &Path) -> Result<LoadedSource, SourceError> {
    let reader = ImageReader::open(path)
        .map_err(|error| SourceError::Open {
            reason: error.to_string(),
        })?
        .with_guessed_format()
        .map_err(|error| SourceError::Decode {
            reason: error.to_string(),
        })?;

    let format = reader.format().ok_or(SourceError::UnsupportedFormat)?;
    let format_label = match format {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::WebP => {
            reject_animated_webp(path)?;
            "WebP"
        }
        _ => return Err(SourceError::UnsupportedFormat),
    };

    let mut decoder = reader.into_decoder().map_err(|error| SourceError::Decode {
        reason: error.to_string(),
    })?;
    let (width, height) = decoder.dimensions();
    validate_dimensions(width, height)?;

    let orientation = decoder.orientation().map_err(|error| SourceError::Decode {
        reason: error.to_string(),
    })?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(|error| SourceError::Decode {
        reason: error.to_string(),
    })?;
    image.apply_orientation(orientation);

    let (width, height) = image.dimensions();
    let preview = create_preview(image);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    Ok(LoadedSource {
        path: path.to_path_buf(),
        name,
        format: format_label,
        width,
        height,
        preview,
    })
}

fn reject_animated_webp(path: &Path) -> Result<(), SourceError> {
    let file = File::open(path).map_err(|error| SourceError::Open {
        reason: error.to_string(),
    })?;
    let decoder = WebPDecoder::new(BufReader::new(file)).map_err(|error| SourceError::Decode {
        reason: error.to_string(),
    })?;

    if decoder.has_animation() {
        Err(SourceError::AnimatedWebp)
    } else {
        Ok(())
    }
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), SourceError> {
    if width == 0 || height == 0 {
        return Err(SourceError::ZeroSized);
    }

    if u64::from(width) * u64::from(height) > MAX_SOURCE_PIXELS {
        return Err(SourceError::TooLarge { width, height });
    }

    Ok(())
}

fn create_preview(image: DynamicImage) -> PreviewPixels {
    let (width, height) = image.dimensions();
    let image = if width > PREVIEW_MAX_EDGE || height > PREVIEW_MAX_EDGE {
        image.resize(PREVIEW_MAX_EDGE, PREVIEW_MAX_EDGE, FilterType::Triangle)
    } else {
        image
    };
    let rgba = image.into_rgba8();

    PreviewPixels {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_sized_images() {
        assert_eq!(validate_dimensions(0, 10), Err(SourceError::ZeroSized));
        assert_eq!(validate_dimensions(10, 0), Err(SourceError::ZeroSized));
    }

    #[test]
    fn rejects_images_beyond_preview_limit() {
        assert_eq!(
            validate_dimensions(20_000, 20_000),
            Err(SourceError::TooLarge {
                width: 20_000,
                height: 20_000,
            })
        );
    }

    #[test]
    fn bounds_retained_preview_dimensions() {
        let image = DynamicImage::new_rgba8(4_096, 1_024);
        let preview = create_preview(image);

        assert_eq!((preview.width, preview.height), (2_048, 512));
        assert_eq!(
            preview.rgba.len(),
            preview.width as usize * preview.height as usize * 4
        );
    }
    #[test]
    fn loads_supported_static_formats() {
        for (suffix, format, label) in [
            (".png", ImageFormat::Png, "PNG"),
            (".jpg", ImageFormat::Jpeg, "JPEG"),
            (".webp", ImageFormat::WebP, "WebP"),
        ] {
            let file = tempfile::Builder::new().suffix(suffix).tempfile().unwrap();
            DynamicImage::new_rgba8(40, 20)
                .save_with_format(file.path(), format)
                .unwrap();

            let source = load_source(file.path()).unwrap();

            assert_eq!(source.format, label);
            assert_eq!((source.width, source.height), (40, 20));
            assert_eq!((source.preview.width, source.preview.height), (40, 20));
        }
    }

    #[test]
    fn loads_unicode_and_space_containing_paths() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("gambar uji åé.png");
        DynamicImage::new_rgba8(16, 9)
            .save_with_format(&path, ImageFormat::Png)
            .unwrap();

        let source = load_source(&path).unwrap();

        assert_eq!(source.name, "gambar uji åé.png");
        assert_eq!(source.path, path);
    }

    #[test]
    fn rejects_unsupported_and_corrupt_files() {
        let unsupported = tempfile::Builder::new().suffix(".gif").tempfile().unwrap();
        std::fs::write(unsupported.path(), b"GIF89a").unwrap();
        assert_eq!(
            load_source(unsupported.path()),
            Err(SourceError::UnsupportedFormat)
        );

        let corrupt = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        std::fs::write(corrupt.path(), b"\x89PNG\r\n\x1a\ninvalid").unwrap();
        assert!(matches!(
            load_source(corrupt.path()),
            Err(SourceError::Decode { .. })
        ));
    }
}
