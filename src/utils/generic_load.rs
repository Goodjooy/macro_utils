use syn::{ConstParam, Ident, Lifetime, LifetimeDef, TypeParam, TypeParamBound, WhereClause};



pub struct GenericLoad {
    pub type_params: Vec<Ident>,
    pub generic: Vec<proc_macro2::TokenStream>,
    pub where_clause: Vec<proc_macro2::TokenStream>,
}

pub fn load_generic(generics: &syn::Generics) -> GenericLoad {
    // 普通泛型，指代某种类型
    let type_params = generics.type_params();
    // const 泛型，是一个编译器常量
    let const_params = generics.const_params();
    // lifetime 生命周期
    let lifetimes = generics.lifetimes();
    // where 代码块，可选部分
    let where_clause = &generics.where_clause;

    // 对于泛型约束
    /*
    fn xxxx<'p, T, const L :usize> (...)->(...) where T:....
    */
    let type_params = load_type_params(type_params);
    let mut const_params = load_const_params(const_params);
    let lifetimes = load_lifetime(lifetimes);
    let mut where_clause = load_where_clause(where_clause);

    let mut res_type_params = Vec::<Ident>::with_capacity(type_params.len());
    let mut generics = Vec::<proc_macro2::TokenStream>::with_capacity(
        type_params.len() + const_params.len() + lifetimes.len(),
    );
    let mut res_where_clause =
        Vec::<proc_macro2::TokenStream>::with_capacity(where_clause.len() + type_params.len());

    // 先添加where clause
    res_where_clause.append(&mut where_clause);
    // 泛型类型顺序： lifetimes typeparams constparams
    for LifetimeMidData { sign, bounds } in lifetimes {
        let head = quote::quote! {#sign};
        generics.push(head.into());

        if bounds.len() != 0 {
            let bounds_iter = bounds.into_iter().map(|k| k);
            let wh = quote::quote! {#sign : #( #bounds_iter )+* };
            res_where_clause.push(wh.into())
        }
    }

    for TypeParamMidData { sign, bounds } in type_params {
        res_type_params.push(sign.clone());
        generics.push(quote::quote! {#sign}.into());
        if bounds.len() != 0 {
            let bound_iter = bounds.into_iter().map(|i| i);
            res_where_clause.push(quote::quote! { #sign : #( #bound_iter )+* }.into())
        }
    }

    generics.append(&mut const_params);

    GenericLoad {
        generic: generics,
        where_clause: res_where_clause,
        type_params:res_type_params
    }
}
/// const generic 继续保留在泛型中，并在impl头部添加
/// 直接写入为tokenStream 打包后返回
fn load_const_params<'a, I: Iterator<Item = &'a ConstParam>>(
    params: I,
) -> Vec<proc_macro2::TokenStream> {
    params
        .map(|cp| {
            quote::quote! {
                #cp
            }
            .into()
        })
        .collect()
}

struct LifetimeMidData {
    sign: Lifetime,
    bounds: Vec<Lifetime>,
}
// lifetime 生命周期，只提取本身名字，限制条件放到where中
fn load_lifetime<'l, I: Iterator<Item = &'l LifetimeDef>>(lifetimes: I) -> Vec<LifetimeMidData> {
    lifetimes
        .map(|lifetime| {
            let bounds = lifetime.bounds.clone().into_iter().collect::<Vec<_>>();
            let sign = lifetime.lifetime.clone();
            LifetimeMidData { sign, bounds }
        })
        .collect()
}

struct TypeParamMidData {
    sign: Ident,
    bounds: Vec<TypeParamBound>,
}

fn load_type_params<'t, I: Iterator<Item = &'t TypeParam>>(
    type_params: I,
) -> Vec<TypeParamMidData> {
    type_params
        .map(|type_param| {
            let sign = type_param.ident.clone();
            let bounds = type_param.bounds.clone().into_iter().collect();

            TypeParamMidData { sign, bounds }
        })
        .collect()
}

fn load_where_clause(where_clasue: &Option<WhereClause>) -> Vec<proc_macro2::TokenStream> {
    if let Some(wc) = where_clasue {
        wc.predicates
            .clone()
            .into_iter()
            .map(|wp| {
                // all where clause pack to result
                quote::quote! {#wp}.into()
            })
            .collect()
    } else {
        vec![]
    }
}
