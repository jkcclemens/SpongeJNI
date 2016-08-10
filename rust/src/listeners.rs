use jni_sys::{JNIEnv, jobject};

use jni::Jni;
use generated_types::*;
use extensions::*;

pub struct Listeners;

impl Listeners {
  pub fn register(jni: &Jni) {
    let listeners = jni.generate_listeners(
      "me.kyleclemens.spongejni.rust.generated.RustyListener",
      &[
        "org.spongepowered.api.event.network.ClientConnectionEvent$Join"
      ]
    );
    jni.get_game().get_event_manager().register_listeners(jni.object, listeners);
  }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_rust_generated_RustyListener_joinReceived(env: *mut JNIEnv, this: jobject, event: jobject) {
  let event = unsafe { event_entity_living_humanoid_TargetHumanoidEvent::from(env, event) };
  let player = event.get_target_entity();
  let user = unsafe { entity_living_player_User::from(player.env, player.object) };
  println!("Rust knows that {} joined... :3", user.get_name().into_rust_string(env));
}
