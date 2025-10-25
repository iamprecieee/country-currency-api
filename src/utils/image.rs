use std::fs::create_dir_all;

use ab_glyph::{FontArc, PxScale};
use anyhow::{Result, anyhow};
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_text_mut, text_size};

use crate::models::country::Country;

pub fn generate_summary_image(
    total_countries: i64,
    top_countries: Vec<Country>,
    last_refreshed: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    create_dir_all("cache").ok();

    // Create a white canvas (800x600)
    let width = 800u32;
    let height = 600u32;
    let mut img: RgbImage = ImageBuffer::from_pixel(width, height, Rgb([255u8, 255u8, 255u8]));

    // Load font (using built-in font data)
    let font_data = include_bytes!("../../assets/DejaVuSans.ttf");
    let font = FontArc::try_from_slice(font_data).map_err(|_| anyhow!("Failed to load font"))?;

    let black = Rgb([0u8, 0u8, 0u8]);

    // Title
    let title_scale = PxScale::from(40.0);
    let text = "Country Data Summary";
    let (text_width, _text_height) = text_size(title_scale, &font, text);
    let x_centered = (width - text_width) / 2;
    let y = 30;
    draw_text_mut(
        &mut img,
        black,
        x_centered as i32,
        y,
        title_scale,
        &font,
        text,
    );

    // Total countries
    let regular_scale = PxScale::from(24.0);
    let total_text = format!("Total Countries: {}", total_countries);
    draw_text_mut(&mut img, black, 30, 100, regular_scale, &font, &total_text);

    // Top 5 heading
    draw_text_mut(
        &mut img,
        black,
        30,
        150,
        regular_scale,
        &font,
        "Top 5 by GDP:",
    );

    // List top 5 countries
    for (i, country) in top_countries.iter().take(5).enumerate() {
        let y_pos = 200 + (i as i32 * 50);
        let gdp_text = match &country.estimated_gdp {
            Some(gdp) => {
                let formatted_gdp = format_number(gdp.to_f64().unwrap());
                format!("{}. {} - ${}", i + 1, country.name, formatted_gdp)
            }
            None => format!("{}. {} - N/A", i + 1, country.name),
        };

        draw_text_mut(&mut img, black, 50, y_pos, regular_scale, &font, &gdp_text);
    }

    // Timestamp
    let timestamp_text = format!(
        "Last Updated: {}",
        last_refreshed.format("%Y-%m-%d %H:%M:%S UTC")
    );
    draw_text_mut(
        &mut img,
        black,
        30,
        500,
        regular_scale,
        &font,
        &timestamp_text,
    );

    img.save("cache/summary.png")?;

    tracing::info!("Summary image generated at cache/summary.png");

    Ok(())
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
