use std::{io, thread};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use mlua::{Function, Lua, Result, Table, Value, Variadic};
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

pub fn run_exec(lua: &Lua, value: Variadic<Value>) -> mlua::Result<()> {
    let table = to_table(lua, value)?;
    verify_not_empty(&table)?;

    let cmd_table = normalise_table(lua, table)?;
    let cmds = generate_cmds(cmd_table)?;

    let _ = run_piped(cmds, false)?;
    Ok(())
}

pub fn run_pipe(lua: &Lua, value: Variadic<Value>) -> mlua::Result<String> {
    let table = to_table(lua, value)?;
    verify_not_empty(&table)?;

    let cmd_table = normalise_table(lua, table)?;
    let cmds = generate_cmds(cmd_table)?;

    let res = run_piped(cmds, true)?;
    Ok(res)
}

fn to_table(lua: &Lua, value: Variadic<Value>) -> mlua::Result<Table> {
    let mut has_table = false;
    for v in value.as_slice() {
        if let Value::Table(_) = v {
            has_table = true;
            break;
        }
    }

    if !has_table {
        // That only a variadic of non tables, E.g. os.pipeline('echo', 'hello')
        let t = lua.create_table()?;
        let mut idx = 1;
        for v in value.as_slice() {
            t.set(idx, v.clone())?;
            idx += 1;
        }
        return Ok(t);
    }

    // Otherwise, let's iterate in all values and if none is a table, let's put in a table
    // The result will be a table of tables, being one table per command
    let root_table = lua.create_table()?;
    let mut idx = 1;
    for v in value.as_slice() {
        match v {
            Value::Nil => {
                return Err(SyntaxError {
                    message: "Nil values are not allowed in pipeline or pipe_exec".to_string(),
                    incomplete_input: false,
                });
            }
            Value::Table(t) => {
                root_table.set(idx, t.clone())?;
                idx += 1;
            }
            val => {
                let t = lua.create_table()?;
                t.set(1, val.clone())?;
                root_table.set(idx, t)?;
                idx += 1;
            }
        }
    }
    Ok(root_table)
}

fn verify_not_empty(table: &Table) -> mlua::Result<()> {
    if table.is_empty() {
        return Err(SyntaxError {
            message: "At least one command is required".to_string(),
            incomplete_input: false,
        });
    }
    Ok(())
}

fn normalise_table(lua: &Lua, table: Table) -> mlua::Result<Table> {
    let cmd_table: Table;

    let mut tab_found = false;
    for pair in table.sequence_values::<Value>() {
        let value = pair?;
        match value {
            Value::Table(_) => {
                tab_found = true;
                break;
            }
            _ => {}
        }
    }

    if !tab_found {
        // It's a single command.
        cmd_table = lua.create_table()?;
        cmd_table.set(1, table)?;
    } else {
        cmd_table = table;
    }
    Ok(cmd_table)
}

