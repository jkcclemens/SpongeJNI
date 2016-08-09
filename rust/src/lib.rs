extern crate jni_sys;

use jni_sys::*;
use std::ffi::CString;

macro_rules! java_method {
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident, $($args:expr),*) => {{
      unsafe {
        let genv = $env.as_ref().and_then(|x| x.as_ref()).unwrap();
        let class: jclass = (genv.GetObjectClass)($env, $caller);
        let method_id: jmethodID = (genv.GetMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        let args = vec![ $( jvalue { _data: &$args as *const _ as u64 } ),* ];
        let ret = (genv.$call_using)($env, $caller, method_id, args.as_ptr());
        (genv.DeleteLocalRef)($env, class);
        ret
      }
    }};
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident) => {{
      unsafe {
        let genv = $env.as_ref().and_then(|x| x.as_ref()).unwrap();
        let class: jclass = (genv.GetObjectClass)($env, $caller);
        let method_id: jmethodID = (genv.GetMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        let ret = (genv.$call_using)($env, $caller, method_id);
        (genv.DeleteLocalRef)($env, class);
        ret
      }
    }}
}

pub mod generated_types;

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_SpongeJNIShim_jniShim(env: *mut JNIEnv, this: jobject, game: jobject) -> jboolean {
  // check that game object passed to us by the shim isn't null
  if game.is_null() {
    // return false to shim if it is, indicating an error
    return 0;
  }
  // wrap the game object in a rust struct
  let game = generated_types::Game {
    env: env,
    object: game
  };
  // get the server
  let server = game.get_server();
  // print out the max number of players the server allows
  println!("The max number of players is {}", server.get_max_players());
  // match against Option<world_storage_WorldProperties>
  let world = match server.get_default_world() {
    // if there is a value
    Some(world) => world,
    // if there isn't a value
    None => {
      // print error message
      println!("No default world!");
      // return false to indicate an error
      return 0;
    }
  };
  // print out the default world's seed
  println!("Seed for default world is {}", world.get_seed());
  // return true to the shim, indicating success
  return 1;
}
