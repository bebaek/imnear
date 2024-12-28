use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, USER_AGENT},
};
use serde_json::Value;

use imnear::Cache;

pub fn locate(address: &str, cache: Cache) -> Option<(f64, f64)> {
    let res_json = match cache.read(address) {
        Some(res_json) => res_json,
        None => {
            eprintln!("Address cache miss");
            let res_json = request_api(address);
            cache.write(address, res_json.clone());
            res_json
        }
    };
    json_to_coords(res_json)
}

fn request_api(address: &str) -> Value {
    let client = Client::new();
    let url = "https://nominatim.openstreetmap.org/search";
    let mut headers = HeaderMap::new();
    // FIXME: enable user to set this to conform to Nominatim policy
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("indie image searcher v0.1.0"),
    );
    let params = vec![("q", address), ("format", "geojson")];
    let url_with_params = reqwest::Url::parse_with_params(url, &params).unwrap();

    let res = client.get(url_with_params).headers(headers).send();
    let res_json = match res {
        Ok(res) => res.json::<serde_json::Value>().unwrap(),
        Err(e) => panic!("err: {:?}", e),
    };
    res_json
}

fn json_to_coords(json_response: serde_json::Value) -> Option<(f64, f64)> {
    let coords = &json_response["features"][0]["geometry"]["coordinates"];
    let lat = match coords[1].as_f64() {
        Some(lat) => lat,
        _ => return None,
    };
    let lon = match coords[0].as_f64() {
        Some(lon) => lon,
        _ => return None,
    };
    Some((lat, lon))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_to_coords_valid() {
        let json_response = r#"
{
  "type": "FeatureCollection",
  "licence": "Data © OpenStreetMap contributors, ODbL 1.0. http://osm.org/copyright",
  "features": [
    {
      "type": "Feature",
      "properties": {
        "place_id": 304848284,
        "osm_type": "relation",
        "osm_id": 1740655,
        "place_rank": 12,
        "category": "boundary",
        "type": "administrative",
        "importance": 0.506869291577351,
        "addresstype": "county",
        "name": "Yellowstone County",
        "display_name": "Yellowstone County, Montana, United States"
      },
      "bbox": [-108.9256571, 45.4608533, -107.4629188, 46.496123],
      "geometry": {
        "type": "Point",
        "coordinates": [-108.276076, 45.9645464]
      }
    },
    {
      "type": "Feature",
      "properties": {
        "place_id": 347170371,
        "osm_type": "relation",
        "osm_id": 9384653,
        "place_rank": 12,
        "category": "boundary",
        "type": "administrative",
        "importance": 0.298273757504901,
        "addresstype": "village",
        "name": "Summer Village of Yellowstone",
        "display_name": "Summer Village of Yellowstone, Alberta, Canada"
      },
      "bbox": [-114.3855856, 53.7311871, -114.3729778, 53.7361412],
      "geometry": {
        "type": "Point",
        "coordinates": [-114.3807141, 53.7335433]
      }
    },
    {
      "type": "Feature",
      "properties": {
        "place_id": 345181622,
        "osm_type": "node",
        "osm_id": 153811247,
        "place_rank": 20,
        "category": "place",
        "type": "hamlet",
        "importance": 0.266167160425083,
        "addresstype": "hamlet",
        "name": "Yellowstone",
        "display_name": "Yellowstone, Town of Fayette, Lafayette County, Wisconsin, United States"
      },
      "bbox": [-89.9904023, 42.7783343, -89.9504023, 42.8183343],
      "geometry": {
        "type": "Point",
        "coordinates": [-89.9704023, 42.7983343]
      }
    },
    {
      "type": "Feature",
      "properties": {
        "place_id": 349935359,
        "osm_type": "node",
        "osm_id": 6646909895,
        "place_rank": 12,
        "category": "place",
        "type": "county",
        "importance": 0.240024700541695,
        "addresstype": "county",
        "name": "Yellowstone (summer village)",
        "display_name": "Yellowstone (summer village), Alberta, Canada"
      },
      "bbox": [-115.0788339, 53.0340404, -113.6788339, 54.4340404],
      "geometry": {
        "type": "Point",
        "coordinates": [-114.3788339, 53.7340404]
      }
    }
  ]
}
"#;
        let des = serde_json::from_str(&json_response).unwrap();

        let coords = json_to_coords(des).unwrap();

        assert_eq!(coords, (45.9645464, -108.276076));
    }

    #[test]
    fn json_to_coords_missing_key() {
        let json_response = r#"{"features": []}"#;
        let des = serde_json::from_str(&json_response).unwrap();

        let coords = json_to_coords(des);

        assert!(coords.is_none());
    }
}
