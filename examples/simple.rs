use host::{add_to_linker, Host};
use wasmtime::component::{ bindgen, Component, ComponentType, Linker, Resource, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::{add_to_linker_sync, IoView, WasiCtx, WasiCtxBuilder, WasiView};


struct HostComponent;
impl Host  for  HostComponent{
    #[doc = " Example function that does a simple a × b operation"]
    fn multiply(&mut self,a:f32,b:f32,) -> f32 {
        println!("we get a*b");
        a*b
    }
}

pub struct ComponentRunStates {
    // These two are required basically as a standard way to enable the impl of IoView and
    // WasiView.
    // impl of WasiView is required by [`wasmtime_wasi::p2::add_to_linker_sync`]
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    pub host_component : HostComponent,
    // You can add other custom host states if needed
}

impl IoView for ComponentRunStates {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}
impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}
bindgen!("simple" in "wasm_simple/simple.wit");

fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker: Linker<ComponentRunStates> = Linker::new(&engine);
    add_to_linker_sync(&mut linker)?;
    add_to_linker(&mut linker, |state| &mut state.host_component)?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let mut state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
        host_component: HostComponent,
    };
    
    let mut store = Store::new(&engine, state);
    // Instantiate our component with the imports we've created, and run it.
    let component = Component::from_file(&engine, "target/wasm32-wasip2/debug/guest_simple.wasm")?;
    let convert = Simple::instantiate(&mut store, &component, &linker)?;
    let program_result = convert.call_convert_celsius_to_fahrenheit(store, 0.4);
    println!("{:?}",program_result);
    Ok(())

}
