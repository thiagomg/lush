use std::{io, thread};
use std::io::{BufRead, BufReader, Write};
use mlua::{Function, Lua, Result, Table, Value};
use mlua::Error::SyntaxError;

use std::process::{Child, Command, Stdio};
use os_pipe::{pipe, PipeReader};

struct SingleCommand {
    func_name: String,
    args: Vec<String>,
}

struct LuaFunction {
    function: Function,
}

unsafe impl Send for LuaFunction {}

enum PipelineStep {
    External(SingleCommand),
    RustFn(fn(Box<dyn BufRead + Send>, Box<dyn Write + Send>) -> io::Result<()>),
    LuaFn(LuaFunction),
}

pub fn process_value(lua: &Lua, value: Value) -> Result<()> {
    match value {
        Value::String(s) => {
            println!("String: {}", s.to_str()?);
        }
        Value::Integer(i) => {
            println!("Integer: {}", i);
        }
        Value::Number(n) => {
            println!("Number: {}", n);
        }
        Value::Function(f) => {
            let result: Value = f.call(())?;
            println!("Function result: {:?}", result);
            process_value(lua, result)?; // recursively handle result
        }
        Value::Table(t) => {
            println!("Table:");
            run_exec(lua, t)?; // recursively process the table
        }
        _ => {
            println!("Unhandled type: {:?}", value);
        }
    }
    Ok(())
}

fn pipe_lua<R: BufRead, W: Write>(mut reader: R, mut writer: W, lua_func: &LuaFunction) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;

        //let res = lua_func.function.call::<String>(line.as_str()).unwrap(); // TODO: Unwrap

        match lua_func.function.call::<String>(line.as_str()) {
            Ok(res) => {
                writeln!(writer, "{}", res)?;
            }
            Err(e) => {
                writeln!(writer, "{}", e)?;
            }
        }
    }

    Ok(())
}

fn process_command(lua: &Lua, t: Table) -> Result<PipelineStep> {
    let mut vals = t.sequence_values::<Value>();
    let function = vals.next().unwrap()?;
    if !function.is_string() && !function.is_function() {
        return Err(SyntaxError {
            message: format!("The first argument has to be a String or a function. Received invalid {:?}", function),
            incomplete_input: false,
        });
    }
    let fname = function.to_string()?;

    let mut args: Vec<String> = vec![];
    for pair in vals {
        let val = match pair? {
            Value::Integer(n) => format!("{}", n),
            Value::Number(n) => format!("{}", n),
            Value::String(s) => format!("{}", s.to_str()?),
            _ => {
                return Err(SyntaxError {
                    message: format!("The arguments following the command have to be a string, integer or real number. Received invalid {:?}", function),
                    incomplete_input: false,
                });
            }
        };
        args.push(val);
    }

    if function.is_string() {
        Ok(PipelineStep::External(SingleCommand {
            func_name: fname,
            args,
        }))
    } else if function.is_function() {
        // let lua_func: Function = function.as_function().unwrap().clone();
        // Ok(PipelineStep::LuaFn(LuaFunction{
        //     function: lua_func,
        // }))
        return Err(SyntaxError {
            message: "Lua Functions are not supported at the moment".to_string(),
            incomplete_input: false,
        });
    } else {
        return Err(SyntaxError {
            message: format!("Function name cannot be {:?}", function),
            incomplete_input: false,
        });
    }
}

pub fn run_exec(lua: &Lua, table: Table) -> Result<()> {
    let mut cmds: Vec<PipelineStep> = vec![];
    for pair in table.sequence_values::<Value>() {
        let value = pair?;
        match value {
            Value::Table(t) => {
                let cmd = process_command(lua, t)?;
                cmds.push(cmd);
            }
            _ => {
                println!("Each argument must be a table: {:?}", value);
                return Err(SyntaxError {
                    message: "Each argument of the exec function must be a table".to_string(),
                    incomplete_input: false,
                });
            }
        }
    }

    for cmd in cmds.iter() {
        match cmd {
            PipelineStep::External(cmd) => {
                print!("-> executing external command {}(", cmd.func_name);
                for arg in cmd.args.iter() {
                    print!(" {}", arg);
                }
                println!(" )");
            }
            PipelineStep::RustFn(_) => {
                println!("-> executing rust function");
            }
            PipelineStep::LuaFn(_) => {
                println!("-> executing lua function");
            }
        }

    }

    run_piped(cmds)?;

    Ok(())
}

fn run_piped(steps: Vec<PipelineStep>) -> io::Result<()> {
    let mut children: Vec<Child> = vec![];
    let mut threads = vec![];

    // First input is stdin
    let mut input: Option<PipeReader> = None;

    for (_i, step) in steps.into_iter().enumerate() {
        // Create a pipe for the current step’s output
        let (reader, writer) = pipe()?;

        match step {
            PipelineStep::External(single_cmd) => {
                let cmd = single_cmd.func_name;
                let args = single_cmd.args;

                let mut command = Command::new(cmd);
                command.args(args);

                // Use previous output as stdin
                if let Some(prev_read) = input.take() {
                    command.stdin(Stdio::from(prev_read));
                }

                command.stdout(Stdio::from(writer));
                let child = command.spawn()?;
                children.push(child);
            }
            PipelineStep::RustFn(func) => {
                let reader: Box<dyn BufRead + Send> = match input {
                    Some(prev_reader) => Box::new(BufReader::new(prev_reader)),
                    None => Box::new(BufReader::new(std::io::stdin())),
                };

                threads.push(thread::spawn(move || {
                    let reader = Box::new(BufReader::new(reader)) as Box<dyn BufRead + Send>;
                    let writer = Box::new(writer) as Box<dyn Write + Send>;
                    func(reader, writer).expect("Rust function failed");
                }));
            }
            PipelineStep::LuaFn(lua_func) => {
                let reader: Box<dyn BufRead + Send> = match input {
                    Some(prev_reader) => Box::new(BufReader::new(prev_reader)),
                    None => Box::new(BufReader::new(std::io::stdin())),
                };

                threads.push(thread::spawn(move || {
                    let reader = Box::new(BufReader::new(reader)) as Box<dyn BufRead + Send>;
                    let writer = Box::new(writer) as Box<dyn Write + Send>;
                    pipe_lua(reader, writer, &lua_func).expect("Rust function failed");
                }));
            }
        }

        // Output of this step becomes input of next step
        input = Some(reader);
    }

    // If the last step wrote to a pipe, forward it to stdout
    if let Some(mut final_output) = input {
        io::copy(&mut final_output, &mut io::stdout())?;
    }

    // Wait for all threads and processes
    for thread in threads {
        thread.join().unwrap();
    }

    for mut child in children {
        child.wait()?;
    }

    Ok(())
}
