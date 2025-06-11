## Usage
The `free_locale_table` function takes a pointer to a LocaleTable instance and free it from memory. If you are confused on where to get the pointer to a LocaleTable instance [take a look at this guide](get_locale_table.md).

### Header File
```c
#ifndef R3LOCALE_FREE_H
#define R3LOCALE_FREE_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Forward declaration of the LocaleTable struct.
 */
typedef struct LocaleTable LocaleTable;

/**
 * @brief Frees a LocaleTable previously allocated by get_locale_table.
 *
 * @param ptr Pointer to the LocaleTable to free. If NULL, this function does nothing.
 */
void free_locale_table(LocaleTable* ptr);

#ifdef __cplusplus
}
#endif

#endif
```

### Main Function
```c
free_locale_table(result.table); //result.table is a pointer to a LocaleTable instance.
```