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

`files.compress(target_file_name, ...files)`

Adds a list of files to a compressed compressed archive
Supported extensions are .zip and .tar.zst

Parameters:

* target_file_name - The name of the resulting compressed file.
* files (Variadic) - A variadic list of files to include in the compressed archive.

Result:

* Indicates success or an error if the compression fails.

Example:

```lua
files.compress("archive.zip", "file1.txt", "file2.txt", "dir3")
files.compress("archive.tar.zst", "file1.txt", "file2.txt", "dir3")
```

---

`files.decompress(source_file_name, output_dir)`

Decompresses a compressed archive into the specified output directory.

Parameters:

* source_file_name: The name of the compressed file to extract.
* output_dir: An optional path to the directory where the contents will be extracted.

Result:

* Indicates success or an error if the extraction fails.

Example:

```lua
fs.decompress("archive.zip", "output_directory")
fs.decompress("archive.tar.zst", "output_directory")
```
