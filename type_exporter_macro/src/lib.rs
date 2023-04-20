extern crate proc_macro;

#[proc_macro_derive(TE, attributes(te))]
pub fn type_exporter(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  proc_macro::TokenStream::new()
}
