use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Earthquake {
    date: String,
    location: String,
    country: String,
    magnitude: MagnitudeValue,
    note: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
enum MagnitudeValue {
    Single(f64),
    Multiple(Vec<f64>),
}

#[derive(Deserialize)]
struct EarthquakeQuery {
    date: Option<String>,
    country: Option<String>,
    min_magnitude: Option<f64>,
    location: Option<String>,
    note: Option<String>,
}

fn load_earthquake_data() -> Vec<Earthquake> {
    let data = r#"[
  {
    "date": "13 พ.ค. 2478",
    "location": "จ.น่าน",
    "country": "ไทย",
    "magnitude": 6.5,
    "note": ""
  },
  {
    "date": "17 ก.พ. 2518",
    "location": "อ.ท่าสองยาง จ.ตาก",
    "country": "ไทย",
    "magnitude": 5.6,
    "note": ""
  },
  {
    "date": "15–22 เม.ย. 2536",
    "location": "จ.ศรีสวัสดิ์ จ.กาญจนบุรี",
    "country": "ไทย",
    "magnitude": [5.3, 5.9, 5.2],
    "note": "เกิด 3 ครั้ง"
  },
  {
    "date": "11 ก.ย. 2537",
    "location": "จ.พาน จ.เชียงราย",
    "country": "ไทย",
    "magnitude": 5.1,
    "note": ""
  },
  {
    "date": "9 ธ.ค. 2538",
    "location": "อ.ร้องกวาง จ.แพร่",
    "country": "ไทย",
    "magnitude": 5.1,
    "note": ""
  },
  {
    "date": "21 ธ.ค. 2538",
    "location": "จ.พร้าว จ.เชียงใหม่",
    "country": "ไทย",
    "magnitude": 5.2,
    "note": ""
  },
  {
    "date": "22 ธ.ค. 2539",
    "location": "พรมแดนไทย–ลาว–เมียนมา",
    "country": "ไทย/ลาว/เมียนมา",
    "magnitude": 5.5,
    "note": ""
  },
  {
    "date": "16 พ.ค. 2550",
    "location": "ใกล้ จ.เชียงราย",
    "country": "ลาว",
    "magnitude": 6.3,
    "note": ""
  },
  {
    "date": "24 มี.ค. 2554",
    "location": "ใกล้ จ.เชียงราย",
    "country": "เมียนมา",
    "magnitude": 6.8,
    "note": ""
  },
  {
    "date": "5 พ.ค. 2557",
    "location": "อ.แม่ลาว จ.เชียงราย",
    "country": "ไทย",
    "magnitude": 6.3,
    "note": ""
  },
  {
    "date": "28 มี.ค. 2568",
    "location": "ประเทศเมียนมา",
    "country": "เมียนมา",
    "magnitude": 8.2,
    "note": "กทม. รับรู้แรงสั่นสะเทือน"
  }
]"#;

    match serde_json::from_str(data) {
        Ok(earthquakes) => earthquakes,
        Err(e) => {
            tracing::error!("Failed to parse earthquake data: {}", e);
            Vec::new()
        }
    }
}

async fn get_earthquakes(
    Query(params): Query<EarthquakeQuery>,
) -> Result<Json<Vec<Earthquake>>, StatusCode> {
    let earthquakes = load_earthquake_data();
    
    let filtered_earthquakes = earthquakes
        .into_iter()
        .filter(|eq| {
            // Filter by date if specified
            if let Some(date) = &params.date {
                if !eq.date.contains(date) {
                    return false;
                }
            }

            // Filter by country if specified
            if let Some(country) = &params.country {
                if !eq.country.contains(country) {
                    return false;
                }
            }

            // Filter by minimum magnitude if specified
            if let Some(min_mag) = params.min_magnitude {
                match &eq.magnitude {
                    MagnitudeValue::Single(mag) => {
                        if *mag < min_mag {
                            return false;
                        }
                    }
                    MagnitudeValue::Multiple(magnitudes) => {
                        if magnitudes.iter().all(|mag| *mag < min_mag) {
                            return false;
                        }
                    }
                }
            }
            
            // Filter by location if specified
            if let Some(location) = &params.location {
                if !eq.location.contains(location) {
                    return false;
                }
            }
            
            // Filter by note if specified
            if let Some(note) = &params.note {
                if !eq.note.contains(note) {
                    return false;
                }
            }

            true
        })
        .collect();
    
    Ok(Json(filtered_earthquakes))
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("Starting server...");

    // Load earthquake data
    let earthquakes = load_earthquake_data();
    tracing::info!("Loaded {} earthquake records", earthquakes.len());
    // Check if data is loaded correctly
    if earthquakes.is_empty() {
        tracing::warn!("No earthquake data loaded");
    } else {
        tracing::info!("Earthquake data loaded successfully");
    }

    // Create a Router
    let app = Router::new()
        .route("/earthquakes", get(get_earthquakes));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
