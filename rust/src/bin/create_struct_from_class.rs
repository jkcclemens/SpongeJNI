extern crate classreader;
extern crate inflector;

use inflector::Inflector;
use classreader::{ClassReader, ConstantPoolInfo, Attribute};
use std::env::args;
use std::fs::File;
use std::collections::HashMap;

#[derive(Debug)]
struct Class {
  name: String,
  methods: Vec<Method>
}

#[derive(Debug)]
struct Method {
  original_name: String,
  name: String,
  descriptor: String,
  signature: Option<String>,
  is_public: bool,
  is_static: bool
}

fn generate_class() -> Class {
  let class_path = args().nth(1).expect("no path provided");
  let mut file = File::open(class_path).expect("could not open path");
  let class = ClassReader::new_from_reader(&mut file).expect("could not read valid class info");
  let class_name = class.constant_pool.get((class.this_class - 1) as usize).expect("could not get class from constant pool");
  let class_name = if let &ConstantPoolInfo::Class(ref index) = class_name {
    if let &ConstantPoolInfo::Utf8(ref name) = class.constant_pool.get((index - 1) as usize).expect("could not get class name from constant pool") {
      name
    } else {
      panic!("class name was not a utf8 string");
    }
  } else {
    panic!("class was not a Class");
  };
  let methods = class.methods.iter()
  .map(|method| {
    let name = if let &ConstantPoolInfo::Utf8(ref name) = class.constant_pool.get((method.name_index - 1) as usize).expect("no name in constant pool for method") {
      name
    } else {
      panic!("method name in constant pool was not a utf8 string");
    };
    let descriptor = if let &ConstantPoolInfo::Utf8(ref descriptor) = class.constant_pool.get((method.descriptor_index - 1) as usize).expect("no descriptor in constant pool for method") {
      descriptor
    } else {
      panic!("method descriptor in constant pool was not a utf8 string");
    };
    let signature = method.attributes
      .iter()
      .map(|attr| match attr {
        &Attribute::Signature { signature_index: index } => {
          if let &ConstantPoolInfo::Utf8(ref signature) = class.constant_pool.get((index - 1) as usize).expect("no signature in constant pool") {
            Some(signature)
          } else {
            None
          }
        },
        _ => None
      })
      .flat_map(|x| x)
      .nth(0)
      .map(Some)
      .unwrap_or(None);
    Method {
      original_name: name.clone(),
      name: name.clone(),
      descriptor: descriptor.clone(),
      signature: signature.cloned(),
      is_public: method.access_flags & 0x0001 == 0x0001,
      is_static: method.access_flags & 0x0008 == 0x0008
    }
  })
  .collect();
  Class {
    name: class_name.clone(),
    methods: methods
  }
}

fn get_rust_type<'a>(c: char) -> &'a str {
  match c {
    'B' => "i8",
    'C' => "char",
    'D' => "f64",
    'F' => "f32",
    'I' => "i32",
    'J' => "i64",
    'S' => "i16",
    'Z' => "bool",
    _ => panic!("unsupported param type: {}", c)
  }
}

fn get_return_type<'a>(return_type: &'a str) -> Option<String> {
  match return_type.chars().nth(0).expect("no first char") {
    'B' => Some("i8".to_owned()),
    'C' => Some("char".to_owned()),
    'D' => Some("f64".to_owned()),
    'F' => Some("f32".to_owned()),
    'I' => Some("i32".to_owned()),
    'J' => Some("i64".to_owned()),
    '[' => Some({
      let array_type = get_return_type(&return_type[1..]);
      format!("Vec<{}>", array_type.unwrap_or("()".to_owned()))
     }),
    'L' => Some({
      let class_name = &return_type[1..];
      if !class_name.starts_with("org/spongepowered/api") {
        if class_name == "java/util/Optional;" {
          String::from("Option")
        } else {
          String::from("jobject")
        }
      } else {
        sanitize_class_name(&class_name).replace(";", "")
      }
    }),
    'S' => Some("i16".to_owned()),
    'Z' => Some("bool".to_owned()),
    'V' => None,
    'T' => Some("jobject".to_owned()), // FIXME: generics
    _ => panic!("unsupported return type: {}", return_type)
  }
}

