## os module

Lua already has a built-in os module. LuSH extends it.

---

`os.name()`

Returns the name of the operating system the program is running on.

Possible return values include:

- linux
- macos
- ios
- freebsd
- dragonfly
- netbsd
- openbsd
- solaris
- android
- windows

Returns:

* A string containing the name of the operating system.

Example:

```lua
print("Operating System:", os.name())
```

---

`os.proc_names()`

Returns a table containing running process names

Returns:

* A table { pid: process_name }

Example:

```lua
os.proc_names()
-- Returns
{
    1121="watchdogd",
    80574="periodic-wrapper",
    1309="distnoted"
}
```

---

`os.proc_exes()`

Returns a table containing running process executables with path

Returns:

* A table { pid: process_name }

Example:

```lua
os.proc_exes()
-- Returns
{
    1121="/usr/libexec/watchdogd",
    80574="/usr/libexec/periodic-wrapper",
    1309="/usr/sbin/distnoted"
}
```

---

`os.pipe_exec()`

Executes one or more commands in parallel, piping the result into the next command.
The final result will be written to stdout

Returns:

* Nothing

Example:

```lua
-- content of ~/my-file.log:
-- error 1 - asd
-- this worked ok
-- error 2 - asd

function in_brackets(x)
    return '[' .. x .. ']'
end

function only_errors(x)
    if string.find(x, 'error') then
        return x
    end
end

os.pipe_exec({
    {"cat", "/tmp/my-file.log"},
    {in_brackets},
    {"grep", "error"},
})

-- If it's a function or a command with no arguments, crurly braces is optional
os.pipe_exec({
    {"cat", "/tmp/my-file.log"},
    in_brackets,
    {only_errors},
})

-- both print to stdout 
-- [error 1 - asd]
-- [error 2 - asd]
```

---

`os.pipeline()`

Executes one or more commands in parallel, piping the result into the next command.
Different than os.pipe_exec, the result is returned 

Returns:

* Output of the last command in the pipe

Example:

```lua
-- content of ~/my-file.log:
-- error 1 - asd
-- this worked ok
-- error 2 - asd

function in_brackets(x)
    return '[' .. x .. ']'
end

function only_errors(x)
    if string.find(x, 'error') then
        return x
    end
end

-- If it's a function or a command with no arguments, crurly braces is optional
local res = os.pipeline({
    {"cat", "/tmp/my-file.log"},
    {in_brackets},
    only_errors,
})

print(res)
```

