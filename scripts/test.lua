env.print('pwd: ' .. tostring(env.pwd()))

env.invalid_f('1')

fs.mkdir('/tmp/lush-ut')

env.print('Zipping files')
files.zip("/tmp/lush-ut/new_post.zip", "src")

env.print('pushd to /tmp/lush-ut')
env.pushd('/tmp/lush-ut')
env.print('Unzipping files')
files.unzip("new_post.zip")
env.print('popd from /tmp/lush-ut')
env.popd()

env.print('Removing /tmp/lush-ut')
fs.rmdir('/tmp/lush-ut', { recursive = true })

env.print('Entering /tmp')
env.cd('/tmp')
env.print('Listing files')
local files = fs.ls() -- '*.md'
for i = 1, #files do
    env.print("- ", files[i])
end

env.set('NAME', 'Thiago')
env.print('ENV: ' .. env.get('NAME'))
env.del('NAME')
env.print('ENV: ' .. tostring(env.get('NAME')))

env.print('os name: ' .. os.name())
