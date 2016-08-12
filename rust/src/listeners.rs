use jni_sys::*;
use std::ffi::CString;

use jni::Jni;
use generated_types::*;

type CollectionValue = data_value_mutable_CollectionValue;
type Event = event_Event;
type Living = entity_living_Living;
type Player = entity_living_player_Player;
type TargetHumanoidEvent = event_entity_living_humanoid_TargetHumanoidEvent;
type User = entity_living_player_User;
type Value = data_value_mutable_Value;

pub struct Listeners;

impl Listeners {
  pub fn register(jni: &Jni) {
    let listeners = jni.generate_listeners(
      "me.kyleclemens.spongejni.rust.generated.RustyListener",
      &[
        "org.spongepowered.api.event.network.ClientConnectionEvent$Join",
        "org.spongepowered.api.event.achievement.GrantAchievementEvent"
      ]
    );
    jni.get_game().get_event_manager().register_listeners(jni.object, listeners);
  }
}

fn set_player_hearts(env: *mut JNIEnv, player: Player) {
  let user = unsafe { User::from(env, player.object) };
  let collection = unsafe { CollectionValue::from(env, user.get_achievement_data().achievements().object) };
  let new_hearts_count = (collection.size() as f64 * 0.5) + 10.0;
  let living = unsafe { Living::from(env, player.object) };
  let max_health = living.get_health_data().max_health();
  let box_double = static_java_method!(env, "java/lang/Double", "valueOf", "(J)Ljava/lang/Double;", CallStaticObjectMethod, new_hearts_count);
  unsafe { Value::from(env, max_health.object) }.set(box_double);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_rust_generated_RustyListener_joinReceived(env: *mut JNIEnv, this: jobject, event: jobject) {
  let event = unsafe { TargetHumanoidEvent::from(env, event) };
  let player = unsafe { Player::from(env, event.get_target_entity().object) };
  set_player_hearts(env, player);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_org_spongepowered_api_event_achievement_GrantAchievementEvent(env: *mut JNIEnv, this: jobject, event: jobject) {
  let raw_event = unsafe { Event::from(env, event) };
  let player_class = unsafe { ((**env).FindClass)(env, CString::new("org/spongepowered/api/entity/living/player/Player").unwrap().as_ptr()) };
  let player = match raw_event.get_cause().first(player_class) {
    Some(u) => unsafe { Player::from(env, u) },
    None => return
  };
  set_player_hearts(env, player);
}
