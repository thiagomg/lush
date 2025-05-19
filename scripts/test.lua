print('name: ' .. os.name())

env.print('pwd: ' .. tostring(env.pwd()))

fs.mkdir('/tmp/lush-ut')
env.print('Zipping files')
files.zip("/tmp/lush-ut/new_post0.zip", "src", "Cargo.toml")

files.compress("/tmp/lush-ut/new_post.zip", "src", "Cargo.toml")
files.compress("/tmp/lush-ut/new_post.tar.zst", "src", "Cargo.toml")

function unc_files(comp_file_name, where, use_zip)
    fs.mkdir(where)
    env.print('pushd to ' .. where)
    env.pushd(where)
    env.print('Unzipping files')
    if(use_zip) then
        files.unzip(comp_file_name)
    else
        files.decompress(comp_file_name)
    end
    env.print('popd from ' .. where)
    env.popd()
end

unc_files("../new_post0.zip", '/tmp/lush-ut/zip0_unc', true)
unc_files("../new_post.zip", '/tmp/lush-ut/zip_unc', false)
unc_files("../new_post.tar.zst", '/tmp/lush-ut/tar_zst_unc', false)

env.print('Deleting all the dirs')
fs.rmdir('/tmp/lush-ut/zip0_unc', { recursive = true })
fs.rmdir('/tmp/lush-ut/zip_unc', { recursive = true })
fs.rmdir('/tmp/lush-ut/tar_zst_unc', { recursive = true })

fs.rmdir('/tmp/lush-ut', { recursive = true })

env.print('Entering /tmp')
env.cd('/tmp')
env.print('Listing files')
local files = fs.ls() -- '*.md'
for _, file in ipairs(files) do
    env.print("- ", file)
end

env.set('NAME', 'Thiago')
env.print('ENV: ' .. env.get('NAME'))
env.del('NAME')
env.print('ENV: ' .. tostring(env.get('NAME')))

env.print('os name: ' .. os.name())
