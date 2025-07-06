## os module

Lua already has a built-in string module. LuSH extends it.

---

`string.split(text, separator, keep_empty)`

keep_empty defaults to true

Returns:

* Returns list of substrings of this string, separated by characters matched by a given separator.

Example:

```lua
string.split("a,b,c", ",", true)
-- Returns ["a","b","c"]
string.split("a,,c", ",", true)
-- Returns ["a","","c"]

string.split("a,b,c", ",", false)
-- Returns ["a","b","c"]
string.split("a,,c", ",", false)
-- Returns ["a","c"]
```

---
