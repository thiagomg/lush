# LuSH

LuSH is a lua for shell scripts - embedded in a single binary, easy to share, to run everywhere

## The problem

I love Bash. I have tons of bash scripts, they're run only on my machine. If I copy to other hosts, they never work properly because either the bash version is older or some of the shell utils are not available or are not the same version, etc. Surely there are many methods to solve this, but the scripts complexity just increase. In practice, they never work everywhere.

I want to __simplify__ my life, not the other way around.

## The solution

My own solution for that is to be able to deploy, just by copying, all I need to run my scripts. My first idea was to build a portable bash version, but I still need to have many other utils, such as awk, cut, wc, etc. So, why not using a more powerful language? For that, one widely used, easy and that just works is [LUA](http://www.lua.org/). I love its simplicity and powerful tables.

Initially, where I am, this will be a portable LUA interpreter (without any dependencies, all static) and many lua scripts with helpers to make it easy for the scripts to call local binaries.

## Why nots

Why not Python?
> Because I really don't like whitespace as block delimiter

Why not Ruby?
> Because the runtime is big and not very portable.

Why not (whatever other language)?
> Because I had to choose one. I got a super popular language as I don't want to be the eternal father of the project. I want something to solve my issues and not creating more. LUA has tons of scripts to do pretty much anything and most of them I just need to copy a lua file to a directory. Isn't it great?

## Installing

Run the script

``` sh
./build.sh
```

If it succeeds, you'll have a bin directory with 2 binaries: lua and luac.

You can delete the directory 3rd now with the command:

``` sh
rm -r ./3rd
```
## Roadmap (so far)

- Adding tests in the code (if it makes sense)
- Adding a shell lib to run shell commands
- Create a lush binary, that won't be a bash script searching for bin_dir/lib automatically
