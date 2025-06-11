The `get_entry` function performs a hash based lookup of a LocaleTable to retrieve a value associated with a provided key.

## Usage
The `get_entry` function takes a pointer to a LocaleTable instance, a pointer to a UTF-8 encoded key string and the length of the UTF-8 key in bytes.
It then returns a FindEntryResult struct. If you are confused on where to get the pointer to a LocaleTable instance [take a look at this guide](get_locale_table.md).

### Header File
```c
#ifndef R3LOCALE_ENTRY_H
#define R3LOCALE_ENTRY_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h> // for size_t

/**
 * @brief Enum representing the result of attempting to look up an entry.
 */
typedef enum {
    FindEntryError_Normal,
    FindEntryError_NullTable,
    FindEntryError_NullKeyPtr,
    FindEntryError_NoEntryFound
} FindEntryError;

/**
 * @brief Struct representing the result of a key lookup in the LocaleTable.
 */
typedef struct {
    const unsigned char* value_ptr; ///< Pointer to the UTF-8 string value (not null-terminated)
    size_t value_len;               ///< Length of the value in bytes
    FindEntryError allocation_state;
} FindEntryResult;

/**
 * @brief Forward declaration of the LocaleTable type.
 */
typedef struct LocaleTable LocaleTable;

/**
 * @brief Looks up a key in the given LocaleTable and returns its corresponding value.
 *
 * @param table    Pointer to a previously loaded LocaleTable.
 * @param key_ptr  Pointer to the key byte array (UTF-8).
 * @param key_len  Length of the key in bytes.
 * @return FindEntryResult struct containing the value pointer and length.
 *
 * @note The returned value pointer is valid as long as the LocaleTable is not freed.
 *       The value is not null-terminated; use `value_len` for string handling.
 */
FindEntryResult get_entry(const LocaleTable* table, const unsigned char* key_ptr, size_t key_len);

#ifdef __cplusplus
}
#endif

#endif
```

### Main Function
```c
FindEntryResult entry = get_entry(result.table, (const unsigned char*)key_str, strlen(key_str));
if (entry.allocation_state == FindEntryError_Normal && entry.value_ptr != NULL) {
    //Process the pointer and string length, then store the processed value for later use
}
```

## FindEntryResult Struct
FindEntryResult represents the result of calling get_entry, which attempts to locate and return an entry in the provided LocaleTable instance associated with a provided key.

| Field              | Type                   | Description                                                                   |
|--------------------|------------------------|-------------------------------------------------------------------------------|
| `value_ptr`        | `const unsigned char*` | Pointer to the UTF-8 value if successful, otherwise returns a `NULL` pointer. |
| `value_len`        | `size_t`               | Length of the returned UTF-8 value in bytes.                                  |
| `allocation_state` | `FindEntryError`       | Status of the lookup                                                          |

### FindEntryError Enum Values
| Variant                       | Description                                                            |
|-------------------------------|------------------------------------------------------------------------|
| `FindEntryError_Normal`       | The entry was successfully found and returned.                         |
| `FindEntryError_NullTable`    | The provided `LocaleTable` pointer was `NULL`.                         |
| `FindEntryError_NullKeyPtr`   | The provided key argument was `NULL`.                                  |
| `FindEntryError_NoEntryFound` | The lookup completed but the specified key was not found in the table. |