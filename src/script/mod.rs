use crate::{SeaSerpentError, database::Database};
use std::{
    cell::RefCell,
    rc::Rc,
    str::FromStr,
};
use mlua::{Lua, Error as LuaError, LuaSerdeExt};

pub fn run_command(database: Database, command: &str, _args: &[String]) -> Result<(), SeaSerpentError> {
    let database = Rc::new(RefCell::new(database));
    let script = database.borrow().get_command(command)?;
    // TODO Remove unwrap
    let lua = create_lua_instance(database.clone()).unwrap();
    lua.load(&script).eval::<()>()?;
    Ok(())
}

fn create_lua_instance(database: Rc<RefCell<Database>>) -> Result<Rc<Lua>, mlua::Error> {
    let lua = Rc::new(Lua::new());
    let lua_globals = lua.clone();
    let globals = lua_globals.globals();
    // Add tag
    let db_clone = database.clone();
    let add_tag = lua.create_function(move |_, (path, tag): (String, String)| {
        db_clone.borrow_mut()
            .add_tag(&std::path::PathBuf::from_str(&path).unwrap(), &tag)
            .or_else(|err| Err(LuaError::external(err)))?;
        Ok(())
    })?;
    globals.set("add_tag", add_tag)?;
    // Save database
    let db_clone = database.clone();
    let save = lua.create_function(move |_, ()| {
        db_clone.borrow().save()
            .or_else(|err| Err(LuaError::external(err)))?;
        Ok(())
    })?;
    globals.set("save", save)?;
    // Search
    let db_clone = database.clone();
    let search = lua.create_function(move |lua, search_query: String| {
        let search_expression = crate::search::parse(&search_query)
            .or_else(|err| Err(LuaError::external(err)))?;
        let db_borrow = db_clone.borrow();
        let results = db_borrow.search(search_expression);
        lua.to_value(&results)
    })?;
    globals.set("search", search)?;
    // File info
    let db_clone = database.clone();
    let info = lua.create_function(move |lua, path: String| {
        let db_borrow = db_clone.borrow();
        let result = db_borrow.get_file_info(&std::path::PathBuf::from_str(&path).unwrap())
            .or_else(|err| Err(LuaError::external(err)))?;
        lua.to_value(&result)
    })?;
    globals.set("info", info)?;
    Ok(lua)
}
