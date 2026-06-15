//! Rhai sandbox API exposed to plugin scripts.
//!
//! Scripts call `memo::read_u8(offset)`, `cell_string("value")`, `section(...)`, etc.
//! The memo bytes are stored in a global `Mutex` before calling `process_memo` so the
//! `memo::*` functions can read from them implicitly.

use rhai::{Dynamic, Engine, EvalAltResult, Module};
use std::sync::{LazyLock, Mutex};

/// Global memo payload for the currently-executing script.
static CURRENT_MEMO: LazyLock<Mutex<Vec<u8>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Set the memo bytes, execute a closure, then clear the bytes.
pub fn with_memo_bytes<F, R>(bytes: &[u8], f: F) -> R
where
    F: FnOnce() -> R,
{
    {
        let mut guard = CURRENT_MEMO.lock().unwrap();
        *guard = bytes.to_vec();
    }
    let result = f();
    {
        let mut guard = CURRENT_MEMO.lock().unwrap();
        guard.clear();
    }
    result
}

// ── memo module functions (fallible for Module::set_native_fn) ─────────

type RhaiResult<T> = Result<T, Box<EvalAltResult>>;

fn memo_len() -> RhaiResult<i64> {
    Ok(CURRENT_MEMO.lock().unwrap().len() as i64)
}

fn memo_read_u8(offset: i64) -> RhaiResult<i64> {
    let idx = offset as usize;
    let guard = CURRENT_MEMO.lock().unwrap();
    Ok(if idx < guard.len() {
        guard[idx] as i64
    } else {
        0
    })
}

fn memo_read_u16_le(offset: i64) -> RhaiResult<i64> {
    let idx = offset as usize;
    let guard = CURRENT_MEMO.lock().unwrap();
    Ok(if idx + 1 < guard.len() {
        u16::from_le_bytes([guard[idx], guard[idx + 1]]) as i64
    } else {
        0
    })
}

fn memo_read_u32_le(offset: i64) -> RhaiResult<i64> {
    let idx = offset as usize;
    let guard = CURRENT_MEMO.lock().unwrap();
    Ok(if idx + 3 < guard.len() {
        u32::from_le_bytes([
            guard[idx],
            guard[idx + 1],
            guard[idx + 2],
            guard[idx + 3],
        ]) as i64
    } else {
        0
    })
}

fn memo_read_u64_le(offset: i64) -> RhaiResult<i64> {
    let idx = offset as usize;
    let guard = CURRENT_MEMO.lock().unwrap();
    Ok(if idx + 7 < guard.len() {
        u64::from_le_bytes([
            guard[idx],
            guard[idx + 1],
            guard[idx + 2],
            guard[idx + 3],
            guard[idx + 4],
            guard[idx + 5],
            guard[idx + 6],
            guard[idx + 7],
        ]) as i64
    } else {
        0
    })
}

fn memo_read_bytes(offset: i64, len: i64) -> RhaiResult<rhai::Blob> {
    let idx = offset.max(0) as usize;
    let n = len.max(0) as usize;
    let guard = CURRENT_MEMO.lock().unwrap();
    let end = (idx + n).min(guard.len());
    Ok(guard[idx..end].to_vec())
}

fn memo_read_string(offset: i64, len: i64) -> RhaiResult<String> {
    let idx = offset.max(0) as usize;
    let n = len.max(0) as usize;
    let guard = CURRENT_MEMO.lock().unwrap();
    let end = (idx + n).min(guard.len());
    let slice = &guard[idx..end];
    // Trim trailing zeros
    let slice = match slice.iter().position(|&b| b == 0) {
        Some(zero_pos) => &slice[..zero_pos],
        None => slice,
    };
    Ok(String::from_utf8_lossy(slice)
        .trim_end_matches('\0')
        .to_string())
}

// ── Constructor functions (global, non-fallible) ───────────────────────

fn cell_number(value: i64) -> Dynamic {
    let mut map = rhai::Map::new();
    map.insert("cell_type".into(), Dynamic::from("number"));
    map.insert("value".into(), Dynamic::from(value.to_string()));
    Dynamic::from(map)
}

fn cell_string(value: &str) -> Dynamic {
    let mut map = rhai::Map::new();
    map.insert("cell_type".into(), Dynamic::from("string"));
    map.insert("value".into(), Dynamic::from(value.to_string()));
    Dynamic::from(map)
}

fn cell_date(timestamp: i64) -> Dynamic {
    let mut map = rhai::Map::new();
    map.insert("cell_type".into(), Dynamic::from("date"));
    map.insert("value".into(), Dynamic::from(timestamp.to_string()));
    Dynamic::from(map)
}

