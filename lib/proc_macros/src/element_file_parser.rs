use proc_macro2::Ident;
use syn::__private::str;
use syn::parse::{Parse, ParseStream, Peek};
use syn::{Error, Token, Type};

// Skips or consumes tokens from stream until it encounters the specified one
fn skip_until(
	stream: &ParseStream,
	token: impl Peek,
	err: &str,
	mut then: impl FnMut(&ParseStream) -> Result<bool, Error>
) -> Result<(), Error> {
	for i in 0..1000 {
		if !stream.peek(token) {
			stream
				.parse::<proc_macro2::TokenTree>()
				.expect("Cannot skip token");
		}
		else if then(stream)? {
			break;
		}

		if i >= 999 {
			return Err(stream.error(err));
		}
	}

	Ok(())
}

pub struct ElemFile {
	pub id:        u16,
	pub elem_name: String
}

impl Parse for ElemFile {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut id: Option<u16> = None;
		let mut name: String = "".to_owned(); // element name

		for _ in 0..2 {
			skip_until(
				&input,
				Token![const],
				"Couldn't find EL_ const or ID const after 1000 tokens",
				|stream| {
					stream.parse::<Token![const]>()?;
					let cname: Ident = stream.parse()?;
					println!("Found {}", cname);
					if cname.to_string().starts_with("EL_") {
						name = cname.to_string().split_off(3);

						println!("Set {}", name);
						return Ok(true);
					}
					else if cname == "ID" {
						stream.parse::<Token![:]>()?;
						let _: Type = stream.parse()?;
						stream.parse::<Token![=]>()?;
						let val: syn::LitInt = stream.parse()?;

						id = Some(val.base10_parse()?);
						return Ok(true);
					}
					Ok(false)
				}
			)?;
		}

		while !input.is_empty() {
			input.parse::<proc_macro2::TokenTree>().unwrap();
		}

		Ok(ElemFile {
			id:        id.unwrap(),
			elem_name: name
		})
	}
}
