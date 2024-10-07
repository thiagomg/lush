## files module

`files.zip(zip_name, ...files)`

Adds a list of files to a compressed zip archive

Parameters:

* zip_name - The name of the resulting ZIP file.
* files (Variadic) - A variadic list of files to include in the ZIP archive.

Result:

* Indicates success or an error if the zipping fails.

Example:

```lua
files.zip("archive.zip", "file1.txt", "file2.txt")
```

---

`files.unzip(zip_name, output_dir)`

Decompresses a ZIP archive into the specified output directory.

Parameters:

* zip_name: The name of the ZIP file to extract.
* output_dir: An optional path to the directory where the contents will be extracted.

Result:

* Indicates success or an error if the extraction fails.

Example:

```lua
fs.unzip("archive.zip", "output_directory")
```
