local utils = {
  _VERSION = 'utils.lua 1.0',
  _DESCRIPTION = 'LuSH utils library',
  _LICENSE = [[
    MIT LICENSE

    Copyright (c) 2021 Thiago Massari Guedes

    Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER     DEALINGS IN THE SOFTWARE.
  ]]
}

-- Inspired in https://stackoverflow.com/questions/22831701/lua-read-beginning-of-a-string
function string.startswith(s, text)
   return string.sub(s, 1, string.len(text)) == text
end

function string.endswith(s, text)
   return string.sub(s, string.len(text)) == text
end

return utils
