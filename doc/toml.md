## toml module

---

`toml.load_file(filename)`

Loads a toml file and returns it as a lua table

Parameters:

* filename (string) - File path to load.

Returns:

* Lua table with the content of the toml file

Example:

For the file /Users/myself/config.toml

```toml
[packages.sgrep]
branch = "main"
path = "/Users/myself/my-project"

[workspace]
name = "martelo"
version = "1"
```

```lua
local cfg = toml.load_file("/Users/myself/config.toml")
print(cfg["packages"]["sgrep"]["branch"])
-- prints main
```

---
`toml.save_file(filename, content)`

Saves a lua table as a toml file

Parameters:

* filename (string) - File path to save.
* content (table) - toml content to be saved

Example:

```lua
local content = { packages = { sgrep = { branch = "main" } } }
local cfg = toml.save_file("test-config.toml", content)
```

Generates the file

```toml
[packages.sgrep]
branch = "main"
```

