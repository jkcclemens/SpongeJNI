extern crate classreader;

use std::env::args;
use std::fs::File;
use classreader::{ClassReader, ConstantPoolInfo, Attribute};

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
  public: bool
}

fn main() {
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
    .map(|x| Some(x))
    .unwrap_or(None);
    Method {
      original_name: name.clone(),
      name: name.clone(),
      descriptor: descriptor.clone(),
      signature: signature.cloned(),
      public: method.access_flags & 0x0001 == 1
    }
  })
  .collect();
  println!("{:#?}", Class {
    name: class_name.clone(),
    methods: methods
  })
}
