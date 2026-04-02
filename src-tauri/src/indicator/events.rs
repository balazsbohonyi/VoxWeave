use serde::{Deserialize, Serialize};

pub const INDICATOR_STATE_EVENT: &str = "indicator-state";
pub const INDICATOR_HIDDEN_EVENT: &str = "indicator-hidden";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorVisualState {
    Recording,
    Processing,
    Injecting,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorStatePayload {
    pub state: IndicatorVisualState,
}

