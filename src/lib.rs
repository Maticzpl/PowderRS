extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use std::string::{String};
use std::any::{Any, TypeId};
use std::{env, fs};

//ik this is garbage, just learning macros :P
#[proc_macro]
pub fn get_part_types_in_dir(stream: TokenStream) -> TokenStream {
    let dir = match stream.into_iter().next().unwrap() {
        TokenTree::Literal(val) => {
            Some(val.to_string().replace("\"",""))
        },
        _ => {None}
    };

    let path = format!("{}/{}", env::current_dir().unwrap().to_str().unwrap(), dir.unwrap());
    let files: Vec<_> = fs::read_dir(path.clone())
        .expect(format!("Can't open dir {}", path).as_str())
        .map(|file| file.unwrap().path())
        .collect();
    let mut imports: String = String::new();
    let mut strc: String = format!("pub const PT_TYPES : [PartType; {}] = [", files.len() - 1); //to lazy to do proper counting
    for path in files {
        if path.is_file() && path.file_name().unwrap() != "mod.rs" {
            let type_ident = path.file_name()
                .unwrap().to_str().unwrap()
                .split(".").next().unwrap();
            imports += format!("pub mod {};\npub use {}::PT_{};", type_ident, type_ident, type_ident.to_uppercase()).as_str();
            strc += format!("PT_{},\n", type_ident.to_uppercase()).as_str();
        }
    }
    strc.remove(strc.len()-1);
    strc += "];";
    return format!("{}\n{}",imports,strc).parse().unwrap();
}