fn cell_url(value: &str) -> Dynamic {
    let mut map = rhai::Map::new();
    map.insert("cell_type".into(), Dynamic::from("url"));
    map.insert("value".into(), Dynamic::from(value.to_string()));
    Dynamic::from(map)
}

fn section(title: &str, headers: Dynamic, rows: Dynamic) -> Dynamic {
    let mut map = rhai::Map::new();
    map.insert("title".into(), Dynamic::from(title.to_string()));
    map.insert("headers".into(), headers);
    map.insert("rows".into(), rows);
    Dynamic::from(map)
}

// ── Engine setup ───────────────────────────────────────────────────────

/// Create a sandboxed rhai `Engine` with only the memo-parsing API available.
pub fn create_sandboxed_engine() -> Engine {
    let mut engine = Engine::new();

    engine.set_max_operations(100_000);
    engine.set_max_call_levels(32);
    engine.set_max_string_size(65536);
    engine.set_allow_looping(false);
    engine.disable_symbol("eval");
    engine.disable_symbol("Fn");
    engine.disable_symbol("call");
    engine.disable_symbol("import");

    // Register memo namespace module (memo::read_u8, etc.)
    let mut memo_module = Module::new();
    memo_module.set_native_fn("len", memo_len);
    memo_module.set_native_fn("read_u8", memo_read_u8);
    memo_module.set_native_fn("read_u16_le", memo_read_u16_le);
    memo_module.set_native_fn("read_u32_le", memo_read_u32_le);
    memo_module.set_native_fn("read_u64_le", memo_read_u64_le);
    memo_module.set_native_fn("read_bytes", memo_read_bytes);
    memo_module.set_native_fn("read_string", memo_read_string);
    engine.register_static_module("memo", memo_module.into());

    // Register section/cell constructors (global)
    engine.register_fn("section", section as fn(&str, Dynamic, Dynamic) -> Dynamic);
    engine.register_fn("cell_number", cell_number as fn(i64) -> Dynamic);
    engine.register_fn("cell_string", cell_string as fn(&str) -> Dynamic);
    engine.register_fn("cell_date", cell_date as fn(i64) -> Dynamic);
    engine.register_fn("cell_url", cell_url as fn(&str) -> Dynamic);

    engine
}

// ── Helpers to extract results from script calls ───────────────────────

