use std::fs::File;
use std::io::{BufReader, BufWriter};

use anyhow::Result;
use flatgeobuf::*;
use geozero::geojson::GeoJsonReader;
use geozero::GeozeroDatasource;

/// This just converts a bunch of GeoJSON files into one flatgeobuffer file. ogr2ogr doesn't
/// reasonably handle multiple input files.
fn main() -> Result<()> {
    let mut fgb = FgbWriter::create("obstacles", GeometryType::Polygon)?;

    for path in std::env::args().skip(1) {
        println!("Reading {path}");
        let mut file = BufReader::new(File::open(path)?);
        let mut reader = GeoJsonReader(&mut file);
        reader.process(&mut fgb)?;
    }

    println!("Writing");
    let mut file = BufWriter::new(File::create("out.fgb")?);
    fgb.write(&mut file)?;

    Ok(())
}
