package dev.nanocode;
import java.nio.charset.StandardCharsets;
/** Thread-safe native codec. Supply packed grayscale; capture and permissions belong to the app. */
public final class NanoCode {
    static { System.loadLibrary("nanocode_jni"); }
    private NanoCode() {}
    public static String svg(byte[] bytes) { return svg(bytes, 0, 0x00a6a1, 0x77dddd, 0xffffff); }
    public static native String svg(byte[] bytes, int format, int one, int zero, int background);
    /** Returns null only when no valid code was found; invalid arguments throw. */
    public static native Detection scan(byte[] gray, int width, int height);
    public static native int paletteScore(int one, int zero, int background);
    public static native int[] backgrounds(int one, int zero);
    public static native int[] secondaryColors(int one, int background);
    public static native int paletteFlags(int one, int zero, int background);
    public static PaletteAssessment assess(int one, int zero, int background) {
        return new PaletteAssessment(paletteScore(one, zero, background), paletteFlags(one, zero, background));
    }
    public static final class PaletteAssessment {
        public final int score, flags;
        public final boolean allowed, recommended;
        private PaletteAssessment(int score, int flags) {
            this.score = score; this.flags = flags;
            this.allowed = flags == 0; this.recommended = allowed && score >= 55;
        }
    }
    public static final class Detection {
        public final byte[] bytes;
        public final int format, corrected;
        public final double[] corners;
        public Detection(byte[] bytes, int format, int corrected, double[] corners) {
            this.bytes = bytes; this.format = format; this.corrected = corrected; this.corners = corners;
        }
        public String text() { return new String(bytes, StandardCharsets.UTF_8); }
    }
}
