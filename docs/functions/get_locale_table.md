!!! info
    All other functions in the API require an instance of the LocaleTable object to be passed to them, so this needs to be the first function you call when using the API.


## Usage
The `get_locale_table` function takes a `const char*` representing the path to the localisation file you want to load.

!!! warning
    The returned LocaleTable must be freed with `free_locale_table` when it's no longer in use.

### Header File
```c
#ifndef R3LOCALE_GET_H
#define R3LOCALE_GET_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h> // for size_t

/**
 * @brief Enum representing possible outcomes of parsing or allocating a locale table.
 */
typedef enum {
    ParseR3Error_Normal,
    ParseR3Error_FileNotFound,
    ParseR3Error_FailedToRead,
    ParseR3Error_KeyValueMismatch,
    ParseR3Error_BracketMismatch,
    ParseR3Error_InvalidUTF8Value,
    ParseR3Error_InvalidUTF8Path,
    ParseR3Error_NullPathProvided
} ParseR3Error;

/**
 * @brief Forward declaration of the opaque LocaleTable struct.
 */
typedef struct LocaleTable LocaleTable;

/**
 * @brief Struct representing the result of loading a locale table.
 */
typedef struct {
    LocaleTable* table;         ///< Pointer to the loaded LocaleTable (NULL if failed)
    ParseR3Error allocation_state; ///< Status of the operation
} AllocationResult;

/**
 * @brief Loads an `.r3locale` file and returns a parsed LocaleTable object.
 *
 * @param path A null-terminated UTF-8 string representing the file path.
 * @return AllocationResult
 *         - `table`: pointer to the allocated LocaleTable (must be freed later)
 *         - `allocation_state`: result of the parsing/allocation process
 *
 * @note Must call `free_locale_table` to deallocate the returned table when done.
 */
AllocationResult get_locale_table(const char* path);

#ifdef __cplusplus
}
#endif

#endif
```

### Main Function
```c
const char* file_path = "example.r3locale"; //Replace with actual path to the localisation file you want to load
AllocationResult result = get_locale_table(file_path);
if (result.allocation_state == ParseR3Error_Normal && result.table != NULL){
 //Store result.table somewhere for later use with other functions.
}
```

## AllocationResult Struct
AllocationResult represents the result of calling get_locale_table, which attempts to parse and allocate memory for a Reloaded-3 localisation file.

| Field              | Type           | Description                                                                                  |
|--------------------|----------------|----------------------------------------------------------------------------------------------|
| `table`            | `LocaleTable*` | Pointer to the allocated LocaleTable object if successful, otherwise returns a NULL pointer. |
| `allocation_state` | `ParseR3Error` | Enum indicating the result of the allocation or parsing process.                             |

### LocaleTable
A LocaleTable holds the key-value string data extracted from a localisation file.
It's fields are not accessible from C, but are used by the Rust implementation to provide fast lookups with low overhead.

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
| `ParseR3Error::DuplicateKeys`   | A key is detected more than one time.                                      |

## In case of multiple locale files
You can call `get_multiple_locale_tables` which takes an array of locale table paths (earlier tables have priority) and the number of table paths. It returns a [MergeResult](merge_locale_table_c.md#mergeresult-struct).

### Header File
```c
#ifndef R3LOCALE_GET_H
#define R3LOCALE_GET_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h> // for size_t

typedef enum {
    ParseR3Error_Normal,
    ParseR3Error_FileNotFound,
    ParseR3Error_FailedToRead,
    ParseR3Error_KeyValueMismatch,
    ParseR3Error_BracketMismatch,
    ParseR3Error_InvalidUTF8Value,
    ParseR3Error_InvalidUTF8Path,
    ParseR3Error_NullPathProvided
} ParseR3Error;

typedef enum {
    MergeTableError_Normal,
    MergeTableError_NullTablePointer,
    MergeTableError_FileNotFound,
    MergeTableError_FailedToRead,
    MergeTableError_KeyValueMismatch,
    MergeTableError_BracketMismatch,
    MergeTableError_InvalidUTF8Value,
    MergeTableError_InvalidUTF8Path,
    MergeTableError_NullPathProvided,
    MergeTableError_DuplicateKeys
} MergeTableError;

typedef struct LocaleTable LocaleTable;

typedef struct {
    LocaleTable* table;
    ParseR3Error allocation_state;
} AllocationResult;

typedef struct {
    LocaleTable* table;
    MergeTableError merge_state;
} MergeResult;

AllocationResult get_locale_table(const char* path);

MergeResult get_multiple_locale_tables(const char* const* paths, size_t count);

#ifdef __cplusplus
}
#endif

#endif
```

### Main Function
const char* locale_paths[] = {
"example_path_1",
"example_path_2"
};

const size_t LOCALE_COUNT = 2;

MergeResult merged = get_multiple_locale_tables(locale_paths, LOCALE_COUNT);