fn get_param_types<'a>(params: &'a str) -> Vec<String> {
  let mut types = Vec::new();
  let mut chars = params.chars();
  while let Some(c) = chars.next() {
    if c == '[' {
      let array_type = chars.next().expect("no array type in param");
      let rust_type = if array_type == 'L' {
        let mut class_name = String::new();
        while let Some(next) = chars.next() {
          if next == ';' {
            break;
          }
          class_name.push(next);
        }
        let name = if class_name.starts_with("org/spongepowered/api") {
          sanitize_class_name(&class_name)
        } else {
          String::from("jobject")
        };
        format!("Vec<{}>", name)
      } else {
        get_rust_type(array_type).to_owned()
      };
      types.push(rust_type);
      continue;
    }
    if c == 'L' {
      let mut class_name = String::new();
      while let Some(next) = chars.next() {
        if next == ';' {
          break;
        }
        class_name.push(next);
      }
      let name = if class_name.starts_with("org/spongepowered/api") {
        sanitize_class_name(&class_name)
      } else {
        String::from("jobject")
      };
      types.push(name);
      continue;
    }
    types.push(get_rust_type(c).to_owned());
  }
  types
}

fn sanitize_class_name<'a>(class_name: &'a str) -> String {
  if class_name.contains("gencore") {
    return String::from("jobject");
  }
  class_name.replace("org/spongepowered/api/", "").replace("/", "_").replace("$", "_")
}

fn create_params<'a>(descriptor: &'a str, signature: Option<String>, is_static: bool) -> String {
  let mut string = String::from("(");
  if is_static {
    string.push_str("env: *mut JNIEnv");
  } else {
    string.push_str("&self");
  }
  let split: Vec<&str> = descriptor[1..].split(')').collect();
  let params = &split.get(0).expect("no params in descriptor");
  let return_type = split.get(1).expect("no return type in descriptor");
  let mut param_num = 0;
  for param in get_param_types(params) {
    param_num += 1;
    string.push_str(&format!(", param_{}: {}", param_num, param));
  }
  string.push(')');
  let return_type = get_return_type(return_type);
  if return_type.is_some() {
    let return_type = return_type.unwrap();
    let return_type = if return_type == "Option" {
      let optional_return_type = get_optional_return_type(signature.clone().expect("optional return type without signature"));
      format!("{}<{}>", return_type, optional_return_type)
    } else {
      return_type
    };
    string.push_str(&format!(" -> {}", return_type));
  }
  string
}

fn get_call_method<'a>(descriptor: &'a str, params: &'a str, is_static: bool) -> String {
  let num_params = params.split(",").collect::<Vec<_>>().len() - 1;
  let return_type = descriptor.split(')').last().expect("no return type");
  let first_letter = return_type.chars().nth(0).expect("no first letter of return type");
  let mut call_method = match first_letter {
    'B' => "CallByteMethod",
    'C' => "CallCharMethod",
    'D' => "CallDoubleMethod",
    'F' => "CallFloatMethod",
    'I' => "CallIntMethod",
    'J' => "CallLongMethod",
    'S' => "CallShortMethod",
    'Z' => "CallBooleanMethod",
    'L' => "CallObjectMethod",
    'V' => "CallVoidMethod",
    '[' => "CryInside", // FIXME
    _ => panic!("unsupported call method: {}", first_letter)
  }.to_owned();
  if num_params > 0 {
    call_method.push('A');
  }
  if is_static {
    call_method = call_method.replace("Call", "CallStatic");
  }
  // FIXME: varargs
  call_method
}

fn get_optional_return_type(signature: String) -> String {
  let optional_type = signature;
  let optional_type = optional_type.split("Optional<").nth(1).expect("no type signature");
  let mut optional_type = optional_type.split(">").nth(0).expect("invalid signature");
  loop {
    let mut split = optional_type.split("<");
    let zero = split.nth(0);
    let one = split.nth(0);
    if one.is_none() {
      break;
    }
    optional_type = zero.expect("bad split");
  }
  let optional_type = if optional_type.starts_with('+') {
    &optional_type[1..]
  } else if optional_type == "*" {
    "L*;" // FIXME: generics
  } else {
    optional_type
  };
  get_return_type(optional_type).expect("no return type from signature")
}

