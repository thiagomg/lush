use std::{io, thread};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use mlua::{Function, Lua, Result, Table, Value};
use mlua::Error::SyntaxError;

use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use os_pipe::{pipe, PipeReader};

struct SingleCommand {
    func_name: String,
    args: Vec<String>,
}

type LuaFuncId = i32;
enum LuaPipeAction<T> {
    Execute(T),
    Finished(LuaFuncId),
}

struct FuncWithArg {
    uuid: LuaFuncId,
    arg: String,
}

enum PipelineStep {
    External(SingleCommand),
    // RustFn(fn(Box<dyn BufRead + Send>, Box<dyn Write + Send>) -> io::Result<()>),
    LuaFn(mlua::Function),
}

fn pipe_lua<R: BufRead, W: Write>(reader: R, mut writer: W, func_uuid: LuaFuncId, tx_to_lua: Sender<LuaPipeAction<FuncWithArg>>, rx_from_lua: Arc<Mutex<Receiver<Option<String>>>>) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;

        tx_to_lua.send(LuaPipeAction::Execute(FuncWithArg { uuid: func_uuid, arg: line })).unwrap();
        let line_from_lua = rx_from_lua.lock().unwrap().recv().unwrap();
        if let Some(line_from_lua) = line_from_lua {
            writeln!(writer, "{}", line_from_lua)?;
            writer.flush()?;
        }
    }

    tx_to_lua.send(LuaPipeAction::Finished(func_uuid)).unwrap();

    Ok(())
}

fn process_command(t: Table) -> Result<PipelineStep> {
    let mut vals = t.sequence_values::<Value>();
    let next_val = vals.next();
    if next_val.is_none() {
        return Err(SyntaxError {
            message: "The argument passed to exec has to be a String or a function. Received nil".to_string(),
            incomplete_input: false,
        });
    }
    let function = next_val.unwrap()?;
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
        let lua_func: Function = function.as_function().unwrap().clone();
        Ok(PipelineStep::LuaFn(lua_func))
    } else {
        return Err(SyntaxError {
            message: format!("Function name cannot be {:?}", function),
            incomplete_input: false,
        });
    }
}

pub fn run_exec(_lua: &Lua, table: Table) -> Result<()> {
    let mut cmds: Vec<PipelineStep> = vec![];
    for pair in table.sequence_values::<Value>() {
        let value = pair?;
        match value {
            Value::Table(t) => {
                let cmd = process_command(t)?;
                cmds.push(cmd);
            }
            _ => {
                eprintln!("Each argument must be a table: {:?}", value);
                return Err(SyntaxError {
                    message: "Each argument of the exec function must be a table".to_string(),
                    incomplete_input: false,
                });
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

    let (tx_to_lua, rx_to_lua) = mpsc::channel::<LuaPipeAction<FuncWithArg>>();
    let lua_result_senders = Arc::new(Mutex::new(HashMap::<LuaFuncId, Sender<Option<String>>>::new()));

    let mut func_map: HashMap<LuaFuncId, mlua::Function> = HashMap::new();
    let mut func_num = 0;

    for step in steps.into_iter() {
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
            /*
            // PipelineStep::RustFn(func) => {
            //     let reader: Box<dyn BufRead + Send> = match input {
            //         Some(prev_reader) => Box::new(BufReader::new(prev_reader)),
            //         None => Box::new(BufReader::new(std::io::stdin())),
            //     };
            //
            //     threads.push(thread::spawn(move || {
            //         let reader = Box::new(BufReader::new(reader)) as Box<dyn BufRead + Send>;
            //         let writer = Box::new(writer) as Box<dyn Write + Send>;
            //         func(reader, writer).expect("Rust function failed");
            //     }));
            // }
            */
            PipelineStep::LuaFn(lua_func) => {
                let reader: Box<dyn Read + Send> = match input {
                    Some(prev_reader) => Box::new(prev_reader),
                    None => Box::new(std::io::stdin()),
                };

                func_num += 1;
                func_map.insert(func_num, lua_func);

                let tx_to_lua = tx_to_lua.clone();

                let (tx_from_lua, rx_from_lua) = mpsc::channel::<Option<String>>();
                lua_result_senders.lock().unwrap().insert(func_num, tx_from_lua);

                let rx_from_lua = Arc::new(Mutex::new(rx_from_lua));

                threads.push(thread::spawn(move || {
                    let reader = Box::new(BufReader::new(reader)) as Box<dyn BufRead + Send>;
                    let writer = Box::new(writer) as Box<dyn Write + Send>;
                    pipe_lua(reader, writer, func_num, tx_to_lua, rx_from_lua).expect("Rust function failed");
                }));
            }
        }

        // Output of this step becomes input of next step
        input = Some(reader);
    }

    while !func_map.is_empty() {
        let func_with_arg = rx_to_lua.recv().unwrap();

        match func_with_arg {
            LuaPipeAction::Execute(func_with_arg) => {
                let arg = func_with_arg.arg;
                let lua_func = func_map.get(&func_with_arg.uuid).unwrap();
                match lua_func.call::<String>(arg.as_str()) {
                    Ok(res) => {
                        let sender_map = lua_result_senders.lock().unwrap();
                        if let Some(tx) = sender_map.get(&func_with_arg.uuid) {
                            tx.send(Some(res)).unwrap();
                        }
                    }
                    Err(_e) => {
                        let sender_map = lua_result_senders.lock().unwrap();
                        if let Some(tx) = sender_map.get(&func_with_arg.uuid) {
                            tx.send(None).unwrap();
                        }
                    }
                }
            }
            LuaPipeAction::Finished(uuid) => {
                func_map.remove(&uuid);
                continue;
            }
        }
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
