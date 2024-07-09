#[cfg(test)]
pub const DATA: &str = r##"
function new_thing()
    local thing = {}
    setmetatable(thing, {
        __close = function()
            print("thing closed")
        end
    })
    return thing
end

do
    local x <close> = new_thing()
    print("using thing")
end

local target_dir = '/tmp/lush-1'
fs.mkdir(target_dir)
print('pwd: ' .. tostring(env.pwd()))
files.zip("/tmp/lush-1/new_post.zip", "src")

env.pushd(target_dir)
local files = fs.ls()
for i = 1, #files do
    print(files[i])
end
env.popd()
fs.rmdir(target_dir, { recursive = true })

env.set('NAME', 'Thiago')
print('ENV: ' .. env.get('NAME'))
env.del('NAME')
print('ENV: ' .. tostring(env.get('NAME')))

print('os name: ' .. os.name())
    "##;