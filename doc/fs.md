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

* bool: true if directory was deleted

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

`fs.rm(path, options)`

Removes a file at the specified path. You can optionally specify whether to delete recursively, if a directory.

Parameters:

* path (string) - The directory path to remove.
* options (table) - Optional. A table with the key `recursive` (boolean). If `true`, the directory and all its contents
  are deleted.

Returns:

* Nothing.

Example:

```lua
fs.rm("/some/file")
fs.rm("/some/directory", { recursive = true })
```

---

`fs.exists(src)`

Checks if a file or a directory exist at the specified path.

Parameters:

* src (string) - The path to check.

Returns:

* `true` if the file exists, `false` otherwise.

Example:

```lua
local exists = fs.exists("/path/to/file")
print(exists)
```

---

`fs.is_dir(src)`

Checks if a path is a directory

Parameters:

* src (string) - The path to check.

Returns:

* `true` if the path is a directory, `false` otherwise.

Example:

```lua
local res = fs.is_dir("/path/to/my-dir")
print(res)
```

---

`fs.is_file(src)`

Checks if a path is a file

Parameters:

* src (string) - The path to check.

Returns:

* `true` if the path is a file, `false` otherwise.

Example:

```lua
local res = fs.is_file("/path/to/my-file")
print(res)
```

---

`fs.parent(src)`

Retrieves the parent path of a given path

Parameters:

* src (string) - The path to check.

Returns:

* Parent path of given path or Nil if not available

Example:

```lua
local parent_dir = fs.parent("/path/to/my-dir")
print(parent_dir) -- prints /path/to
```

---

`fs.read_file(file_path)`

Reads a file from the given path and returns the content as string

Parameters:

* file_path (string) - Path of the file to be read

Returns:

* content as string or nil if the file does not exist

Example:

```lua
local content = fs.read_file('/tmp/my-file.txt')
env.print(content)
```

---

`fs.write_file(file_path, content)`

Writes a given content to a file in file_path

Parameters:

* file_path (string) - Path of the file to be read
* content (string) - Content to be written to the file

Returns:

* true if succeeded to write or false if not

Example:

```lua
local ok = fs.write_file(dir_name .. '/test2.json', "test content")
env.print(ok)
```
