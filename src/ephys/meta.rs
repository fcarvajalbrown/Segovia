use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetaError {
    #[error("failed to read meta file: {0}")]
    Io(#[from] std::io::Error),
    #[error("missing required meta key: {0}")]
    MissingKey(&'static str),
    #[error("could not parse meta key '{key}' from value '{value}'")]
    InvalidValue { key: &'static str, value: String },
}

#[derive(Debug, Clone)]
pub struct SpikeGlxMeta {
    pub n_channels: usize,
    pub sample_rate: f64,
    pub stream_type: String,
    pub declared_file_size_bytes: Option<u64>,
    pub fields: HashMap<String, String>,
}

impl SpikeGlxMeta {
    pub fn from_path(path: &Path) -> Result<Self, MetaError> {
        let text = fs::read_to_string(path)?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, MetaError> {
        let mut fields = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                fields.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        let n_channels = parse_required::<usize>(&fields, "nSavedChans")?;
        let sample_rate = parse_sample_rate(&fields)?;
        let stream_type = fields
            .get("typeThis")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let declared_file_size_bytes = match fields.get("fileSizeBytes") {
            Some(value) => Some(value.parse::<u64>().map_err(|_| MetaError::InvalidValue {
                key: "fileSizeBytes",
                value: value.clone(),
            })?),
            None => None,
        };

        Ok(Self {
            n_channels,
            sample_rate,
            stream_type,
            declared_file_size_bytes,
            fields,
        })
    }
}

fn parse_required<T: std::str::FromStr>(
    fields: &HashMap<String, String>,
    key: &'static str,
) -> Result<T, MetaError> {
    let value = fields.get(key).ok_or(MetaError::MissingKey(key))?;
    value.parse::<T>().map_err(|_| MetaError::InvalidValue {
        key,
        value: value.clone(),
    })
}

fn parse_sample_rate(fields: &HashMap<String, String>) -> Result<f64, MetaError> {
    if fields.contains_key("imSampRate") {
        parse_required::<f64>(fields, "imSampRate")
    } else if fields.contains_key("niSampRate") {
        parse_required::<f64>(fields, "niSampRate")
    } else {
        Err(MetaError::MissingKey("imSampRate|niSampRate"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "nSavedChans=385\nimSampRate=30000.5\ntypeThis=imec\nfileSizeBytes=1234\n~imroTbl=(0,1)(1,2)\n";

    #[test]
    fn parses_required_fields() {
        let meta = SpikeGlxMeta::parse(SAMPLE).unwrap();
        assert_eq!(meta.n_channels, 385);
        assert_eq!(meta.sample_rate, 30000.5);
        assert_eq!(meta.stream_type, "imec");
        assert_eq!(meta.declared_file_size_bytes, Some(1234));
    }

    #[test]
    fn keeps_complex_fields_verbatim() {
        let meta = SpikeGlxMeta::parse(SAMPLE).unwrap();
        assert_eq!(
            meta.fields.get("~imroTbl").map(String::as_str),
            Some("(0,1)(1,2)")
        );
    }

    #[test]
    fn falls_back_to_ni_sample_rate() {
        let meta = SpikeGlxMeta::parse("nSavedChans=8\nniSampRate=25000\ntypeThis=nidq\n").unwrap();
        assert_eq!(meta.sample_rate, 25000.0);
        assert_eq!(meta.stream_type, "nidq");
    }

    #[test]
    fn missing_channel_count_errors() {
        let err = SpikeGlxMeta::parse("imSampRate=30000\n").unwrap_err();
        assert!(matches!(err, MetaError::MissingKey("nSavedChans")));
    }

    #[test]
    fn missing_sample_rate_errors() {
        let err = SpikeGlxMeta::parse("nSavedChans=4\n").unwrap_err();
        assert!(matches!(
            err,
            MetaError::MissingKey("imSampRate|niSampRate")
        ));
    }
}
