use crate::{SeaSerpentError, database::Database};
use std::rc::Rc;
use mlua::Lua;

pub fn run_command(database: Database, command: &str, _args: &[String]) -> Result<(), SeaSerpentError> {
    let database = Rc::new(database);
    let script = database.get_command(command)?;
    // TODO Remove unwrap
    let lua = create_lua_instance().unwrap();
    lua.load(&script).eval::<()>();
    Ok(())
}

fn create_lua_instance() -> Result<Lua, mlua::Error> {
    let lua = Lua::new();
    Ok(lua)
}
