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

### Shell exec

To simplify execution of shell command, if a line starts with `$>`, LuSH replaces the command with os.pipe_exec.

**Important:** Lua functions need to be surrounded by back quotes "`" 

E.g.

```bash 
$> tail /tmp/my-file.log | `in_brackets` | grep "error"
```

Generates:

```lua
os.pipe_exec(
  {"tail", "/tmp/my-file.log"},
  {in_brackets},
  {"grep", "error"}
)

```

---

### Subshell exec

To simplify execution of subshell command, if this is found in a script `$()`, LuSH replaces the command with os.pipeline.

**Important:** Lua functions need to be surrounded by back quotes "`"

E.g.

```bash 
local x = $(cat /tmp/my-file.log | `in_brackets` | grep "error")
```

Generates:

```lua
local x = os.pipeline(
  {"cat", "/tmp/my-file.log"},
  {in_brackets},
  {only_errors}
)
```

