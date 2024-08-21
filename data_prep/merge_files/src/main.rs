use std::fs::File;
use std::io::{BufReader, BufWriter};

use anyhow::Result;
use flatgeobuf::*;
use geozero::geojson::GeoJsonReader;
use geozero::GeozeroDatasource;

/// This just converts a bunch of GeoJSON files into one flatgeobuffer file. ogr2ogr doesn't
/// reasonably handle multiple input files.
fn main() -> Result<()> {
    // Use Unknown to handle both polygons and multipolygons
    let mut fgb = FgbWriter::create("obstacles", GeometryType::Unknown)?;

    let paths: Vec<_> = std::env::args().skip(1).collect();
    for (idx, path) in paths.iter().enumerate() {
        println!("Reading {path} ({} / {})", idx + 1, paths.len());
        let mut file = BufReader::new(File::open(path)?);
        let mut reader = GeoJsonReader(&mut file);
        reader.process(&mut fgb)?;
    }

    println!("Writing");
    let mut file = BufWriter::new(File::create("out.fgb")?);
    fgb.write(&mut file)?;

    Ok(())
}
