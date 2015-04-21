// Lifted from https://github.com/rust-lang/cargo/blob/master/src/cargo/core/shell.rs
// under MIT license https://github.com/rust-lang/cargo/blob/master/LICENSE-MIT
//
// Modified by: Ignacio Corderi

#![crate_name = "shell"]

extern crate term;
extern crate libc;

use std::fmt;
use std::fmt::Display;
use std::io::prelude::*;
use std::io;
use std::error::Error;

use term::Attr;
use term::color::{Color, BLACK, RED, GREEN, YELLOW, BRIGHT_YELLOW, BRIGHT_RED};
use term::{Terminal, TerminfoTerminal, color};

use self::AdequateTerminal::{NoColor, Colored};

static GREY: u16 = 8;

#[derive(Clone, Copy)]
pub struct ShellConfig {
    pub color: bool,
    pub verbose: bool,
    pub tty: bool
}

enum AdequateTerminal {
    NoColor(Box<Write + Send>),
    Colored(Box<Terminal<UghWhyIsThisNecessary> + Send>)
}

pub struct Shell {
    terminal: AdequateTerminal,
    config: ShellConfig,
}

pub struct MultiShell {
    out: Shell,
    err: Shell,
    verbose: bool
}

struct UghWhyIsThisNecessary {
    inner: Box<Write + Send>,
}

#[cfg(unix)]
fn isatty(fd: libc::c_int) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}
#[cfg(windows)]
fn isatty(fd: libc::c_int) -> bool {
    extern crate kernel32;
    extern crate winapi;
    unsafe {
        let handle = kernel32::GetStdHandle(
            if fd == libc::STDOUT_FILENO {
                winapi::winbase::STD_OUTPUT_HANDLE
            } else {
                winapi::winbase::STD_ERROR_HANDLE
            });
        let mut out = 0;
        kernel32::GetConsoleMode(handle, &mut out) != 0
    }
}

impl MultiShell {

    pub fn new_stdio(verbose: bool) -> MultiShell {
        let tty = isatty(libc::STDERR_FILENO);
        let stderr = Box::new(io::stderr()) as Box<Write + Send>;

        let config = ShellConfig { color: true, verbose: verbose, tty: tty };
        let err = Shell::create(stderr, config);

        let tty = isatty(libc::STDOUT_FILENO);
        let stdout = Box::new(io::stdout()) as Box<Write + Send>;

        let config = ShellConfig { color: true, verbose: verbose, tty: tty };
        let out = Shell::create(stdout, config);

        MultiShell::new(out, err, verbose)
    }

    pub fn new(out: Shell, err: Shell, verbose: bool) -> MultiShell {
        MultiShell { out: out, err: err, verbose: verbose }
    }

    pub fn out(&mut self) -> &mut Shell {
        &mut self.out
    }

    pub fn err(&mut self) -> &mut Shell {
        &mut self.err
    }

    pub fn say<T: ToString>(&mut self, message: T, color: Color) -> io::Result<()> {
        self.out().say(message, color)
    }

    pub fn status<T, U>(&mut self, status: T, message: U) -> io::Result<()>
        where T: fmt::Display, U: fmt::Display
    {
        self.out().say_status(status, message, GREEN)
    }

    pub fn verbose<F>(&mut self, mut callback: F) -> io::Result<()>
        where F: FnMut(&mut MultiShell) -> io::Result<()>
    {
        if self.verbose { return callback(self) }
        Ok(())
    }

    pub fn concise<F>(&mut self, mut callback: F) -> io::Result<()>
        where F: FnMut(&mut MultiShell) -> io::Result<()>
    {
        if !self.verbose { return callback(self) }
        Ok(())
    }

    pub fn error<T: ToString>(&mut self, message: T) -> io::Result<()> {
        self.err().say(message, RED)
    }

