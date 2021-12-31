
pub struct FieldData<R> {
    pub extra: R,
    pub ident: Option<syn::Ident>,
    pub ftype: syn::Type,
}

pub fn load_fields<T: darling::FromField, R, F: Fn(T, &Option<syn::Ident>, &syn::Type, usize) -> R>(
    fields: &syn::Fields,
    handle: F,
) -> Vec<FieldData<R>> {
    fields
        .iter()
        .enumerate()
        .map(|(idx, f)| {
            let extra = T::from_field(f).expect("Attrs Info Not found");
            let extra = handle(extra,&f.ident,&f.ty, idx);

            let ident = f.ident.clone();
            let ftype = f.ty.clone();

            FieldData {
                extra,
                ident,
                ftype,
            }
        })
        .collect()
}
