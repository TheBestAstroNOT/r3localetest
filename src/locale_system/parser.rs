use std::fs;
use std::path::Path;
use std::collections::HashMap;
use super::simd::{get_bitmask, LANES};
use super::types::LocaleTable;

//Parses a reloaded 3 localisation file and returns a LocaleTable
pub fn parse_r3locale_file(path: Option<&Path>) -> LocaleTable{
    //Initialising all variables
     let bytes: Vec<u8> = match path {
         Some(p) => fs::read(p).expect(&format!("Unable to locate locale file: {}", p.display())),
         None => Vec::from(include_bytes!("../../src/bigexample.r3l") as &[u8]),
     };
    let mut opening_matches = Vec::new();
    let mut closing_matches = Vec::new();
    let mut i = 0;
    let bytes_len = bytes.len();
    let simd_end = bytes_len.saturating_sub(2 + LANES);
    let mut locale_map: HashMap<String, String> = HashMap::new();
    let mut last_close = 0;
    let mut last_key: Option<String> = None;

    //Simd search for keys
    while i <= simd_end {
        let opening_bitmask = get_bitmask(i, &bytes, [Some(b'['), Some(b'['), None]);
        for lane in 0..LANES {
            if (opening_bitmask & (1 << lane)) != 0 {
                if i + lane == 0 || bytes[i + lane - 1] == b'\n' {
                    opening_matches.push(i + lane);
                }
            }
        }

        let closing_bitmask = get_bitmask(i, &bytes, [Some(b']'), Some(b']'), None]);
        for lane in 0..LANES {
            if (closing_bitmask & (1 << lane)) != 0 {
                closing_matches.push(i + lane);
            }
        }

        i += LANES;
    }

    //Scalar search for keys
    while i < bytes_len {
        if bytes[i] == b'[' && i >= 1 && bytes[i - 1] == b'[' {
            if bytes[i - 2] == b'\n' {
                opening_matches.push(i - 1);
            }
        }

        if bytes[i] == b']' && i >= 1 && bytes[i - 1] == b']' {
            closing_matches.push(i - 1);
        }

        i += 1;
    }

    //Cleaning up collected indices
    opening_matches.sort_unstable();
    opening_matches.dedup();
    closing_matches.sort_unstable();
    closing_matches.dedup();

    //Safety check for matches
    if opening_matches.len() != closing_matches.len() {
        #[cfg(feature = "additional_locale_safety_checks")]
        {
            panic!("Opening and Closing bracket mismatch! Openings: {}, Closings: {}", opening_matches.len(), closing_matches.len());
        }
    }

    //Parsing values and keys, then adding them to a HashMap
    for (&open_pos, &close_pos) in opening_matches.iter().zip(closing_matches.iter()) {
        if open_pos < close_pos && close_pos + 2 <= bytes_len {
            if last_close != 0 {
                if let Some(ref key) = last_key {
                    let value = String::from_utf8_lossy(&bytes[last_close..open_pos])
                        .trim()
                        .to_string();
                    locale_map.insert(key.clone(), value);
                }
            }

            let key = String::from_utf8_lossy(&bytes[open_pos + 2..close_pos])
                .trim()
                .to_string();
            last_key = Some(key);
            last_close = close_pos + 2;
        } else {
            panic!(
                "Skipping invalid range: open={}, close={}, bytes_len={}",
                open_pos, close_pos, bytes_len
            );
        }
    }

    // Insert the final value after the last closing bracket (which can be at EOF)
    if let Some(ref key) = last_key {
        if last_close <= bytes_len {
            let value = String::from_utf8_lossy(&bytes[last_close..])
                .trim()
                .to_string();
            locale_map.insert(key.clone(), value);
        } else {
            locale_map.insert(key.clone(), String::new());
        }
    }

    LocaleTable{ entries: locale_map}
}

#[cfg(test)]
mod tests {
use super::*;
use std::path::Path;

    #[test]
    fn test_parsing_valid_file() {
        let path = Path::new("src/example.r3l");
        let result = parse_r3locale_file(Some(path));
        let mut expected = std::collections::HashMap::new();
        expected.insert("Bye".to_string(), "Bievenue".to_string());
        expected.insert("Key2".to_string(), "Value2".to_string());
        expected.insert("Hello".to_string(), "Bonjour".to_string());
        expected.insert(
            "Logs".to_string(),
            "Log entry 1\nLog entry 2\nLog entry 3".to_string(),
        );
        expected.insert(("Fin").to_string(), String::new());
        assert_eq!(result.entries, expected);
    }
}