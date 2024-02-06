#![no_main]

use libfuzzer_sys::fuzz_target;
use rew::pattern::Pattern;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let mut chars = text.chars();
        let _ignore_result = if let Some(escape) = chars.next() {
            Pattern::parse(chars.as_str(), escape)
        } else {
            Pattern::parse(text, '\\')
        };
    }
});
