
// Use wit_bindgen to generate the bindings from the component model to Rust.
// For more information see: https://github.com/bytecodealliance/wit-bindgen/
wit_bindgen::generate!({
    path: ".",
    world: "kv",
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn replace_value(f: &Connection,key:_rt::String,value:_rt::String,) -> Option<_rt::String> {
        let f = Connection::new();
        let kv = f.get(&key);
        f.set(&key, &value);
        kv
    }
    
    fn consume(f:Connection,) -> () {
        println!("consume");
    }
}
