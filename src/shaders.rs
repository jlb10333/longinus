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
