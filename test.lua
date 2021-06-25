local cmdline = require("cmdline")
local inspect = require("inspect")

-- Testing - for now only dumping to console
-- ./lush test.lua 1a --fk=1 -sk=a tk=3 2a --name me
local options = cmdline:parse(arg)
print(inspect(options))
print('----------------------------------------')
