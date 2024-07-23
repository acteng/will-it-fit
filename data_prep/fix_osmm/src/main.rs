use std::fs::File;
use std::io::{BufWriter, Cursor, Read};

use anyhow::Result;
use flatgeobuf::{FgbWriter, GeometryType};
use geozero::{mvt::Message, GeozeroDatasource};
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
    let zoom = 16_u8;
    let (x1, y1) = lon_lat_to_tile(bounds.left, bounds.top, zoom.into());
    let (x2, y2) = lon_lat_to_tile(bounds.right, bounds.bottom, zoom.into());
    // TODO Something is quite odd here, but the loop below works
    println!("Reading x = {x1} to {x2}, y = {y1} to {y2}");

    let mut fgb = FgbWriter::create("obstacles", GeometryType::MultiPolygon)?;

    let mut n = 0;
    'OUTER: for x in x1..=x2 {
        for y in y1..=y2 {
            match mbtiles.get_tile(&mut conn, zoom, x, y).await {
                Ok(Some(bytes)) => {
                    handle_tile(bytes, &mut fgb)?;

                    n += 1;
                    if n == 100 {
                        break 'OUTER;
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    println!("Error for {x}, {y}: {err}");
                }
            }
        }
    }

    println!("Writing");
    let mut file = BufWriter::new(File::create("out.fgb")?);
    fgb.write(&mut file)?;

    Ok(())
}

fn handle_tile(bytes: Vec<u8>, fgb: &mut FgbWriter) -> Result<()> {
    let mut decoder = flate2::read::GzDecoder::new(Cursor::new(bytes));
    let mut gunzipped = Vec::new();
    decoder.read_to_end(&mut gunzipped)?;

    let tile = geozero::mvt::Tile::decode(Cursor::new(gunzipped))?;
    for mut layer in tile.layers {
        layer.process(fgb)?;
    }
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

    let x = (1.0 + (x / std::f64::consts::PI)) / 2.0;
    let y = (1.0 - (y / std::f64::consts::PI)) / 2.0;

    let num_tiles = 2u32.pow(zoom) as f64;

    (
        (x * num_tiles).floor() as u32,
        (y * num_tiles).floor() as u32,
    )
}
