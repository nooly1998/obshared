extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// 一个属性宏，用于在函数执行前后添加日志，并记录执行时间。
///
/// 用法:
/// ```rust
/// use trace_func::instrument;
///
/// #[instrument]
/// fn my_function(arg: u32) -> u32 {
///     // ... 函数体 ...
///     arg * 2
/// }
/// ```
#[proc_macro_attribute]
pub fn instrument(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析输入的 TokenStream 为一个函数项 (ItemFn)
    let input_fn = parse_macro_input!(item as ItemFn);

    // 提取函数的各个部分
    let vis = &input_fn.vis;        // 可见性 (e.g., pub)
    let sig = &input_fn.sig;        // 函数签名 (包括 fn 关键字, 函数名, 参数, 返回类型等)
    let block = &input_fn.block;    // 函数体 (包含 `{` 和 `}` 的代码块)
    let attrs = &input_fn.attrs;    // 函数上的其他属性 (例如 #[test], #[allow(...)])

    let fn_name = &sig.ident;                // 函数名标识符
    let fn_name_str = fn_name.to_string();   // 函数名字符串

    // 判断函数是否是 async 函数
    let is_async = sig.asyncness.is_some();
    let fn_type_log_str = if is_async { "async function" } else { "function" };

    // 构建新的函数体
    // 我们将原始函数体包裹在日志和计时逻辑中
    let new_body = quote! {
        {
            // 使用 ::std::time::Instant 来避免在目标代码中需要 `use std::time::Instant;`
            // println! 通常在预导入 (prelude) 中，所以一般不需要完全限定路径。
            println!("[START] Executing {}: {}...", #fn_type_log_str, #fn_name_str);
            let __instrument_start_time = ::std::time::Instant::now();

            // 执行原始函数体并捕获其结果
            // #block 会展开为原始函数的代码块 `{ ... }`
            // `let result = { original_body };` 模式可以正确处理返回值和 `()`
            let __instrument_result = #block;

            let __instrument_duration = __instrument_start_time.elapsed();
            println!("[END] {}: {} executed in {:?}", #fn_type_log_str, #fn_name_str, __instrument_duration);

            // 返回原始结果
            __instrument_result
        }
    };

    // 重新构建整个函数，使用新的函数体
    // #(#attrs)* 用于保留函数上的其他属性
    let output = quote! {
        #(#attrs)*
        #vis #sig {
            #new_body
        }
    };

    // 将生成的 TokenStream 返回
    output.into()
}
