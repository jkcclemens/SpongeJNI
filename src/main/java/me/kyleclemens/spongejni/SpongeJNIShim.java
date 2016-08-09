package me.kyleclemens.spongejni;

import org.spongepowered.api.Game;
import org.spongepowered.api.event.Event;
import org.spongepowered.api.event.Listener;

public class SpongeJNIShim {

    private final SpongeJNI jni;

    SpongeJNIShim(final SpongeJNI jni) {
        this.jni = jni;
    }

    private SpongeJNI getJNI() {
        return this.jni;
    }

    native boolean init(final Game game);

    @Listener
    public native void eventReceived(final Event event);

}
