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
