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

pub fn system_device_snapshot() -> DeviceSnapshot {
    #[cfg(test)]
    {
        return DeviceSnapshot {
            input_devices: vec!["Mock Microphone".to_string()],
            default_input: Some("Mock Microphone".to_string()),
        };
    }

    #[allow(unreachable_code)]
    DeviceSnapshot {
        input_devices: Vec::new(),
        default_input: None,
    }
}

