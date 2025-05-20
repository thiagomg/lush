local mod = require('mod')
local mod1 = require('mod1')

print('cmd line args:')
for i = 0, #arg do
    local cur = arg[i]
    local n = tonumber(cur)
    if n ~= nil then
        print("arg[" .. i .. "] = " .. tostring(n) .. ' + 1 = ' .. tostring(mod.plus_one(n))
            .. " | " .. tostring(n) .. ' + 10 = ' .. tostring(mod1.plus_ten(n)))
    else
        print("arg[" .. i .. "] = " .. tostring(arg[i]))
    end
end
