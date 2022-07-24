use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct LoggingRequestInfo {
    #[serde(rename="ContainerID")]
    pub container_id: String,
    pub config:              Option<HashMap<String, String>>,
    pub container_name:       Option<String>,
    pub container_entrypoint: Option<String>,
    pub container_args:       Option<Vec<String>>,
    #[serde(rename="ContainerImageID")]
    pub container_image_id:    Option<String>,
    pub container_image_name:  Option<String>,
    pub container_created:       Option<String>,
    pub container_env:        Option<Vec<String>>,
    pub container_labels:     Option<HashMap<String, String>>,
    pub log_path:             Option<String>,
    pub daemon_name:          Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StartLoggingRequest {
    pub file: String,
    pub info: LoggingRequestInfo,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StopLoggingRequest {
    pub file: String,
}
