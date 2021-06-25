local utils = require("utils")

local cmdline = {
  _VERSION = 'cmdline.lua 1.0',
  _DESCRIPTION = 'LuSH command line parsing library',
  _LICENSE = [[
    MIT LICENSE

    Copyright (c) 2021 Thiago Massari Guedes

    Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER     DEALINGS IN THE SOFTWARE.
  ]]
}

--[[

Supported argument types:

Positional:
- script.lua first second

Key/Value:
- script.lua name=Thiago lastname=Guedes
- script.lua -name=Thiago -lastname=Guedes
    - script.lua -name Thiago -lastname Guedes
    - script.lua --name=Thiago --lastname=Guedes
    - script.lua --name Thiago --lastname Guedes
]]--

function is_opt(t)
    return t:startswith("-")
end

function get_key(t)
    local patt = [[-?-?(%g+)]]
    local k = string.match(t, patt)
    return k
end

function get_kv(t)
    local patt = [[-?-?(%g+)=(%g+)]]
    local k, v = string.match(t, patt)
    if k ~= nil then
        return k, v
    end

    -- maybe option with value as next token
    if is_opt(t) then
        return get_key(t), nil
    end

    return nil
end

function cmdline.parse(c, arg)
    local ret = {}

    local last_key = nil
    for i = 1, #arg do
        local cur = arg[i]
        local k, v = get_kv(cur)

        if k ~= nil and v ~= nil then
            -- First case: key, val
            ret[k] = v
        elseif k ~= nil and v == nil then
            -- Second case: key, no val (option)
            last_key = k
        elseif k == nil and last_key ~= nil then
            -- Second case, part 2: key, no val (option)
            ret[last_key] = cur
            last_key = nil
        else
            -- Thrird case: positional
            table.insert(ret, cur)
        end
    end

    return ret
end

return cmdline
