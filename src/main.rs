use anyhow::{Context, Result};
use goo::{HeaderInfo, LayerEncoder};
use goo::serde::{DynamicSerializer, Serializer};
use image::{ImageReader, GenericImageView, imageops::FilterType, RgbaImage, Rgba};
use goo::PreviewImage;
use std::fs;

fn main() -> Result<()> {
    // ---- args: png_in goo_out width height exposure_seconds layer_height_mm ----
    // Example:
    //   png2goo.exe mask.png out.goo 11520 5120 20 0.05
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 7 {
        eprintln!("Usage: png2goo <input.png> <output.goo> <x_res> <y_res> <exposure_s> <layer_h_mm>");
        std::process::exit(2);
    }
    let inp = &args[1];
    let out = &args[2];
    let x_res: u32 = args[3].parse()?;
    let y_res: u32 = args[4].parse()?;
    let exposure_s: f32 = args[5].parse()?;
    let layer_h_mm: f32 = args[6].parse()?;

    // ---- load PNG (any bit depth), convert to 8-bit grayscale
    let img = ImageReader::open(inp)?.decode()?.to_luma8();
    if img.width() != x_res || img.height() != y_res {
        anyhow::bail!(
            "Input image size {}x{} does not match requested LCD {}x{}",
            img.width(),
            img.height(),
            x_res,
            y_res
        );
    }

    // ---- encode layer as runs (row-major)
    let mut enc = LayerEncoder::new();
    for y in 0..y_res {
        let row = img.view(0, y, x_res, 1);
        // compress consecutive pixels with same value
        let mut run_val = row.get_pixel(0, 0)[0];
        let mut run_len = 1u32;
        for x in 1..x_res {
            let v = row.get_pixel(x, 0)[0];
            if v == run_val {
                run_len += 1;
            } else {
                enc.add_run(run_len.into(), run_val);
                run_val = v;
                run_len = 1;
            }
        }
        enc.add_run(run_len.into(), run_val);
    }
    let (layer_bytes, checksum) = enc.finish();

    // ---- minimal header
    let mut header = HeaderInfo::default();
    header.x_resolution = x_res as u16;
    header.y_resolution = y_res as u16;
    header.layer_count = 1;
    header.layer_thickness = layer_h_mm;
    header.bottom_exposure_time = exposure_s;
    header.exposure_time = 0.0;

    // Convert grayscale image (Luma<u8>) to RGBA because PreviewImage helpers expect RgbaImage
    let rgba_img: RgbaImage = {
        let mut out = RgbaImage::new(img.width(), img.height());
        for y in 0..img.height() {
            for x in 0..img.width() {
                let v = img.get_pixel(x, y)[0];
                out.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        out
    };

    // Resize RGBA to exact preview sizes then construct preview images
    let big_rgba = image::imageops::resize(&rgba_img, 290, 290, FilterType::Triangle);
    let small_rgba = image::imageops::resize(&rgba_img, 116, 116, FilterType::Triangle);

    // Populate PreviewImage by mapping pixels to set_pixel (expects f32 RGB 0.0..1.0)
    let mut big_preview = PreviewImage::<290, 290>::empty();
    for y in 0..290u32 {
        for x in 0..290u32 {
            let p = big_rgba.get_pixel(x, y).0;
            let color = (p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0);
            big_preview.set_pixel(x as usize, y as usize, color);
            
        }
    }

    let mut small_preview = PreviewImage::<116, 116>::empty();
    for y in 0..116u32 {
        for x in 0..116u32 {
            let p = small_rgba.get_pixel(x, y).0;
            let color = (p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0);
            small_preview.set_pixel(x as usize, y as usize, color);
        }
    }

    header.big_preview = big_preview;
    header.small_preview = small_preview;

    // ---- build .goo and serialize manually (use public Serializer API)
    // We can't construct goo::layer_content::LayerContent directly because the
    // module is private, so serialize header and layer contents manually using
    // the public Serializer (DynamicSerializer). We use reasonable defaults for
    // the layer header fields mirroring LayerContent::default().
    let mut ser = DynamicSerializer::new();

    // serialize header (this writes the big fixed-size header)
    header.serialize(&mut ser);

    // --- serialize single layer (fields match LayerContent::serialize)
    // Values here mirror LayerContent::default() where appropriate.
    ser.write_u16(0); // pause_flag
    ser.write_f32(200.0); // pause_position_z
    ser.write_f32(header.layer_thickness); // layer_position_z (use header thickness)
    ser.write_f32(header.bottom_exposure_time); // layer_exposure_time (reuse bottom_exposure_time or user-specified)
    ser.write_f32(0.0); // layer_off_time
    ser.write_f32(0.0); // before_lift_time
    ser.write_f32(0.0); // after_lift_time
    ser.write_f32(0.0); // after_retract_time
    ser.write_f32(5.0); // lift_distance
    ser.write_f32(65.0); // lift_speed
    ser.write_f32(0.0); // second_lift_distance
    ser.write_f32(0.0); // second_lift_speed
    ser.write_f32(5.0); // retract_distance
    ser.write_f32(150.0); // retract_speed
    ser.write_f32(0.0); // second_retract_distance
    ser.write_f32(0.0); // second_retract_speed
    ser.write_u16(255); // light_pwm

    // delimiter (CRLF)
    ser.write_bytes(&[0x0D, 0x0A]);

    // write data length (data len + 2) as u32
    ser.write_u32(layer_bytes.len() as u32 + 2);

    // marker byte 0x55 then layer data and checksum
    ser.write_bytes(&[0x55]);
    ser.write_bytes(&layer_bytes);
    ser.write_u8(checksum);

    // delimiter
    ser.write_bytes(&[0x0D, 0x0A]);

    // append ending string (same as goo::ENDING_STRING)
    ser.write_bytes(&[0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x44, 0x4C, 0x50, 0x00]);

    let bytes = ser.into_inner();
    fs::write(out, bytes).context("write .goo")?;
    println!("Wrote {}", out);
    Ok(())
}
