//! File import for SVG and bitmap files.

use base64::{engine::general_purpose::STANDARD, Engine};
use image::GenericImageView;
use regex::Regex;
use std::fs;
use std::path::Path;
use thiserror::Error;

use super::document::{BitmapContent, BoundingBox, Document, DocumentKind, SvgContent, Transform};

/// Import errors
#[derive(Error, Debug)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to parse SVG: {0}")]
    SvgParse(String),
}

/// Supported file extensions
pub fn is_supported_extension(ext: &str) -> bool {
    let ext = ext.to_lowercase();
    matches!(
        ext.as_str(),
        "svg" | "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp"
    )
}

/// Import a file and create a Document
pub fn import_file(path: &Path) -> Result<Document, ImportError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled")
        .to_string();

    let (kind, bounds) = match ext.as_str() {
        "svg" => import_svg(path)?,
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => import_bitmap(path, &ext)?,
        _ => return Err(ImportError::UnsupportedFormat(ext)),
    };

    Ok(Document {
        id: 0, // Will be assigned by DocumentList
        name,
        source_path: Some(path.to_path_buf()),
        kind,
        transform: Transform::default(),
        visible: true,
        locked: false,
        original_bounds: bounds,
    })
}

/// Import an SVG file
fn import_svg(path: &Path) -> Result<(DocumentKind, BoundingBox), ImportError> {
    let raw_svg = fs::read_to_string(path)?;

    // Extract dimensions from SVG
    let (width, height) = parse_svg_dimensions(&raw_svg)?;

    let content = SvgContent {
        width,
        height,
        paths: Vec::new(), // Path extraction can be added later for GCode generation
        raw_svg,
    };

    let bounds = BoundingBox::new(0.0, 0.0, width, height);

    Ok((DocumentKind::Svg(content), bounds))
}

/// Parse SVG dimensions from viewBox or width/height attributes
fn parse_svg_dimensions(svg: &str) -> Result<(f64, f64), ImportError> {
    // Try viewBox first: viewBox="0 0 width height"
    let viewbox_re = Regex::new(r#"viewBox\s*=\s*["']([^"']+)["']"#).unwrap();
    if let Some(caps) = viewbox_re.captures(svg) {
        let viewbox = &caps[1];
        let parts: Vec<f64> = viewbox
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 4 {
            return Ok((parts[2], parts[3]));
        }
    }

    // Try width/height attributes
    let width_re = Regex::new(r#"width\s*=\s*["']([0-9.]+)(px|mm|in|pt|)?["']"#).unwrap();
    let height_re = Regex::new(r#"height\s*=\s*["']([0-9.]+)(px|mm|in|pt|)?["']"#).unwrap();

    let width = width_re
        .captures(svg)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok());

    let height = height_re
        .captures(svg)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok());

    match (width, height) {
        (Some(w), Some(h)) => Ok((w, h)),
        _ => Err(ImportError::SvgParse(
            "Could not determine SVG dimensions".into(),
        )),
    }
}

/// Import a bitmap file
fn import_bitmap(path: &Path, format: &str) -> Result<(DocumentKind, BoundingBox), ImportError> {
    // Read image to get dimensions
    let img = image::open(path)?;
    let (width, height) = img.dimensions();

    // Read raw file and encode as data URL
    let raw_bytes = fs::read(path)?;
    let b64 = STANDARD.encode(&raw_bytes);

    let mime_type = match format {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    let data_url = format!("data:{};base64,{}", mime_type, b64);

    let content = BitmapContent {
        width,
        height,
        data_url,
        format: format.to_string(),
    };

    // Default: 1 pixel = 0.1mm (adjust as needed, or make configurable)
    let pixels_per_mm = 10.0;
    let width_mm = width as f64 / pixels_per_mm;
    let height_mm = height as f64 / pixels_per_mm;

    let bounds = BoundingBox::new(0.0, 0.0, width_mm, height_mm);

    Ok((DocumentKind::Bitmap(content), bounds))
}

/// Import from raw bytes (for drag-drop)
pub fn import_from_bytes(
    name: &str,
    bytes: &[u8],
    mime_type: &str,
) -> Result<Document, ImportError> {
    let (kind, bounds) = if mime_type == "image/svg+xml" || name.ends_with(".svg") {
        let raw_svg = String::from_utf8_lossy(bytes).to_string();
        let (width, height) = parse_svg_dimensions(&raw_svg)?;
        let content = SvgContent {
            width,
            height,
            paths: Vec::new(),
            raw_svg,
        };
        let bounds = BoundingBox::new(0.0, 0.0, width, height);
        (DocumentKind::Svg(content), bounds)
    } else {
        // Treat as bitmap
        let img = image::load_from_memory(bytes)?;
        let (width, height) = img.dimensions();

        let b64 = STANDARD.encode(bytes);
        let data_url = format!("data:{};base64,{}", mime_type, b64);

        let format = mime_type.split('/').nth(1).unwrap_or("png").to_string();
        let content = BitmapContent {
            width,
            height,
            data_url,
            format,
        };

        let pixels_per_mm = 10.0;
        let width_mm = width as f64 / pixels_per_mm;
        let height_mm = height as f64 / pixels_per_mm;
        let bounds = BoundingBox::new(0.0, 0.0, width_mm, height_mm);

        (DocumentKind::Bitmap(content), bounds)
    };

    Ok(Document {
        id: 0,
        name: name.to_string(),
        source_path: None,
        kind,
        transform: Transform::default(),
        visible: true,
        locked: false,
        original_bounds: bounds,
    })
}
