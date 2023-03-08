extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use std::string::{String};
use std::any::{Any, TypeId};
use std::{env, fs};
use std::cmp::Ordering::{Greater, Less};
use regex::Regex;

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
    let mut files: Vec<_> = fs::read_dir(path.clone())
        .expect(format!("Can't find dir {}", path).as_str())
        .map(|file| file.unwrap().path())
        .collect();

    //ngl this is really awful xdd
    files.sort_by(|a, b| {
        if !a.is_file() || a.file_name().unwrap().to_str().unwrap() == "mod.rs" {
            return Less;
        }
        if !b.is_file() || b.file_name().unwrap().to_str().unwrap() == "mod.rs" {
            return Greater;
        }

        let reg = Regex::new(r"(.*\n)*.*(const\s*PT_.*(\n.*)*id\s*:\s*)(?P<res>\d+)(,)(.*\n?)*").unwrap();

        let va : u32 = reg.replace_all(fs::read_to_string(a).unwrap().as_str(), "$res").parse().unwrap();
        let vb : u32 = reg.replace_all(fs::read_to_string(b).unwrap().as_str(), "$res").parse().unwrap();

        va.cmp(&vb)
    });

    let len = files.len() - 1; //to lazy to do proper counting

    let mut imports: String = String::new();
    let mut strc: String = format!("pub const PT_TYPES : [PartType; {}] = [", len);
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

    return format!("{}\n{}", imports, strc).parse().unwrap();
}