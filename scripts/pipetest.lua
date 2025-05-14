function showme(x)
    -- print('from lua:', x)
    return 'showme res - not an error !'
end

exec({
    {"cat", "/Users/thiago/src/pipetest/my-file.log"},
    {"grep", "error"}
})


exec({ {"cat", "~/my-file.log"}, {"grep", "error"} })
