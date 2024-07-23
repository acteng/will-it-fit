mod wrap;

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
    // TODO Could read as the minzoom/maxzoom, which should be the same
    let zoom = 16_u8;
    let (x1, y1) = lon_lat_to_tile(bounds.left, bounds.top, zoom.into());
    let (x2, y2) = lon_lat_to_tile(bounds.right, bounds.bottom, zoom.into());
    // TODO Something is quite odd here, but the loop below works
    println!("Reading x = {x1} to {x2}, y = {y1} to {y2}");

    let mut fgb = Some(FgbWriter::create("obstacles", GeometryType::MultiPolygon)?);

    let mut n = 0;
    'OUTER: for tile_x in x1..=x2 {
        for tile_y in y1..=y2 {
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

                    n += 1;
                    if n == 1000 {
                        break 'OUTER;
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    println!("Error for {tile_x}, {tile_y}: {err}");
                }
            }
        }
    }

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

    let x = (1.0 + (x / std::f64::consts::PI)) / 2.0;
    let y = (1.0 - (y / std::f64::consts::PI)) / 2.0;

    let num_tiles = 2u32.pow(zoom) as f64;

    (
        (x * num_tiles).floor() as u32,
        (y * num_tiles).floor() as u32,
    )
}

// Via https://gis.stackexchange.com/questions/401541/decoding-mapbox-vector-tiles
// TODO I think mercantile or tile-grid can replace both of these
fn pixel_to_lon_lat(
    pixel_x: f64,
    pixel_y: f64,
    tile_x: u32,
    tile_y: u32,
    zoom: u8,
    extent: f64,
) -> (f64, f64) {
    let n = 2u32.pow(zoom.into()) as f64;
    let x_tile = (tile_x as f64) + (pixel_x / extent);
    let y_tile = (tile_y as f64) + ((extent - pixel_y) / extent);
    let lon = (x_tile / n) * 360.0 - 180.0;
    let lat = (std::f64::consts::PI * (1.0 - 2.0 * y_tile / n))
        .sinh()
        .atan()
        .to_degrees();
    (lon, lat)
}
