## preprocessors

Some syntax sugars are added as a preprocessor

---

String interpolation is supported for strings using double quotes only (`"`)

```lua
local name = "Thiago"
print("My name is ${name}")
```

This is a syntax sugar for

```lua
print("My name is " .. tostring(name))
```

In a more complete example, this code:

```lua
local a = 1
local b = true
local c = 'hello'

print("a: " .. a .. " b: " .. tostring(b) .. " c: " .. tostring(c) .. " d: " .. tostring(d))
print("a: ${a} b: ${b} c: ${c} d: ${d}")
print('a: ${a} b: ${b} c: ${c} d: ${d}')
```

will print

```
a: 1 b: true c: hello d: nil
a: 1 b: true c: hello d: nil
a: ${a} b: ${b} c: ${c} d: ${d}
```

---
