struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct InputUniform {
    effect: i32,
    fill_mode: i32,
    window_ratio: f32,
    img_ratio: f32
};

@group(1) @binding(0)
var<uniform> input_uniform: InputUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    if (input_uniform.effect == 1) { 
        out.tex_coords = vec2<f32>(1.0f - in.tex_coords[0], in.tex_coords[1]); // flip horizontally
    } else {
        out.tex_coords = in.tex_coords;
    }

    // if taller than wide then shorten.
    // 0,0 becomes 0,0+x and 1,1 becomes 1,1-x
    // if wider than tall then squeeze
    // 0,0 becomes 0+x,0 & 1,1 becomes 1-x,1

    /*let x = 0.25;

    if (in.tex_coords[0] == 0f) {
        out.tex_coords = vec2<f32>(0f - x, in.tex_coords[1]);
    } else if (in.tex_coords[0] == 1f) {
        out.tex_coords = vec2<f32>(1f + x, in.tex_coords[1]);
    }*/


    switch input_uniform.fill_mode {
        case 0: { 
            if (input_uniform.img_ratio < input_uniform.window_ratio) { // image taller than display (squashed - make quad narrower)
                out.position = vec4<f32>(in.pos[0] / input_uniform.window_ratio * input_uniform.img_ratio, in.pos[1], 0f, 1f);
            } else { // display taller than image (stretched - make quad shorter)
                out.position = vec4<f32>(in.pos[0], in.pos[1] * input_uniform.window_ratio / input_uniform.img_ratio, 0f, 1f);
            }
        }
        case 1: {
            if (input_uniform.img_ratio < input_uniform.window_ratio) { // image taller than display (squashed - make quad narrower)
                out.position = vec4<f32>(in.pos[0], in.pos[1] * input_uniform.window_ratio / input_uniform.img_ratio, 0f, 1f);

            } else { // display taller than image (stretched - make quad shorter)
                out.position = vec4<f32>(in.pos[0] / input_uniform.window_ratio * input_uniform.img_ratio, in.pos[1], 0f, 1f);

            }
        }
        default {
            out.position = vec4<f32>(in.pos, 0f, 1f);
        }
    }
    
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pixel: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let avg: f32 = (pixel.r + pixel.g + pixel.b) / 3f;

    switch input_uniform.effect {
        case 1: {
            pixel = vec4<f32>(0.5f - avg, 0.5f - avg, 0.5f - avg, 1f); //inverted greyscale
        }
        case 0: {
            pixel = vec4<f32>(1f, 1f, 1f, avg); 
        }
        default {
            // regular image
        }
        case 2: {
            pixel = vec4<f32>(0.86275, 0.79608, 0.89020, avg); 
        }
        case 3: {
            pixel = vec4<f32>(0.3, avg, avg, 1f);
        }
        case 4: {
            pixel = vec4<f32>(avg, avg, 1f, 1f);
        }
        case 5: {
            pixel = vec4<f32>(0f, avg, 0f, 1f);
        }
    }

    return pixel;
}