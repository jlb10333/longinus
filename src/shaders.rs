pub const DISSOLVE_FRAGMENT_SHADER: &str = r#"#version 100
precision lowp float;

varying vec2 uv;

uniform sampler2D Texture;

uniform sampler2D NoiseTexture;
uniform float Progress;

uniform float PixelsX;
uniform float PixelsY;

uniform vec2 TextureSize;

void main() {
    vec2 subsetUv = vec2((TextureSize.x * uv.x) / PixelsX, (TextureSize.y * uv.y) / PixelsY);

    vec2 adjustedUv = mod(vec2(floor(uv.x * TextureSize.x) / PixelsX, floor(uv.y * TextureSize.y) / PixelsY), 1.0);

    float noiseValue = texture2D(NoiseTexture, adjustedUv).r;

    if (noiseValue < Progress) {
        discard;
    }

    vec4 res = texture2D(Texture, uv);

    if (res.a < 0.1) {
        discard;
    }

    gl_FragColor = res;
}
"#;

pub const COLOR_MAP_FRAGMENT_SHADER: &str = r#"#version 100
precision lowp float;

varying vec2 uv;

uniform sampler2D _ScreenTexture;

uniform vec4 COLOR_1;
uniform vec4 COLOR_2;
uniform vec4 COLOR_3;
uniform vec4 COLOR_4;

uniform vec4 MAPPED_COLOR_1;
uniform vec4 MAPPED_COLOR_2;
uniform vec4 MAPPED_COLOR_3;
uniform vec4 MAPPED_COLOR_4;

void main() {
    vec2 adjusted_uv = vec2(uv.x, 1.0 - uv.y);
    vec4 cur = texture2D(_ScreenTexture, adjusted_uv);

    if (cur.rgb == COLOR_1.rgb) {
        cur = MAPPED_COLOR_1;
    } else if (cur.rgb == COLOR_2.rgb) {
        cur = MAPPED_COLOR_2;
    } else if (cur.rgb == COLOR_3.rgb) {
        cur = MAPPED_COLOR_3;
    } else if (cur.rgb == COLOR_4.rgb) {
        cur = MAPPED_COLOR_4;
    }

    gl_FragColor = cur;
}
"#;

pub const IDENTITY_VERTEX_SHADER: &str = r#"#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
"#;