fn create_method<'a>(class_name: &'a str, method: &Method) -> String {
  let mut string = String::new();
  let snake_case_name = method.name.to_snake_case();
  let rust_params = create_params(&method.descriptor, method.signature.clone(), method.is_static);
  let map_params = rust_params
    .split(", ")
    .map(|x| {
      let mut x = x.split(")").next().expect("invalid params 0");
      if x.starts_with('(') {
        x = &x[1..];
      }
      let mut split = x.split(": ");
      let name = split.next().expect("invalid param 1");
      if name == "self" || name == "&self" {
        return None;
      }
      let t = split.next().expect("invalid param 2");
      if name == "env" && t == "*mut JNIEnv" {
        None
      } else {
        Some((name, t))
      }
    })
    .flat_map(|x| x)
    .collect::<Vec<_>>();
  let mut unimplemented = false;
  if map_params.iter().any(|x| x.1.starts_with("Vec<")) {
    unimplemented = true;
  }
  let call_method = get_call_method(&method.descriptor, &rust_params, method.is_static);
  string.push_str(&format!("\n  pub fn {}", snake_case_name));
  string.push_str(&rust_params);
  string.push_str(" {\n");
  string.push_str("    ");
  if call_method.contains("CryInside") {
    unimplemented = true;
  }
  if unimplemented {
    string.push_str("unimplemented!();\n  }");
    return string;
  }
  if call_method.contains("ObjectMethod") {
    string.push_str("let ret = ");
  }
  let (s, macro_prefix, caller) = if method.is_static {
    ("", "static_", format!("\"{}\"", class_name))
  } else {
    ("self.", "", "self.object".to_owned())
  };
  string.push_str(&format!(r#"{}java_method!({}env, {}, "{}", "{}", {}"#, macro_prefix, s, caller, method.original_name, method.descriptor, call_method));
  if !map_params.is_empty() {
    for param in map_params {
      let (name, t) = param;
      let obj = if !t.starts_with("Vec<") && t.split("_").last().expect("invalid type").chars().next().expect("invalid type 2").is_uppercase() {
        ".object"
      } else {
        ""
      };
      string.push_str(&format!(", {}{}", name, obj));
    }
  }
  string.push_str(")");
  if call_method.contains("BooleanMethod") {
    string.push_str(" == 1");
  }
  if call_method.contains("CharMethod") {
    string.push_str(" as u8 as char"); // FIXME
  }
  if call_method.contains("VoidMethod") || call_method.contains("ObjectMethod") {
    string.push_str(";");
  }
  string.push_str("\n");
  if call_method.contains("ObjectMethod") {
    string.push_str(&format!("    if ret.is_null() {{ panic!(\"{}#{} was null\") }}\n", class_name, method.original_name));
    let return_type = rust_params.split(" -> ").last().expect("no return type");
    if return_type == "jobject" {
      string.push_str("    ret\n");
    } else if return_type.starts_with("Option") {
      let optional_return_type = get_optional_return_type(method.signature.clone().expect("optional return type without signature"));
      string.push_str(&format!(r#"    let unwrapped = java_method!({}env, ret, "orElse", "(Ljava/lang/Object;)Ljava/lang/Object;", CallObjectMethodA, ::std::ptr::null() as *const jobject);"#, s));
      string.push_str("\n    if unwrapped.is_null() { None } else { ");
      if optional_return_type == "jobject" {
        string.push_str("Some(unwrapped) }\n");
      } else {
        string.push_str(&format!("Some({} {{ env: {}env, object: unwrapped }}) }}\n", optional_return_type, s));
      }
    } else {
      string.push_str(&format!("    {} {{ env: {}env, object: ret }}\n", return_type, s));
    }
  }
  string.push_str("  }\n");
  string
}

fn create_struct(class: Class) -> String {
  let mut string = String::new();
  let end_name = sanitize_class_name(&class.name);
  if class.name.split('/').last().expect("no end class name") == "package-info" {
    return String::new();
  }
  string.push_str(&format!("#[derive(Debug)]\npub struct {} {{\n  pub env: *mut JNIEnv,\n  pub object: jobject\n}}", end_name));
  let mut methods: Vec<Method> = class.methods.into_iter().filter(|m| m.is_public).collect();
  string.push_str(&format!("\n\nimpl {} {{", end_name));
  string.push_str("\n  pub unsafe fn from(env: *mut JNIEnv, object: jobject) -> Self {\n");
  string.push_str(&format!("    {} {{\n", end_name));
  string.push_str("      env: env,\n      object: object\n    }\n  }\n");
  let mut method_count = HashMap::new();
  method_count.insert("from".to_owned(), 1);
  for method in methods.iter_mut() {
    if method.name == "<init>" {
      method.name = String::from("new");
      method.is_static = true;
      method.descriptor = format!("{}L{}", &method.descriptor[..method.descriptor.len() - 1], class.name);
    }
    if method.name == "type" {
      method.name = String::from("type_");
    }
    if method.name == "match" {
      method.name = String::from("match_");
    }
    if method.name == "override" {
      method.name = String::from("override_");
    }
    // type, match
    let entry = method_count.entry(method.name.clone()).or_insert(0);
    if entry != &0 {
      method.name = format!("{}{}", method.name, entry);
    }
    string.push_str(&create_method(&class.name, method));
    *entry += 1;
  }
  string.push_str("\n}");
  string.push_str("\n");
  string
}

fn main() {
  let class = generate_class();
  println!("{}", create_struct(class));
}
