## net module

---

`net.wget(url, out_filename)`

Downloads a file and optionally renames to out_filename if provided

Returns:

* Name of the downloaded file

Example:

```lua
local filename1 = net.wget("https://my-server/file.txt")
local filename2 = net.wget("https://my-server/file.txt", 'another.txt')
```

---
