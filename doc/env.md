## env module

These functions are part of the global environment (`env`) in Lua, providing utilities for directory management,
environment variables, and output printing.

---

`env.cwd()`

Retrieves the current directory

Returns:

* The current directory or nil if not available

Example:

```lua
env.print(env.cwd())
/Users/myself/src/lush
```

---

`env.pushd(new_dir)`

Changes the current working directory and pushes the previous one onto the stack.

Parameters:

* new_dir (string) - The new directory path to switch to.

Returns:

* Nothing on success. Raises an error if the directory change fails.

Example:

```lua
env.pushd("/path/to/directory")
```

---

`env.popd()`

Pops the top directory from the stack and changes to it.

Parameters:

* None.

Returns:

* Nothing on success. Raises an error if the directory stack is empty or the change fails.

Example:

```lua
env.popd()
```

---

`env.chdir(new_dir)`

Changes the current working directory.

Parameters:

* new_dir (string) - The new directory to switch to.

Returns:

* Nothing on success. Raises an error if the directory change fails.

Example:

```lua
env.chdir("/another/directory")
```

---

`env.pwd()`

Returns the current working directory as a string.

Parameters:

* None.

Returns:

* A string representing the current directory.

Example:

```lua
print(env.pwd())
```

---

`env.set(name, value)`

Sets an environment variable.

Parameters:

* name (string) - The name of the environment variable.
* value (string) - The value to set.

Returns:

* Nothing.

Example:

```lua
env.set("MY_VAR", "some_value")
```

---

`env.get(name)`

Gets the value of an environment variable.

Parameters:

* name (string) - The name of the environment variable.

Returns:

* The value of the environment variable as a string, or `nil` if it does not exist.

Example:

```lua
local value = env.get("MY_VAR")
print(value)
```

---

`env.del(name)`

Removes an environment variable.

Parameters:

* name (string) - The name of the environment variable to remove.

Returns:

* Nothing.

Example:

```lua
env.del("MY_VAR")
```

---

`env.print(...)`

Prints the provided tokens to the standard output.

Parameters:

* A variadic number of arguments to print.
* If the first argument is a string and the second a table, it formats by name or index

Returns:

* Nothing.

Example:

```lua
env.print("Hello", "World", 123)
env.print('ten={}, twenty={}, my name={name}', {10, 20, name='Thiago'}) 
```
