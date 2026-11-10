//! Internal host calls retain Win64 on x86 and use C on native ARM.
use proc_macro::{Literal, TokenStream, TokenTree};
fn replace_abi(input: TokenStream) -> TokenStream {
    input
        .into_iter()
        .map(|token| match token {
            TokenTree::Literal(ref value) if value.to_string() == "\"win64\"" => {
                TokenTree::Literal(Literal::string("C"))
            }
            other => other,
        })
        .collect()
}
#[proc_macro_attribute]
pub fn host_abi(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let mut result: TokenStream = "#[cfg(target_arch=\"x86_64\")]".parse().unwrap();
    result.extend(item.clone());
    result.extend(
        "#[cfg(target_arch=\"aarch64\")]"
            .parse::<TokenStream>()
            .unwrap(),
    );
    result.extend(replace_abi(item));
    result
}
/// `(arguments)->return; address` creates a host-call pointer, with the same
/// signature/ABI used by host_abi. The address must be a matching trusted shim.
#[proc_macro]
pub fn host_call(input: TokenStream) -> TokenStream {
    let mut signature = TokenStream::new();
    let mut address = TokenStream::new();
    let mut after = false;
    for token in input {
        if !after && matches!(&token,TokenTree::Punct(p) if p.as_char()==';') {
            after = true;
            continue;
        }
        if after {
            address.extend([token]);
        } else {
            signature.extend([token]);
        }
    }
    assert!(after, "host_call requires signature; address");
    format!("{{ #[cfg(target_arch=\"x86_64\")] type Call=unsafe extern \"win64\" fn{signature}; #[cfg(target_arch=\"aarch64\")] type Call=unsafe extern \"C\" fn{signature}; unsafe {{std::mem::transmute::<usize,Call>({address})}} }}").parse().unwrap()
}

/// Materialize the PE32 service modules for an ARM host and translate explicit
/// raw-pointer casts through the biased guest mapping. Numeric DWORDs, sizes,
/// handles and guest-visible pointer fields are deliberately left unchanged.
#[proc_macro_attribute]
pub fn native_runtime(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    use quote::quote;
    use std::path::Path;
    use syn::visit_mut::VisitMut;
    fn modules(items: &mut [syn::Item], directory: &Path) {
        for item in items {
            if let syn::Item::Mod(module) = item {
                if let Some((_, items)) = &mut module.content {
                    modules(items, directory);
                    continue;
                }
                let explicit = module.attrs.iter().find_map(|a| {
                    if a.path().is_ident("path") {
                        if let syn::Meta::NameValue(v) = &a.meta
                            && let syn::Expr::Lit(l) = &v.value
                            && let syn::Lit::Str(s) = &l.lit
                        {
                            return Some(s.value());
                        }
                        None
                    } else {
                        None
                    }
                });
                let path =
                    directory.join(explicit.unwrap_or_else(|| format!("{}.rs", module.ident)));
                let mut file =
                    syn::parse_file(&std::fs::read_to_string(&path).unwrap_or_else(|e| {
                        panic!("read native service module {}: {e}", path.display())
                    }))
                    .expect("valid original service Rust");
                modules(&mut file.items, path.parent().unwrap());
                module.attrs.retain(|a| !a.path().is_ident("path"));
                module.attrs.extend(file.attrs);
                module.content = Some((Default::default(), file.items));
                module.semi = None;
            }
        }
    }
    let mut module = syn::parse_macro_input!(item as syn::ItemMod);
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("consumer manifest");
    let path = Path::new(&manifest).join("src/runtime32.rs");
    let mut file = syn::parse_file(&std::fs::read_to_string(&path).expect("runtime source"))
        .expect("valid runtime");
    modules(&mut file.items, path.parent().unwrap());
    for item in &mut file.items {
        Pointers.visit_item_mut(item);
    }
    module.attrs.extend(file.attrs);
    module.content = Some((Default::default(), file.items));
    module.semi = None;
    quote!(#module).into()
}

use quote::quote;
use syn::visit_mut::VisitMut;
struct Pointers;
impl VisitMut for Pointers {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, expr);
        if let syn::Expr::Cast(cast) = expr
            && matches!(&*cast.ty, syn::Type::Ptr(_))
        {
            let value = &cast.expr;
            let ty = &cast.ty;
            *expr =
                syn::parse_quote!(crate::runtime32::host_address((#value as #ty) as usize) as #ty);
        }
    }
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        use syn::parse::Parser;
        if let Ok(mut args) =
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
                .parse2(mac.tokens.clone())
        {
            for arg in &mut args {
                self.visit_expr_mut(arg);
            }
            mac.tokens = quote!(#args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guest_casts_preserve_numeric_values_and_reference_coercions() {
        let mut item: syn::ItemFn = syn::parse_quote! {fn translate(address:u32,source:&u8){
            let handle=address as usize;let size=address as u32;
            let pointer=address as *mut u8;let reference=source as *const u8;
            check!(address as *const u32,source as *const u8);
        }};
        Pointers.visit_item_fn_mut(&mut item);
        let text = quote!(#item).to_string();
        assert_eq!(text.matches("host_address").count(), 4);
        assert!(text.contains("handle = address as usize"));
        assert!(text.contains("size = address as u32"));
        assert!(text.contains("source as * const u8"));
        // Every original expression is evaluated once, even when it has effects.
        let mut e: syn::Expr = syn::parse_quote!(next_guest() as *mut u8);
        Pointers.visit_expr_mut(&mut e);
        assert_eq!(quote!(#e).to_string().matches("next_guest").count(), 1);
    }
}