    pub fn warn<T: ToString>(&mut self, message: T) -> io::Result<()> {
        self.err().say(message, YELLOW)
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn get_verbose(&self) -> bool {
        self.verbose
    }

    pub fn tag<T: Display, U: Display>(&mut self, tag: T, message: U) -> io::Result<()>{
        self.out().say_status(tag, message, BLACK)
    }

    pub fn header<T: Display>(&mut self, message: T) -> io::Result<()> {
        self.out().say_attr(message, BLACK, Attr::Underline(true), true)
    }

    pub fn comment<T: Display>(&mut self, message: T) -> io::Result<()> {
        self.out().say_attr(message, GREY, Attr::Dim, true)
    }

    pub fn tag_color<T: Display, U: Display>(&mut self, tag: T, message: U, color: Color) -> io::Result<()>{
        self.out().say_status(tag, message, color)
    }

    pub fn error_full(&mut self, e: &Error, mut show_cause: bool) -> io::Result<()>{
        try!(self.err().say_write(      "error: ", BRIGHT_RED));
        try!(self.err().say_attr(format!("{}", e.description()), BLACK, Attr::Bold, true));

        let mut e = e;
        while show_cause {
            if e.cause().is_some() {
                e = e.cause().unwrap();
                try!(self.err().say_write(      "caused by: ", BRIGHT_YELLOW));
                try!(self.err().say(format!("{}", e.description()), BLACK));
            } else { show_cause = false; }
        }

        Ok(())
    }
}

impl Shell {
    pub fn create(out: Box<Write + Send>, config: ShellConfig) -> Shell {
        let out = UghWhyIsThisNecessary { inner: out };
        if config.tty && config.color {
            let term = TerminfoTerminal::new(out);
            term.map(|t| Shell {
                terminal: Colored(Box::new(t)),
                config: config
            }).unwrap_or_else(|| {
                Shell { terminal: NoColor(Box::new(io::stderr())), config: config }
            })
        } else {
            Shell { terminal: NoColor(out.inner), config: config }
        }
    }

    pub fn verbose<F>(&mut self, mut callback: F) -> io::Result<()>
        where F: FnMut(&mut Shell) -> io::Result<()>
    {
        if self.config.verbose { return callback(self) }
        Ok(())
    }

    pub fn concise<F>(&mut self, mut callback: F) -> io::Result<()>
        where F: FnMut(&mut Shell) -> io::Result<()>
    {
        if !self.config.verbose { return callback(self) }
        Ok(())
    }

    pub fn say_write<T: Display>(&mut self, message: T, color: Color) -> io::Result<()> {
        try!(self.reset());
        if color != BLACK { try!(self.fg(color)); }
        try!(write!(self, "{}", message));
        try!(self.reset());
        try!(self.flush());
        Ok(())
    }

    pub fn say_attr<T: Display>(&mut self, message: T, color: Color, attr: Attr, new_line: bool) -> io::Result<()> {
        try!(self.reset());
        try!(self.attr(attr));
        if color != BLACK { try!(self.fg(color)); }
        if new_line {
            try!(write!(self, "{}\n", message));
        } else {
            try!(write!(self, "{}", message));
        }
        try!(self.reset());
        try!(self.flush());
        Ok(())
    }

    pub fn say<T: ToString>(&mut self, message: T, color: Color) -> io::Result<()> {
        try!(self.reset());
        if color != BLACK { try!(self.fg(color)); }
        try!(write!(self, "{}\n", message.to_string()));
        try!(self.reset());
        try!(self.flush());
        Ok(())
    }

    pub fn say_status<T, U>(&mut self, status: T, message: U, color: Color)
                            -> io::Result<()>
        where T: fmt::Display, U: fmt::Display
    {
        try!(self.reset());
        if color != BLACK { try!(self.fg(color)); }
        if self.supports_attr(Attr::Bold) { try!(self.attr(Attr::Bold)); }
        try!(write!(self, "{:>12}", status.to_string()));
        try!(self.reset());
        try!(write!(self, " {}\n", message));
        try!(self.flush());
        Ok(())
    }

    fn fg(&mut self, color: color::Color) -> io::Result<bool> {
        match self.terminal {
            Colored(ref mut c) => c.fg(color),
            NoColor(_) => Ok(false)
        }
    }

    fn attr(&mut self, attr: Attr) -> io::Result<bool> {
        match self.terminal {
            Colored(ref mut c) => c.attr(attr),
            NoColor(_) => Ok(false)
        }
    }

    fn supports_attr(&self, attr: Attr) -> bool {
        match self.terminal {
            Colored(ref c) => c.supports_attr(attr),
            NoColor(_) => false
        }
    }

    fn reset(&mut self) -> io::Result<()> {
        match self.terminal {
            Colored(ref mut c) => c.reset().map(|_| ()),
            NoColor(_) => Ok(())
        }
    }
}

impl Write for Shell {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.terminal {
            Colored(ref mut c) => c.write(buf),
            NoColor(ref mut n) => n.write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.terminal {
            Colored(ref mut c) => c.flush(),
            NoColor(ref mut n) => n.flush()
        }
    }
}

impl Write for UghWhyIsThisNecessary {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.inner.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod test {

    use MultiShell;

    #[test]
    fn create_multishell() {
       MultiShell::new_stdio(false);
    }

    #[test]
    fn create_multishell_verbose() {
       MultiShell::new_stdio(true);
    }

}
