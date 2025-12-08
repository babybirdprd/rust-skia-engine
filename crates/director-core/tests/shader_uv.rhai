// Tests automatic uniform injection (u_resolution, u_time) and basic shader compilation

let movie = new_director(200, 200, 30);
let scene = movie.add_scene(2.0);

// Create a box that fills the screen
let box = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#000000"
});

// Apply a shader that uses u_resolution to create a UV gradient
// Red = x / width, Green = y / height
let shader_code = "uniform float2 u_resolution; uniform float u_time; uniform float2 center; half4 main(float2 p) { float2 uv = p / u_resolution; float blue = 0.0; if (u_time > 0.0) { blue = 0.5; } float d = length(p - center); if (d < 20.0) { return half4(1, 1, 1, 1); } return half4(uv.x, uv.y, blue, 1); }";

let effect = box.apply_effect("shader", #{
    code: shader_code,
    uniforms: #{
        center: [100.0, 100.0]
    }
});

// Animate the vector uniform "center"
// Move from 100,100 to 0,0 over 2 seconds
effect.animate("center", [100.0, 100.0], [0.0, 0.0], 2.0, "linear");

movie
