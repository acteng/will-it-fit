mod wrap;

use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufWriter, Cursor, Read};

use anyhow::Result;
use flatgeobuf::{FgbWriter, GeometryType};
use geozero::{mvt::Message, GeozeroDatasource};
use indicatif::{ProgressBar, ProgressStyle};
use mbtiles::Mbtiles;

/// This script takes the `OSMasterMapTopography_gb_TopographicArea.mbtiles` file as input and
/// converts it to flatgeobuf. It's specialized for this one file.
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path = &args[1];
    println!("Opening {path}");

    let mbtiles = Mbtiles::new(path)?;
    let mut conn = mbtiles.open_readonly().await?;

    let metadata = mbtiles.get_metadata(&mut conn).await?;
    let bounds = metadata.tilejson.bounds.unwrap();
    // TODO Could read as the minzoom/maxzoom, which should be the same
    let zoom = 16_u8;
    let (x1, y1) = lon_lat_to_tile(bounds.left, bounds.top, zoom.into());
    let (x2, y2) = lon_lat_to_tile(bounds.right, bounds.bottom, zoom.into());
    // TODO Something is quite odd here, but the loop below works
    println!("Reading x = {x1} to {x2}, y = {y1} to {y2}");

    let mut fgb = Some(FgbWriter::create("obstacles", GeometryType::MultiPolygon)?);
    let progress = ProgressBar::new(((x2 - x1 + 1) * (y2 - y1 + 1)).into()).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    'OUTER: for tile_x in x1..=x2 {
        for tile_y in y1..=y2 {
            progress.inc(1);
            // TODO Still tmp debugging
            if false && progress.position() == 2_000_000 {
                break 'OUTER;
            }

            match mbtiles.get_tile(&mut conn, zoom, tile_x, tile_y).await {
                Ok(Some(bytes)) => {
                    let extent = 4096.0;
                    let mut out = wrap::WrappedProcessor::new(fgb.take().unwrap(), |x, y| {
                        pixel_to_lon_lat(x, y, tile_x, tile_y, zoom, extent)
                    });
                    let mut decoder = flate2::read::GzDecoder::new(Cursor::new(bytes));
                    let mut gunzipped = Vec::new();
                    decoder.read_to_end(&mut gunzipped)?;

                    let tile = geozero::mvt::Tile::decode(Cursor::new(gunzipped))?;
                    for mut layer in tile.layers {
                        layer.process(&mut out)?;
                    }
                    fgb = Some(out.inner);
                }
                Ok(None) => {}
                Err(err) => {
                    println!("Error for {tile_x}, {tile_y}: {err}");
                }
            }
        }
    }
    progress.finish();

    println!("Writing");
    let mut file = BufWriter::new(File::create("out.fgb")?);
    fgb.take().unwrap().write(&mut file)?;

    Ok(())
}

// Thanks to https://github.com/MilesMcBain/slippymath/blob/master/R/slippymath.R
// Use https://crates.io/crates/tile-grid or something instead?
// Alternatively https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames#Python
fn lon_lat_to_tile(lon: f64, lat: f64, zoom: u32) -> (u32, u32) {
    let lon_radians = lon.to_radians();
    let lat_radians = lat.to_radians();

    let x = lon_radians;
    let y = lat_radians.tan().asinh();

    let x = (1.0 + (x / PI)) / 2.0;
    let y = (1.0 - (y / PI)) / 2.0;

    let num_tiles = 2u32.pow(zoom) as f64;

    (
        (x * num_tiles).floor() as u32,
        (y * num_tiles).floor() as u32,
    )
}

// Via
// https://github.com/Amyantis/python-vt2geojson/blob/0ab4f10fcf5dc51ce3aa605506dd46f3601292ae/vt2geojson/features.py#L35
// TODO I think mercantile or tile-grid can replace both of these
fn pixel_to_lon_lat(
    pixel_x: f64,
    pixel_y: f64,
    tile_x: u32,
    tile_y: u32,
    zoom: u8,
    extent: f64,
) -> (f64, f64) {
    let size = extent * 2.0_f64.powi(zoom.into());
    let x0 = extent * (tile_x as f64);
    let y0 = extent * (tile_y as f64);

    let y2 = 180.0 - (pixel_y + y0) * 360.0 / size;
    let lon = (pixel_x + x0) * 360.0 / size - 180.0;
    let lat = 360.0 / PI * (y2 * PI / 180.0).exp().atan() - 90.0;
    (lon, lat)
}
