

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperaturePressureReading {
    pub ts: u64,
    pub t: f64,
    pub p: f64,
}
