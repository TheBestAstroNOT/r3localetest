## Usage
The `parse_r3locale_bytes` function is a rust only function and doesn't support C interop. It takes a sequence of bytes `&[u8]` of the file you want to parse (must be pre-sanitised) and returns a Result<LocaleTable, ParseR3Error>.

### Main Function Usage

```rust
match parse_r3locale_bytes(input) {
    Ok(table) => {
        let value = table.get("title").unwrap();
        println!("Parsed title: {}", value);
    }
    Err(err) => {
        eprintln!("Failed to parse: {}", err);
    }
}
```

### ParseR3Error Enum Values
| Variant                         | Description                                                                |
|---------------------------------|----------------------------------------------------------------------------|
| `ParseR3Error_Normal`           | The operation completed successfully.                                      |
| `ParseR3Error_FileNotFound`     | The specified file could not be found.                                     |
| `ParseR3Error_FailedToRead`     | Failed to read the file from disk.                                         |
| `ParseR3Error_KeyValueMismatch` | Mismatch in number of keys and values while parsing the localisation file. |
| `ParseR3Error_BracketMismatch`  | Detected invalid bracket structure in the localisation file.               |
| `ParseR3Error_InvalidUTF8Value` | A string value in the localisation file was not valid UTF-8.               |
| `ParseR3Error_InvalidUTF8Path`  | The file path provided could not be parsed as valid UTF-8.                 |
| `ParseR3Error_NullPathProvided` | The input path pointer was `NULL`.                                         |