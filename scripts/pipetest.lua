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

exec({
    {"cat", "~/my-file.log"},
    {in_brackets},
    -- {"grep", "error"},
    {only_errors},
    {showme},
})
