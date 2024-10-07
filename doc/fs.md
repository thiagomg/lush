## fs module

fs Global Functions

The `fs` global provides utilities for file and directory management, including listing directories, creating and
removing directories, copying and moving files, and checking file existence.

`fs.ls(paths)`

Lists the contents of a directory. If no path is provided, it lists the contents of the current working directory.

Parameters:

* paths (string) - Optional. A path to the directory to list. If omitted, the current directory is used.

Returns:

* A table of strings representing file paths within the directory.

Example:

```lua
local files = fs.ls("/path/to/directory")
for _, file in ipairs(files) do
    print(file)
end
```

---

`fs.mkdir(path)`

Creates a directory at the specified path. If the parent directories don't exist, it creates them as well.

Parameters:

* path (string) - The directory path to create.

Returns:

* Nothing.

Example:

```lua
fs.mkdir("/new/directory")
```

---

`fs.rmdir(path, options)`

Removes a directory at the specified path. You can optionally specify whether to delete the directory recursively.

Parameters:

* path (string) - The directory path to remove.
* options (table) - Optional. A table with the key `recursive` (boolean). If `true`, the directory and all its contents
  are deleted.

Returns:

* Nothing.

Example:

```lua
fs.rmdir("/some/directory", { recursive = true })
```

---

`fs.copy(src, target)`

Copies a file from the source path to the target path. If the target is a directory, the file is copied into the
directory with its original name.

Parameters:

* src (string) - The source file path.
* target (string) - The target file or directory path.

Returns:

* Nothing.

Example:

```lua
fs.copy("/path/to/source", "/path/to/destination")
```

---

`fs.move(src, target)`

Moves a file from the source path to the target path. If the target is a directory, the file is moved into the directory
with its original name.

Parameters:

* src (string) - The source file path.
* target (string) - The target file or directory path.

Returns:

* Nothing.

Example:

```lua
fs.move("/path/to/source", "/path/to/destination")
```

---

`fs.exists(src)`

Checks if a file exists at the specified path.

Parameters:

* src (string) - The file path to check.

Returns:

* `true` if the file exists, `false` otherwise.

Example:

```lua
local exists = fs.exists("/path/to/file")
print(exists)
```
