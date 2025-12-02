// RFC 017: Standard VFX Library

pub const FILM_GRAIN: &str = r#"
    uniform shader image;
    uniform float intensity; // 0.0 to 1.0
    uniform float time;      // Auto-injected
    uniform float2 resolution; // Auto-injected

    half4 main(float2 p) {
        half4 color = image.eval(p);
        // Random noise generator
        float noise = fract(sin(dot(p + time * 0.1, float2(12.9898, 78.233))) * 43758.5453);

        // Overlay blend
        float3 grain = float3(noise);
        float3 result = color.rgb + (grain - 0.5) * intensity;

        return half4(result, color.a);
    }
"#;

pub const VIGNETTE: &str = r#"
    uniform shader image;
    uniform float intensity; // 0.0 to 1.0
    uniform float2 resolution; // Auto-injected

    half4 main(float2 p) {
        half4 color = image.eval(p);
        float2 uv = p / resolution;
        uv *=  1.0 - uv.yx;
        float vig = uv.x * uv.y * 15.0;
        vig = pow(vig, intensity);
        return half4(color.rgb * vig, color.a);
    }
"#;

pub const GLITCH_ANALOG: &str = r#"
    uniform shader image;
    uniform float speed;
    uniform float amount;
    uniform float time;
    uniform float2 resolution;

    half4 main(float2 p) {
        float2 uv = p / resolution;
        float t = time * speed;

        // Horizontal tear
        float tear = 0.5 + 0.5 * sin(t * 10.0 + uv.y * 50.0);
        float shift = (tear > 0.95) ? amount : 0.0;

        half4 r = image.eval(p + float2(shift * 10.0, 0.0));
        half4 g = image.eval(p);
        half4 b = image.eval(p - float2(shift * 10.0, 0.0));

        return half4(r.r, g.g, b.b, g.a);
    }
"#;

pub const HALFTONE: &str = r#"
    uniform shader image;
    uniform float dotSize;
    uniform float2 resolution;

    half4 main(float2 p) {
        half4 color = image.eval(p);
        float gray = dot(color.rgb, float3(0.299, 0.587, 0.114));

        float2 center = floor(p / dotSize) * dotSize + dotSize * 0.5;
        float dist = distance(p, center);
        float radius = (dotSize * 0.5) * (1.0 - gray);

        if (dist < radius) {
            return half4(0.0, 0.0, 0.0, color.a); // Black dots
        } else {
            return half4(1.0, 1.0, 1.0, color.a); // White bg
        }
    }
"#;
