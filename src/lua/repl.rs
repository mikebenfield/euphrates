// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;
use std::sync::mpsc;

use serde::de::DeserializeOwned;
use rlua;

use lua::serde as lua_serde;

pub fn repl<T>(sender: mpsc::Sender<T>, initial_code: &str)
where
    T: 'static + Send + DeserializeOwned
{
    use std::io::Write;

    let stdin = std::io::stdin();
    let lua = rlua::Lua::new();
    let f = lua.create_function(
        move |_, command: lua_serde::FromLuaN<T>| {
            sender.send(command.0).unwrap();
            Ok(())
        }
    );
    lua.globals().raw_set("command", f).unwrap();

    let () = lua.eval(initial_code, Some("Lua REPL Initial code")).unwrap();

    loop {
        let mut input_string = "".to_string();
        std::thread::sleep(std::time::Duration::from_millis(50));
        print!("> ");
        std::io::stdout().flush().unwrap();
        stdin.read_line(&mut input_string).unwrap();
        match lua.eval::<rlua::Value>(&input_string, None) {
            Ok(v) => {
                println!("{:?}", v);
            },
            Err(e) => println!("{:?}", std::error::Error::description(&e)),
        }
    }
}
