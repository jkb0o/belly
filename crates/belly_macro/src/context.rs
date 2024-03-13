use proc_macro2::TokenStream;
use quote::*;

pub struct Context {
    core_path: TokenStream,
    is_interal: bool,
}

impl Context {
    pub fn new() -> Context {
        let mut context = Context {
            core_path: quote! { ::belly_core },
            is_interal: true,
        };
        let Some(manifest_path) = std::env::var_os("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .map(|mut path| {
                path.push("Cargo.toml");
                path
            })
        else {
            return context;
        };
        let Ok(manifest) = std::fs::read_to_string(&manifest_path) else {
            return context;
        };
        let Ok(manifest) = toml::from_str::<toml::map::Map<String, toml::Value>>(&manifest) else {
            return context;
        };

        let Some(pkg) = manifest.get("package") else {
            return context;
        };
        let Some(pkg) = pkg.as_table() else {
            return context;
        };
        let Some(pkg) = pkg.get("name") else {
            return context;
        };
        let Some(pkg) = pkg.as_str() else {
            return context;
        };
        if pkg.trim() == "belly_widgets" {
            context.core_path = quote! { ::belly_core };
        } else {
            context.core_path = quote! { ::belly::core };
            context.is_interal = false;
        };
        context
    }
    pub fn core_path(&self) -> &TokenStream {
        &self.core_path
    }
}
