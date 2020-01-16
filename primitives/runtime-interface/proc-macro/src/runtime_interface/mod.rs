// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use crate::utils::generate_runtime_interface_include;

use proc_macro2::{Span, TokenStream};

use syn::{Ident, ItemTrait, Result};

use inflector::Inflector;

use quote::quote;

mod bare_function_interface;
mod host_function_interface;
mod trait_decl_impl;

/// Custom keywords supported by the `runtime_interface` attribute.
pub mod keywords {
	// Custom keyword `wasm_only` that can be given as attribute to [`runtime_interface`].
	syn::custom_keyword!(wasm_only);
}

/// Implementation of the `runtime_interface` attribute.
///
/// It expects the trait definition the attribute was put above and if this should be an wasm only
/// interface.
pub fn runtime_interface_impl(trait_def: ItemTrait, is_wasm_only: bool) -> Result<TokenStream> {
	let bare_functions = bare_function_interface::generate(&trait_def, is_wasm_only)?;
	let crate_include = generate_runtime_interface_include();
	let mod_name = Ident::new(
		&trait_def.ident.to_string().to_snake_case(),
		Span::call_site(),
	);
	let trait_decl_impl = trait_decl_impl::process(&trait_def, is_wasm_only)?;
	let host_functions = host_function_interface::generate(&trait_def, is_wasm_only)?;
	let vis = trait_def.vis;
	let attrs = &trait_def.attrs;

	let res = quote! {
		#( #attrs )*
		#vis mod #mod_name {
			use super::*;
			#crate_include

			#bare_functions

			#trait_decl_impl

			#host_functions
		}
	};

	Ok(res)
}
