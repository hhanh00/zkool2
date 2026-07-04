# Plugins

- Use the rhai.rs crate
- Plugins have different types based on the API they support
- A plugin may support more than one type
- A introspection should be considered
- Currently only the memo interface

## Memo Interface

given a memo type -> List of memo sections
- plugin register the prefix (the first 4 bytes of the memo binary)
- a memo section is a table (title + rows)
- columns have headers
- type supported: number, string, date
- the app calls a memo plugin based on prefix

## Management

- Plugins can be downloaded and installed via
plugin manager page (in settings)
- Plugins can be disabled and removed
- Installing plugins is at the risk of the user
