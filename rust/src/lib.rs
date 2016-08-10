extern crate jni_sys;

macro_rules! java_method {
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident, $($args:expr),*) => {{
      unsafe {
        let class: jclass = ((**$env).GetObjectClass)($env, $caller);
        if class.is_null() { panic!("class was null"); }
        let method_id: jmethodID = ((**$env).GetMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        if method_id.is_null() { panic!("method_id was null"); }
        let args = vec![ $( jvalue { _data: $args as u64 } ),* ];
        let ret = ((**$env).$call_using)($env, $caller, method_id, args.as_ptr());
        ((**$env).DeleteLocalRef)($env, class);
        ret
      }
    }};
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident) => {{
      unsafe {
        let class: jclass = ((**$env).GetObjectClass)($env, $caller);
        if class.is_null() { panic!("class was null"); }
        let method_id: jmethodID = ((**$env).GetMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        if method_id.is_null() { panic!("method_id was null"); }
        let ret = ((**$env).$call_using)($env, $caller, method_id);
        ((**$env).DeleteLocalRef)($env, class);
        ret
      }
    }}
}

macro_rules! static_java_method {
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident, $($args:expr),*) => {{
      unsafe {
        let class: jclass = ((**$env).FindClass)($env, CString::new($caller).unwrap().as_ptr());
        if class.is_null() { panic!("class was null"); }
        let method_id: jmethodID = ((**$env).GetStaticMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        if method_id.is_null() { panic!("method_id was null"); }
        let args = vec![ $( jvalue { _data: $args as u64 } ),* ];
        let ret = ((**$env).$call_using)($env, class, method_id, args.as_ptr());
        ((**$env).DeleteLocalRef)($env, class);
        ret
      }
    }};
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident) => {{
      unsafe {
        let class: jclass = ((**$env).FindClass)($env, CString::new($caller).unwrap().as_ptr());
        if class.is_null() { panic!("class was null"); }
        let method_id: jmethodID = ((**$env).GetStaticMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        if method_id.is_null() { panic!("method_id was null"); }
        let ret = ((**$env).$call_using)($env, class, method_id);
        ((**$env).DeleteLocalRef)($env, class);
        ret
      }
    }}
}

macro_rules! java_field {
    ($env:expr, $caller:expr, $field:expr, $sig:expr, $call_using:ident) => {{
      unsafe {
        let class: jclass = ((**$env).GetObjectClass)($env, $caller);
        let field_id: jfieldID = ((**$env).GetFieldID)($env, class, CString::new($field).unwrap().as_ptr(), CString::new($sig).unwrap().as_ptr());
        let ret = ((**$env).$call_using)($env, $caller, field_id);
        ((**$env).DeleteLocalRef)($env, class);
        ret
      }
    }}
}

pub mod commands;
pub mod listeners;
pub mod jni;
pub mod generated_types;
pub mod extensions;
