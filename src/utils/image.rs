use std::fs::create_dir_all;

use ab_glyph::{FontArc, PxScale};
use anyhow::{Result, anyhow};
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage, imageops};
use imageproc::drawing::{draw_text_mut, text_size};

use crate::models::country::Country;

pub async fn generate_summary_image(
    total_countries: i64,
    top_countries: Vec<Country>,
    last_refreshed: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    create_dir_all("cache").ok();

    // Create a black canvas (800x600)
    let width = 800u32;
    let height = 600u32;
    let mut img: RgbImage = ImageBuffer::from_pixel(width, height, Rgb([0u8, 0u8, 0u8]));

    // Load font
    let font_data = include_bytes!("../../assets/DejaVuSans.ttf");
    let font = FontArc::try_from_slice(font_data).map_err(|_| anyhow!("Failed to load font"))?;

    let white = Rgb([255u8, 255u8, 255u8]);

    // Title
    let title_scale = PxScale::from(40.0);
    let text = "Country Data Summary";
    let (text_width, _text_height) = text_size(title_scale, &font, text);
    let x_centered = (width - text_width) / 2;
    let y = 30;
    draw_text_mut(
        &mut img,
        white,
        x_centered as i32,
        y,
        title_scale,
        &font,
        text,
    );

    // Total countries
    let regular_scale = PxScale::from(24.0);
    let total_text = format!("Total Countries: {}", total_countries);
    draw_text_mut(&mut img, white, 30, 100, regular_scale, &font, &total_text);

    // Top 5 heading
    draw_text_mut(
        &mut img,
        white,
        30,
        150,
        regular_scale,
        &font,
        "Top 5 by GDP:",
    );

    // List top 5 countries with flags
    for (i, country) in top_countries.iter().take(5).enumerate() {
        let y_pos = 200 + (i as i32 * 60);

        if let Some(flag_url) = &country.flag_url {
            if let Ok(flag_img) = fetch_and_resize_flag(flag_url, 40, 30).await {
                overlay_image(&mut img, &flag_img, 50, y_pos as u32);
            }
        }

        let gdp_text = match &country.estimated_gdp {
            Some(gdp) => {
                let formatted_gdp = format_number(gdp.to_f64().unwrap());
                format!("{}. {} - ${}", i + 1, country.name, formatted_gdp)
            }
            None => format!("{}. {} - N/A", i + 1, country.name),
        };

        draw_text_mut(
            &mut img,
            white,
            100,
            y_pos + 5,
            regular_scale,
            &font,
            &gdp_text,
        );
    }

    // Timestamp
    let timestamp_text = format!(
        "Last Updated: {}",
        last_refreshed.format("%Y-%m-%d %H:%M:%S UTC")
    );
    draw_text_mut(
        &mut img,
        white,
        30,
        520,
        regular_scale,
        &font,
        &timestamp_text,
    );

    img.save("cache/summary.png")?;

    tracing::info!("Summary image generated at cache/summary.png");

    Ok(())
}

async fn fetch_and_resize_flag(
    flag_url: &str,
    target_width: u32,
    target_height: u32,
) -> Result<DynamicImage> {
    let png_url = convert_to_png_url(flag_url);

    let response = reqwest::get(&png_url).await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch flag: HTTP {}", response.status()));
    }

    let bytes = response.bytes().await?;

    let flag_image = image::load_from_memory(&bytes)?;

    let resized =
        flag_image.resize_exact(target_width, target_height, imageops::FilterType::Lanczos3);

    Ok(resized)
}

fn convert_to_png_url(url: &str) -> String {
    if url.contains("flagcdn.com") && url.ends_with(".svg") {
        // Convert https://flagcdn.com/ng.svg to https://flagcdn.com/w80/ng.png
        url.replace(".svg", ".png")
            .replace("flagcdn.com/", "flagcdn.com/w80/")
    } else {
        url.to_string()
    }
}

fn overlay_image(base: &mut RgbImage, overlay: &DynamicImage, x: u32, y: u32) {
    let overlay_rgb = overlay.to_rgb8();

    for (dx, dy, pixel) in overlay_rgb.enumerate_pixels() {
        let px = x + dx;
        let py = y + dy;

        if px < base.width() && py < base.height() {
            base.put_pixel(px, py, *pixel);
        }
    }
}

fn format_number(num: f64) -> String {
    if num >= 1_000_000_000.0 {
        format!("{:.1}B", num / 1_000_000_000.0)
    } else if num >= 1_000_000.0 {
        format!("{:.1}M", num / 1_000_000.0)
    } else if num >= 1_000.0 {
        format!("{:.1}K", num / 1_000.0)
    } else {
        format!("{}", num)
    }
}
