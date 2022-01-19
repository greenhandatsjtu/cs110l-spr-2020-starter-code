use std::collections::HashMap;

use crate::debugger_command::DebuggerCommand;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use crate::inferior::{Inferior, Status};
use nix::sys::ptrace;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[derive(Clone)]
pub struct Breakpoint {
    addr: usize,
    pub orig_byte: u8,
}

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: HashMap<usize, Breakpoint>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        debug_data.print();

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            breakpoints: HashMap::new(),
        }
    }

    pub fn cont(&mut self) {
        if self.inferior.is_none() {
            println!("The program is not being run.");
            return;
        }
        let result = self.inferior.as_mut().unwrap().cont(&self.breakpoints);
        match result {
            Ok(status) => match status {
                Status::Exited(exit_code) => {
                    println!("Child exited (status {})", exit_code);
                    self.inferior = None;
                }
                Status::Signaled(signal) => {
                    println!("Child exited (signal {})", signal);
                    self.inferior = None;
                }
                Status::Stopped(signal, rip) => {
                    println!("Child stopped (signal {})", signal);
                    let line = match self.debug_data.get_line_from_addr(rip as usize) {
                        None => return,
                        Some(val) => val,
                    };
                    let func = match self.debug_data.get_function_from_addr(rip as usize) {
                        None => return,
                        Some(val) => val,
                    };
                    println!("Stopped at {} ({})", func, line);
                    let pid = self.inferior.as_ref().unwrap().pid();
                    let mut regs = ptrace::getregs(pid).unwrap();
                    regs.rip -= 1;
                    // if inferior is stopped at a breakpoint
                    if self.breakpoints.contains_key(&(regs.rip as usize)) {
                        // rewind the instruction pointer (subtract 1 byte)
                        if ptrace::setregs(pid, regs).is_err() {
                            println!("Unable to set rip");
                            return;
                        }

                        let bp = self.breakpoints.get(&(regs.rip as usize)).unwrap();
                        // restore instruction
                        if self
                            .inferior
                            .as_mut()
                            .unwrap()
                            .write_byte(bp.addr, bp.orig_byte)
                            .is_err()
                        {
                            println!("Unable to restore instruction at {:#x}", bp.addr);
                        }
                    }
                }
            },
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill().unwrap();
                    }
                    if let Some(inferior) =
                        Inferior::new(&self.target, &args, &mut self.breakpoints)
                    {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        self.cont();
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Continue => {
                    self.cont();
                }
                DebuggerCommand::Backtrace => {
                    if self.inferior.is_none() {
                        println!("The program is not being run.");
                        continue;
                    }
                    self.inferior
                        .as_mut()
                        .unwrap()
                        .print_backtrace(&self.debug_data)
                        .unwrap();
                }
                DebuggerCommand::Break(addr) => {
                    if !addr.starts_with('*') {
                        continue;
                    }
                    let res = parse_address(&addr[1..]);
                    match res {
                        Some(addr) => {
                            println!("Set breakpoint {} at {:#x}", self.breakpoints.len(), addr);
                            if self.inferior.is_some() {
                                let res = self.inferior.as_mut().unwrap().write_byte(addr, 0xcc);
                                match res {
                                    Ok(orig_byte) => {
                                        self.breakpoints
                                            .insert(addr, Breakpoint { addr, orig_byte });
                                    }
                                    Err(_) => {
                                        println!("Unable to set breakpoint at {:#x}", addr);
                                    }
                                }
                            } else {
                                self.breakpoints
                                    .insert(addr, Breakpoint { addr, orig_byte: 0 });
                            }
                        }
                        None => {
                            println!("Please provide a valid address!");
                        }
                    }
                }
                DebuggerCommand::Quit => {
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill().unwrap();
                    }
                    return;
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}

// parse a usize from a hexadecimal string
fn parse_address(addr: &str) -> Option<usize> {
    let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
        &addr[2..]
    } else {
        &addr
    };
    usize::from_str_radix(addr_without_0x, 16).ok()
}
