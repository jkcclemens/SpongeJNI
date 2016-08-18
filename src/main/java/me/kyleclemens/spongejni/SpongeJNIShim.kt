package me.kyleclemens.spongejni;

class SpongeJNIShim(val plugin: SpongeJNI) {
    /**
     * Native method called when the shim is ready for external code.
     *
     * The native library should do an initial setup, such as registering listeners and commands, here.
     *
     * @return true if the native library was set up successfully, false if there was an error
     */
    external fun init(): Boolean

}
