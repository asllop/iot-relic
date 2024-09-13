use hyper::{
    Body, Method, Client, Request
};
use hyper_tls::HttpsConnector;
use serde::{
    Deserialize, Serialize
};
use std::collections::HashMap;
use nrltp_server::{
    IntMetricHunk, MetricType
};

#[derive(Serialize, Deserialize, Debug)]
struct Metric {
    name: String,
    #[serde(rename = "type")]
    m_type: String,
    value: f64,
    timestamp: u64,
    attributes: HashMap<String, String>
}

impl Metric {
    pub fn new(name: String, m_type: String, value: f64, timestamp: u64, attributes: HashMap<String, String>) -> Self {
        Self {
            name, m_type, value, timestamp, attributes
        }
    }
}

type DataStore = HashMap<String, Vec<Metric>>;
type MetricStore = Vec<DataStore>;

/// Represents a single message in the New Relic Metric API format.
#[derive(Serialize, Deserialize, Debug)]
struct MetricApiModel(MetricStore);

impl MetricApiModel {
    pub fn new(hunks: &mut [IntMetricHunk], attributes: HashMap<String, String>) -> Self {
        let metrics = hunks
            .iter_mut()
            .map(|hunk| {
                let timestamp = hunk.timestamp();
                let metric_type = Self::convert_type(hunk.metric_type());
                let metric_name: String = hunk.metric_name().into();
                
                hunk.map(|(value, t_offset)| {
                    Metric::new(
                        metric_name.clone(), metric_type.clone(), value as f64, timestamp + (t_offset as f64 / 1000.0) as u64, attributes.clone()
                    )
                }).collect()
            })
            .fold(Vec::<Metric>::new(), |mut a, mut b| {
                a.append(&mut b);
                a
            });

        Self::pack(metrics)
    }

    fn convert_type(metric_type: MetricType) -> String {
        match metric_type {
            MetricType::Gauge => "gauge",
            MetricType::Count => "count",
            _ => "gauge",
        }.into()
    }

    /// Generate a New Relic Metric API message
    fn pack(metrics: Vec<Metric>) -> Self {
        let mut data_store = DataStore::new();
        data_store.insert("metrics".into(), metrics);
        let mut metric_store = MetricStore::new();
        metric_store.push(data_store);
        Self(metric_store)
    }
}

pub async fn nr_push_metrics(license_key: &str, client_id: &str, metric_hunks: &mut [IntMetricHunk]) -> Result<(), Box<dyn std::error::Error>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut attributes = HashMap::new();
    attributes.insert("clientId".into(), client_id.into());

    let metric_api_model = MetricApiModel::new(metric_hunks, attributes);

    let json = serde_json::to_string(&metric_api_model)?;

    println!("JSON =\n{}\n", json);

    let body = Body::from(json);

    let req = Request::builder()
        .method(Method::POST)
        .uri("https://metric-api.newrelic.com/metric/v1")
        .header("Content-Type", "application/json")
        .header("Api-Key", license_key)
        .body(body)
        .expect("request builder");

    let future = client.request(req);
    let res = future.await?;

    println!("Result status = {:?}", res.status().as_str());

    Ok(())
}
