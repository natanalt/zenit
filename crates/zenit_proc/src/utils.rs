//use syn::{PathArguments, Type};
//
///// Extracts `T` out of `Expected<T>`, if possible
//pub fn extract_generic_type<'a>(ty: &'a Type, expected: &str) -> Option<&'a Type> {
//    // I kinda hate this
//    Some(ty)
//        .and_then(|ty| match ty {
//            Type::Path(tp) => Some(tp),
//            _ => None,
//        })
//        .filter(|tp| tp.path.segments.len() == 1)
//        .map(|tp| tp.path.segments.first().unwrap())
//        .filter(|seg| seg.ident.to_string() == expected)
//        .and_then(|seg| match &seg.arguments {
//            PathArguments::AngleBracketed(ab) => Some(ab),
//            _ => None,
//        })
//        .filter(|ab| ab.args.len() == 1)
//        .map(|ab| ab.args.first().unwrap())
//        .and_then(|ga| match ga {
//            syn::GenericArgument::Type(ty) => Some(ty),
//            _ => None,
//        })
//}
