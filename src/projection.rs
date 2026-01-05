// Copyright 2026 Colton Loftus
// SPDX-License-Identifier: Apache-2.0

use proj::Proj;

pub const RATATUI_MAP_CRS: &str = "EPSG:4326";

#[derive(Clone)]
pub struct Bbox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl Bbox {
    pub fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Self {
        Self {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }

    pub fn from_flatgeobuf_envelope<'a>(
        envelope: &flatbuffers::Vector<'a, f64>,
    ) -> Result<Self, String> {
        if envelope.len() != 4 {
            return Err("Flatgeobuf envelope must have 4 values".to_string());
        }

        Ok(Self {
            xmin: envelope.get(0),
            ymin: envelope.get(1),
            xmax: envelope.get(2),
            ymax: envelope.get(3),
        })
    }
    pub fn project_to_ratatui_map_crs(
        &self,
        source_crs: &str,
    ) -> Result<(Self, String), proj::ProjError> {
        if source_crs == RATATUI_MAP_CRS {
            return Ok((
                self.to_owned(),
                format!("Extent of data in {RATATUI_MAP_CRS}"),
            ));
        }

        let src_to_ratatui_crs = Proj::new_known_crs(source_crs, RATATUI_MAP_CRS, None).unwrap();
        let (new_xmin, new_ymin) = src_to_ratatui_crs.convert((self.xmin, self.ymin))?;
        let (new_xmax, new_ymax) = src_to_ratatui_crs.convert((self.xmax, self.ymax))?;

        Ok((
            Bbox::new(new_xmin, new_ymin, new_xmax, new_ymax),
            format!("Extent of data in {source_crs} projected to {RATATUI_MAP_CRS}"),
        ))
    }
}
