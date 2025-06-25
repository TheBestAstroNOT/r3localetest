## Usage
The `merge_locale_table_c` merges existing LocaleTable objects into a single LocaleTable object. It takes an array of LocaleTable pointers `const LocaleTable** tables` and the number of table pointers in the array as `size_t`. It returns `MergeResult`.

!!! warning
    In case of duplicate keys, the earlier tables in the tables array will have priority. So for example if the default language is English but the user preference is Spanish, then add the Spanish table to the array first.

### Header File
```c
#ifndef R3LOCALE_MERGE_H
#define R3LOCALE_MERGE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

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
    MergeTableError merge_state;
} MergeResult;

MergeResult merge_locale_table_c(const LocaleTable** tables, size_t count);

#ifdef __cplusplus
}
#endif

#endif
```

### Main Function
```c
AllocationResult table1 = get_locale_table("example_path_1");
AllocationResult table2 = get_locale_table("example_path_2");

const LocaleTable* tables[] = { table1.table, table2.table };
MergeResult merged = merge_locale_table_c(tables, 2);
```

## MergeResult Struct
This represents the result of calling `merge_locale_table_c`.

| Field         | Type              | Description                                                            |
|---------------|-------------------|------------------------------------------------------------------------|
| `table`       | `LocaleTable*`    | Pointer to the merged `LocaleTable` if successful, otherwise `NULL`.   |
| `merge_state` | `MergeTableError` | Status of the merge operation, indicating success or specific failure. |


### MergeTableError Enum Values
Most of the values are the same as [ParseR3Error](parse_r3locale_bytes.md#parser3error-enum-values). But there is one more value.

| Variant                            | Description                        |
|------------------------------------|------------------------------------|
| `MergeTableError_NullTablePointer` | A null table pointer was provided. |