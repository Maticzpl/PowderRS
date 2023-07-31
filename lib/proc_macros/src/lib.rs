extern crate proc_macro;

mod element_file_parser;

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fs};

use element_file_parser::ElemFile;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, LitStr};

/// For all files in provided directory except for mod.rs
/// Will import elements and validate their IDs
#[proc_macro]
pub fn import_elements(stream: TokenStream) -> TokenStream {
	let input = parse_macro_input!(stream as LitStr);
	let path = format!(
		"{}/{}",
		env::current_dir().unwrap().to_str().unwrap(),
		input.value()
	);

	let files: Vec<PathBuf> = fs::read_dir(path.clone())
		.unwrap_or_else(|_| panic!("Can't find dir {}", path))
		.map(|file| file.unwrap().path())
		.collect();

	let mut pieces: Vec<proc_macro2::TokenStream> = vec![];
	let mut ids: HashMap<u16, String> = HashMap::new();

	for file in files {
		if file.is_file() && file.file_name().unwrap().to_str().unwrap() != "mod.rs" {
			println!("Checking {}", file.file_name().unwrap().to_str().unwrap());

			let contents =
				TokenStream::from_str(&fs::read_to_string(file).expect("Can't read file"))
					.expect("Invalid token stream");
			let parsed = parse_macro_input!(contents as ElemFile); // not a macro input :P


			let name = Ident::new(
				format!("EL_{}", parsed.elem_name).as_str(),
				Span::call_site()
			);
			let name_lower =
				Ident::new(&parsed.elem_name.as_str().to_lowercase(), Span::call_site());

			if ids.contains_key(&parsed.id) {
				panic!(
					"Duplicate ID {} in {} and {}",
					parsed.id,
					ids.get(&parsed.id).unwrap(),
					name
				);
			}
			ids.insert(parsed.id, name.to_string());

			pieces.push(quote! {
				mod #name_lower;
				pub use #name_lower::#name;
			})
		}
	}

	let mut expanded = proc_macro2::TokenStream::new();

	for piece in pieces {
		expanded.append_all(piece);
	}

	TokenStream::from(expanded)
}
