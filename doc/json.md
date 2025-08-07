## json module

---

`json.load_file(filename)`

Loads a json file and returns it as a lua table

Parameters:

* filename (string) - File path to load.

Returns:

* Lua table with the content of the json file

Example:

For the file /Users/myself/config.json

```json
{
  "name": "Scarlett Johansson",
  "age": 38,
  "occupation": "Actress",
  "hair_colour": {
    "blonde": true,
    "red": true
  }
}
```

```lua
local s = json.load_file('/tmp/file.json')
env.print(s['hair_colour']["blonde"])
env.print(s['hair_colour']["red"])
s['hair_colour']["blue"] = false
env.print(s['hair_colour']["blue"])
json.save_file('/tmp/file.json', s)
```

---

`json.from_string(content)`

Loads a json from a string and returns it as a lua table

Parameters:

* content (string) - Content of the json.

Returns:

* Lua table with the content of the json file

Example:

```lua
local cont = [[{
  "name": "Scarlett Johansson",
  "age": 38,
  "occupation": "Actress",
  "hair_colour": {
    "blonde": true,
    "red": true
  }
}]]

local s = json.from_string(cont)
env.print(s['hair_colour']["blonde"])
env.print(s['hair_colour']["red"])
```

---
`json.save_file(filename, content)`

Saves a lua table as a json file

Parameters:

* filename (string) - File path to save.
* content (table) - json content to be saved

Example:

```lua
local content = { packages = { sgrep = { branch = "main" } } }
local cfg = json.save_file("test-config.json", content)
```

Generates the file

```json
{
  "packages": {
    "sgrep": {
      "branch": "main"
    }
  }
}
```
