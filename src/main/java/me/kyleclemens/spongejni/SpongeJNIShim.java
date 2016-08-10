package me.kyleclemens.spongejni;

public class SpongeJNIShim {

    private final SpongeJNI jni;

    SpongeJNIShim(final SpongeJNI jni) {
        this.jni = jni;
    }

    @SuppressWarnings("unused")
    private SpongeJNI getJNI() {
        return this.jni;
    }

    native boolean init();

}
