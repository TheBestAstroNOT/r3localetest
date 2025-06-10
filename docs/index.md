---
hide:
  - toc
---

<div align="center">
	<h1>The Reloaded-3 Localisation API</h1>
	<img src="Reloaded/Images/Reloaded-Icon.png"/>
	<br/> <br/>
    A guide on how to use R3's locale api (hopefully).
    <br/>
</div>

## About

Reloaded3.Localisation is a high performance API to easily parse R3 locale files and access their values in C or Rust!

!!! warning

    All locale files must only use UTF-8 characters, using non-UTF-8 characters will result in a default LocaleTable being returned with no values.

## Used Methods

| Method              | Description                                                                          |
|---------------------|--------------------------------------------------------------------------------------|
| `get_locale_table`  | Used to get an instance of the LocaleTable object to be used in all other API calls. |
| `get_entry`         | Used to fetch a value from the LocaleTable                                           |
| `free_locale_table` | Used to free the LocaleTable instance from memory.                                   |