//! Arduino library metadata models and CLI JSON output decoders.

/// A library entry from the Arduino Library Manager.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LibraryInfo {
    pub name: String,
    pub version: String,
    pub sentence: String,
}

/// Returns true when `candidate` is a newer semantic version than `current`.
pub fn version_is_newer(candidate: &str, current: &str) -> bool {
    let parse = |value: &str| -> Vec<u32> {
        value
            .split('.')
            .map(|part| part.parse::<u32>().unwrap_or(0))
            .collect()
    };
    let left = parse(candidate);
    let right = parse(current);
    for index in 0..left.len().max(right.len()) {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    false
}

/// Parses the JSON output of `arduino-cli lib search`.
pub fn parse_search_results(json: &str) -> Vec<LibraryInfo> {
    let mut out = Vec::new();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return out;
    };
    let Some(libraries) = value.get("libraries").and_then(|value| value.as_array()) else {
        return out;
    };

    for library in libraries {
        let name = library
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }

        let mut version = String::new();
        let mut sentence = String::new();
        if let Some(releases) = library.get("releases").and_then(|value| value.as_object()) {
            for (release_version, release) in releases {
                if version_is_newer(release_version, &version) {
                    version = release_version.clone();
                    sentence = release
                        .get("sentence")
                        .and_then(|value| value.as_str())
                        .unwrap_or("")
                        .to_string();
                }
            }
        }
        out.push(LibraryInfo {
            name,
            version,
            sentence,
        });
    }
    out
}

/// Parses the JSON output of `arduino-cli lib list`.
pub fn parse_installed_libraries(json: &str) -> Vec<LibraryInfo> {
    let mut out = Vec::new();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return out;
    };
    let Some(libraries) = value
        .get("installed_libraries")
        .and_then(|value| value.as_array())
    else {
        return out;
    };

    for item in libraries {
        let library = item.get("library").unwrap_or(item);
        let name = library
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        let version = library
            .get("version")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        out.push(LibraryInfo {
            name,
            version,
            sentence: String::new(),
        });
    }
    out
}
