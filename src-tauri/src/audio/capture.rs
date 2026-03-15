#[cfg(not(test))]
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceSnapshot {
    pub input_devices: Vec<String>,
    pub default_input: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedDevice {
    pub active_device: String,
    pub fallback_from: Option<String>,
}

pub fn dedupe_preserve_order(devices: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for device in devices {
        if !out.contains(&device) {
            out.push(device);
        }
    }
    out
}

pub fn resolve_input_device(selected: Option<&str>, snapshot: &DeviceSnapshot) -> Result<ResolvedDevice, String> {
    let available = &snapshot.input_devices;
    let default = snapshot.default_input.as_ref();

    if available.is_empty() {
        return Err("No microphone input device is available.".to_string());
    }

    match selected {
        Some(requested) => {
            if available.iter().any(|name| name == requested) {
                return Ok(ResolvedDevice {
                    active_device: requested.to_string(),
                    fallback_from: None,
                });
            }

            let fallback = default
                .cloned()
                .or_else(|| available.first().cloned())
                .ok_or_else(|| "No microphone input device is available.".to_string())?;

            Ok(ResolvedDevice {
                active_device: fallback,
                fallback_from: Some(requested.to_string()),
            })
        }
        None => {
            let active = default
                .cloned()
                .or_else(|| available.first().cloned())
                .ok_or_else(|| "No microphone input device is available.".to_string())?;
            Ok(ResolvedDevice {
                active_device: active,
                fallback_from: None,
            })
        }
    }
}

pub fn list_input_device_names(snapshot: &DeviceSnapshot) -> Vec<String> {
    dedupe_preserve_order(snapshot.input_devices.clone())
}

fn snapshot_from_device_names(input_devices: Vec<String>, default_input: Option<String>) -> DeviceSnapshot {
    let input_devices = dedupe_preserve_order(input_devices);
    let default_input = default_input.filter(|default_name| input_devices.iter().any(|name| name == default_name));
    DeviceSnapshot {
        input_devices,
        default_input,
    }
}

pub fn system_device_snapshot() -> DeviceSnapshot {
    #[cfg(test)]
    {
        return DeviceSnapshot {
            input_devices: vec!["Mock Microphone".to_string()],
            default_input: Some("Mock Microphone".to_string()),
        };
    }

    #[cfg(not(test))]
    {
        let host = cpal::default_host();

        let input_devices = host
            .input_devices()
            .map(|devices| {
                devices
                    .filter_map(|device| device.name().ok())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let default_input = host.default_input_device().and_then(|device| device.name().ok());

        return snapshot_from_device_names(input_devices, default_input);
    }

    #[allow(unreachable_code)]
    snapshot_from_device_names(Vec::new(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_mapping_dedupes_devices_and_preserves_order() {
        let snapshot = snapshot_from_device_names(
            vec![
                "Mic B".to_string(),
                "Mic A".to_string(),
                "Mic B".to_string(),
                "Mic C".to_string(),
            ],
            Some("Mic B".to_string()),
        );

        assert_eq!(
            snapshot.input_devices,
            vec!["Mic B".to_string(), "Mic A".to_string(), "Mic C".to_string()]
        );
        assert_eq!(snapshot.default_input, Some("Mic B".to_string()));
    }

    #[test]
    fn snapshot_mapping_drops_default_not_in_enumerated_inputs() {
        let snapshot = snapshot_from_device_names(
            vec!["Mic A".to_string(), "Mic B".to_string()],
            Some("External Mic".to_string()),
        );

        assert_eq!(
            snapshot.input_devices,
            vec!["Mic A".to_string(), "Mic B".to_string()]
        );
        assert_eq!(snapshot.default_input, None);
    }
}
