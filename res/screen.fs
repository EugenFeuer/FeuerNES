precision mediump float;

uniform float uTime;
uniform sampler2D uScreenTex;
varying highp vec2 vTexCoord;

void main() {
    gl_FragColor = texture2D(uScreenTex, vTexCoord) * 5.0;
}