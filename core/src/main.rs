extern crate libloading;

use libloading::{Library, Symbol};

use models::ImageReader;

pub mod models;

type ReaderProvider = unsafe fn() -> Box<dyn ImageReader>;

fn main() {
    println!("Hello, world!");

    unsafe { 
        let lib = Library::new("./plugins/libbmp_support.so")
            .expect("failed to load bmp plugin");
        let func: Symbol<ReaderProvider> = lib.get(b"init_reader").unwrap();
    
        let reader = func();
        reader.read(&[]);

        println!("done");
    }
}