fn generate_cmds(cmd_table: Table) -> mlua::Result<Vec<PipelineStep>> {
    let mut cmds: Vec<PipelineStep> = vec![];
    for pair in cmd_table.sequence_values::<Value>() {
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
    Ok(cmds)
}

fn run_piped(steps: Vec<PipelineStep>, return_output: bool) -> io::Result<String> {
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
                } else {
                    command.stdin(Stdio::inherit());
                }

                command.stdout(Stdio::from(writer));
                command.stderr(Stdio::inherit());

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

    if !return_output {
        // If the last step wrote to a pipe, forward it to stdout with immediate flushing
        if let Some(mut final_output) = input {
            let mut buffer = [0; 1024];
            loop {
                match final_output.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        io::stdout().write_all(&buffer[..n])?;
                        io::stdout().flush()?; // Immediate flush
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        for mut child in children {
            child.wait()?;
        }

        for thread in threads {
            thread.join().unwrap();
        }

        Ok("".to_string())
    } else {
        let mut buffer = Vec::new();
        // Wait for all threads and processes
        for thread in threads {
            thread.join().unwrap();
        }

        for mut child in children {
            child.wait()?;
        }
        if let Some(mut final_output) = input {
            final_output.read_to_end(&mut buffer)?;
        }
        Ok(String::from_utf8(buffer).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{Lua, Value, Variadic};

    #[test]
    fn test_simple_echo_variadic() {
        let lua = Lua::new();

        // Test the new variadic API: run_pipe(&lua, "echo", "asd")
        let values = vec![
            Value::String(lua.create_string("echo").unwrap()),
            Value::String(lua.create_string("asd").unwrap()),
        ];
        let variadic = Variadic::from_iter(values);

        let result = run_pipe(&lua, variadic).unwrap();
        assert_eq!(result.trim(), "asd");
    }

    #[test]
    fn test_echo_with_exec_variadic() {
        let lua = Lua::new();

        // Test run_exec with variadic args
        let values = vec![
            Value::String(lua.create_string("echo").unwrap()),
            Value::String(lua.create_string("asd").unwrap()),
        ];
        let variadic = Variadic::from_iter(values);

        let result = run_exec(&lua, variadic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_echo_pipe_grep_tables() {
        let lua = Lua::new();

        // Test pipeline with table format: {{"echo", "asd\ntest\nasd again"}, {"grep", "asd"}}
        let echo_table = lua.create_table().unwrap();
        echo_table.set(1, "echo").unwrap();
        echo_table.set(2, "asd\ntest\nasd again").unwrap();

        let grep_table = lua.create_table().unwrap();
        grep_table.set(1, "grep").unwrap();
        grep_table.set(2, "asd").unwrap();

        let values = vec![
            Value::Table(echo_table),
            Value::Table(grep_table),
        ];
        let variadic = Variadic::from_iter(values);

        let result = run_pipe(&lua, variadic).unwrap();
        let lines: Vec<&str> = result.trim().split('\n').collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("asd"));
        assert!(lines[1].contains("asd"));
    }

    #[test]
    fn test_single_table_command() {
        let lua = Lua::new();

        // Test single table: {{"echo", "test"}}
        let echo_table = lua.create_table().unwrap();
        echo_table.set(1, "echo").unwrap();
        echo_table.set(2, "test").unwrap();

        let values = vec![Value::Table(echo_table)];
        let variadic = Variadic::from_iter(values);

        let result = run_pipe(&lua, variadic).unwrap();
        assert_eq!(result.trim(), "test");
    }

    #[test]
    fn test_multiple_echo_args() {
        let lua = Lua::new();

        // Test: run_pipe(&lua, "echo", "hello", "world", "!")
        let values = vec![
            Value::String(lua.create_string("echo").unwrap()),
            Value::String(lua.create_string("hello").unwrap()),
            Value::String(lua.create_string("world").unwrap()),
            Value::String(lua.create_string("!").unwrap()),
        ];
        let variadic = Variadic::from_iter(values);

        let result = run_pipe(&lua, variadic).unwrap();
        assert_eq!(result.trim(), "hello world !");
    }

    #[test]
    fn test_numeric_args() {
        let lua = Lua::new();

        // Test with numeric arguments: "echo", 123, 45.6
        let values = vec![
            Value::String(lua.create_string("echo").unwrap()),
            Value::Integer(123),
            Value::Number(45.6),
        ];
        let variadic = Variadic::from_iter(values);

        let result = run_pipe(&lua, variadic).unwrap();
        assert_eq!(result.trim(), "123 45.6");
    }

    #[test]
    fn test_error_empty_pipeline() {
        let lua = Lua::new();

        // Test empty pipeline should return error
        let values: Vec<Value> = vec![];
        let variadic = Variadic::from_iter(values);

        let result = run_pipe(&lua, variadic);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_pipeline() {
        let lua = Lua::new();

        // Test complex pipeline: echo "line1\nline2\nline3" | grep "line" | wc -l
        let echo_table = lua.create_table().unwrap();
        echo_table.set(1, "echo").unwrap();
        echo_table.set(2, "line1\nline2\nline3").unwrap();

        let grep_table = lua.create_table().unwrap();
        grep_table.set(1, "grep").unwrap();
        grep_table.set(2, "line").unwrap();

        let wc_table = lua.create_table().unwrap();
        wc_table.set(1, "wc").unwrap();
        wc_table.set(2, "-l").unwrap();

        let values = vec![
            Value::Table(echo_table),
            Value::Table(grep_table),
            Value::Table(wc_table),
        ];
        let variadic = Variadic::from_iter(values);

        let result = run_pipe(&lua, variadic).unwrap();
        assert_eq!(result.trim(), "3");
    }
}
