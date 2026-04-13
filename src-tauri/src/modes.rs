use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DictationMode {
    General,
    Code,
    Command,
    Email,
}

impl Default for DictationMode {
    fn default() -> Self {
        Self::General
    }
}

pub fn mode_prompt(mode: DictationMode) -> &'static str {
    match mode {
        DictationMode::General => {
            "Format natural conversational text with correct punctuation and capitalization. Preserve meaning."
        }
        DictationMode::Code => {
            "Format as code-aware technical dictation. Use camelCase for variables and functions, PascalCase for components, preserve acronyms such as API, URL, UI, and SDK, and convert spoken punctuation."
        }
        DictationMode::Command => {
            "Format literally for terminal commands and config files. Convert spoken control words such as new line and tab. Do not add extra prose."
        }
        DictationMode::Email => {
            "Format as a clear email or message with appropriate structure, capitalization, and professional tone."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{mode_prompt, DictationMode};

    #[test]
    fn code_mode_prompt_preserves_acronyms() {
        let prompt = mode_prompt(DictationMode::Code);

        assert!(prompt.contains("camelCase"));
        assert!(prompt.contains("API"));
        assert!(prompt.contains("spoken punctuation"));
    }
}