/// Extract `Vec<String>` from a rhai Dynamic returned by `get_prefixes()`.
pub fn extract_prefixes(result: Dynamic) -> Vec<String> {
    let json = serde_json::to_value(&result).unwrap_or_default();
    json.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedMemoSection {
    pub title: String,
    pub headers: Vec<String>,
    /// Each row is a direct array of cells (no wrapper object).
    pub rows: Vec<Vec<ParsedMemoCell>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedMemoCell {
    #[serde(rename = "cell_type")]
    pub cell_type: String,
    pub value: String,
}

/// Extract `Vec<ParsedMemoSection>` from a rhai Dynamic returned by `process_memo()`.
pub fn extract_sections(result: Dynamic) -> Vec<ParsedMemoSection> {
    let json = serde_json::to_value(&result).unwrap_or_default();
    serde_json::from_value(json).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_sandbox_no_eval() {
        let engine = create_sandboxed_engine();
        let result = engine
            .compile(r#"eval("1+1")"#)
            .map(|_| ())
            .err()
            .map(|e| e.to_string());
        assert!(result.is_some(), "eval should be disabled in sandbox");
    }

    #[test]
    fn test_memo_functions_out_of_bounds() {
        let engine = create_sandboxed_engine();
        let ast = engine
            .compile("fn t() { memo::read_u8(1000) } t()")
            .unwrap();
        with_memo_bytes(&[0x01, 0x02], || {
            let result: i64 = engine.eval_ast(&ast).unwrap();
            assert_eq!(result, 0, "OOB read should return 0");
        });
    }

    #[test]
    fn test_memo_functions_in_bounds() {
        let engine = create_sandboxed_engine();
        let data = vec![0xAA, 0xBB, 0x01, 0x00, 0x00, 0x00];
        let ast = engine
            .compile("fn t() { [memo::read_u8(0), memo::read_u32_le(2)] } t()")
            .unwrap();
        with_memo_bytes(&data, || {
            let result: Dynamic = engine.eval_ast(&ast).unwrap();
            let json = serde_json::to_value(&result).unwrap();
            let arr = json.as_array().unwrap();
            assert_eq!(arr[0].as_i64().unwrap(), 0xAA);
            assert_eq!(arr[1].as_i64().unwrap(), 1);
        });
    }

    #[test]
    fn test_extract_prefixes() {
        let engine = create_sandboxed_engine();
        let ast = engine
            .compile(r#"fn t() { ["deadbeef", "cafebabe"] } t()"#)
            .unwrap();
        let result: Dynamic = engine.eval_ast(&ast).unwrap();
        let prefixes = extract_prefixes(result);
        assert_eq!(prefixes, vec!["deadbeef", "cafebabe"]);
    }

    #[test]
    fn test_cell_and_section() {
        let engine = create_sandboxed_engine();
        let script = r#"
            fn t() {
                let headers = ["Field", "Value"];
                let rows = [
                    [cell_string("Version"), cell_number(1)],
                    [cell_string("Amount"), cell_number(1000)],
                ];
                return [section("Test", headers, rows)];
            }
            t()
        "#;
        let ast = engine.compile(script).unwrap();
        let result: Dynamic = engine.eval_ast(&ast).unwrap();
        let sections = extract_sections(result);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Test");
        assert_eq!(sections[0].headers, vec!["Field", "Value"]);
        assert_eq!(sections[0].rows.len(), 2);
        assert_eq!(sections[0].rows[0][0].cell_type, "string");
        assert_eq!(sections[0].rows[0][0].value, "Version");
        assert_eq!(sections[0].rows[0][1].cell_type, "number");
        assert_eq!(sections[0].rows[0][1].value, "1");
    }

    #[test]
    fn test_process_memo_with_two_prefixes() {
        let engine = create_sandboxed_engine();
        let data = vec![0xde, 0xad, 0xbe, 0xef, 0x01];
        let script = r#"
            fn get_prefixes() { return ["deadbeef"]; }
            fn process_memo() {
                let version = memo::read_u8(4);
                let headers = ["Version"];
                let rows = [[cell_number(version)]];
                return [section("Test", headers, rows)];
            }
            process_memo()
        "#;
        let ast = engine.compile(script).unwrap();
        with_memo_bytes(&data, || {
            let result: Dynamic = engine.eval_ast(&ast).unwrap();
            let sections = extract_sections(result);
            assert_eq!(sections.len(), 1);
            assert_eq!(sections[0].rows[0][0].value, "1");
        });
    }

    #[test]
    fn test_read_string_with_null_termination() {
        let engine = create_sandboxed_engine();
        let mut data = vec![0u8; 32];
        data[0..5].copy_from_slice(b"hello");
        data[5] = 0;
        let ast = engine
            .compile("fn t() { memo::read_string(0, 32) } t()")
            .unwrap();
        with_memo_bytes(&data, || {
            let result: String = engine.eval_ast(&ast).unwrap();
            assert_eq!(result, "hello");
        });
    }

    #[test]
    fn test_dkg_dk00_plugin() {
        let engine = create_sandboxed_engine();
        // Build a DK00 memo: prefix + from_id(1) + data_len(32u64 LE) + 32 zero bytes
        let mut data = vec![0u8; 45];
        data[0..4].copy_from_slice(b"DK00");        // prefix
        data[4] = 1;                                 // from_id = 1
        data[5..13].copy_from_slice(&32u64.to_le_bytes()); // data_len = 32
        // data[13..45] = zeros (VerifyingKey placeholder)

        let script = r#"
            fn get_prefixes() { return ["444b3030"]; }
            fn process_memo() {
                let from_id = memo::read_u8(4);
                let data_len = memo::read_u64_le(5);
                let headers = ["Field", "Value"];
                let rows = [
                    [cell_string("Round"), cell_string("DKG Round 0")],
                    [cell_string("From ID"), cell_number(from_id)],
                    [cell_string("Data Size"), cell_number(data_len)],
                ];
                return [section("DKG Message", headers, rows)];
            }
            process_memo()
        "#;
        let ast = engine.compile(script).unwrap();
        with_memo_bytes(&data, || {
            let result: Dynamic = engine.eval_ast(&ast).unwrap();
            let sections = extract_sections(result);
            assert_eq!(sections.len(), 1);
            assert_eq!(sections[0].title, "DKG Message");
            assert_eq!(sections[0].rows[0][1].value, "DKG Round 0");
            assert_eq!(sections[0].rows[1][1].value, "1");       // from_id
            assert_eq!(sections[0].rows[2][1].value, "32");      // data_len
        });
    }
}
