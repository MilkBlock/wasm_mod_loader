use std::collections::HashMap;

use local::kv_store::kvdb::{add_to_linker,  Host, HostConnection};
use wasmtime::component::{ bindgen, Component, ComponentType, Linker, Resource, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::{add_to_linker_sync, IoView, WasiCtx, WasiCtxBuilder, WasiView};

pub struct Conn{
    storage : HashMap<String,String>
}
impl HostConnection for ComponentCtx {
    fn new(&mut self) -> Result<Resource<Conn>, wasmtime::Error> {
        Ok(self.resource_table.push(Conn {
            storage: HashMap::new(),
        })?)
    }

    fn get(
        &mut self,
        resource: Resource<Conn>,
        key: String,
    ) -> Result<Option<String>, wasmtime::Error> {
        let connection = self.resource_table.get(&resource)?;
        Ok(connection.storage.get(&key).map(String::clone))
    }

    fn set(&mut self, resource: Resource<Conn>, key: String, value: String) -> Result<()> {
        let connection = self.resource_table.get_mut(&resource)?;
        connection.storage.insert(key, value);
        Ok(())
    }

    fn remove(&mut self, resource: Resource<Conn>, key: String) -> Result<Option<String>> {
        let connection = self.resource_table.get_mut(&resource)?;
        Ok(connection.storage.remove(&key))
    }

    fn clear(&mut self, resource: Resource<Conn>) -> Result<(), wasmtime::Error> {
        let large_string = self.resource_table.get_mut(&resource)?;
        large_string.storage.clear();
        Ok(())
    }

    fn drop(&mut self, resource: Resource<Conn>) -> Result<()> {
        println!("{:?} dropped in wasm",resource);
        let _ = self.resource_table.delete(resource)?;
        Ok(())
    }
}

pub struct ComponentCtx {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}
impl Host for ComponentCtx{ }

impl IoView for ComponentCtx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}
impl WasiView for ComponentCtx {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}
bindgen!({
    world:"kv" ,
    path:"wasm_resource/resource.wit",
    with: {
        "local:kv-store/kvdb/connection" : Conn
    },
    trappable_imports: true,
});


fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker: Linker<ComponentCtx> = Linker::new(&engine);
    add_to_linker_sync(&mut linker)?;
    add_to_linker(&mut linker, |state| state)?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let mut state = ComponentCtx {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);
    // Instantiate our component with the imports we've created, and run it.
    let component = Component::from_file(&engine, "target/wasm32-wasip2/debug/guest_resource.wasm")?;
    let kv = Kv::instantiate(&mut store, &component, &linker)?;
    let con = store.data_mut().new().unwrap();
    let program_result = kv.call_replace_value(&mut store, con.borrowed(),"a", "b");
    let program_result = kv.call_replace_value(&mut store, con.borrowed(),"a", "c");
    kv.call_consume(&mut store, con)?;
    // println!("{:?}",program_result);
    println!("{:?}",program_result);
    Ok(())

}

trait ResourceExt<T> {
    fn borrowed(&self) -> Resource<T>;
}

impl<T: 'static> ResourceExt<T> for Resource<T> {
    fn borrowed(&self) -> Resource<T> {
        Resource::new_borrow(self.rep())
    }
}

