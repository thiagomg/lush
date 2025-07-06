function showme(x)
    return 'from lua: ' .. x
end

function in_brackets(x)
    return '[' .. x .. ']'
end

function only_errors(x)
    if string.find(x, 'error') then
        return x
    end
end

$> tail /tmp/my-file.log | `in_brackets` | grep "error"
-- os.pipe_exec(
--     {"tail", "/tmp/my-file.log"},
--     {in_brackets},
--     {"grep", "error"}
-- )

local x = $(cat /tmp/my-file.log | `in_brackets` | grep "error")
-- x = os.pipeline(
--     {"cat", "/tmp/my-file.log"},
--     {in_brackets},
--     {only_errors}
-- )
print("content:" .. x)

local lush_files = $(find . -name "*.lush")
print("lush files: " .. lush_files)
