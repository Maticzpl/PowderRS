#version 330 core
out vec4 FragColor;

in vec2 v_tex_coords;
uniform sampler2D tex;

vec4 lerp(vec4 a, vec4 b, float t)
{
    return ((b - a) * t) + a;
}

float rand(vec2 co){
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // i doubt i will do particle shaders this way but idk
    vec4 col = texture(tex, v_tex_coords);
    int type = int(col.a * 255);
    col.a = 1 * int(type != 0);

    vec2 pixel_size = 1.0 / vec2(textureSize(tex, 0));

    float sameNeigh = 0;
    int r = 1;
    for(int y = -r; y <= r; y++)
    {
        for (int x = -r; x <= r; x++)
        {
            ivec2 off = ivec2(x, y);
            vec4 pixel = texture(tex, v_tex_coords + off * pixel_size);
            sameNeigh += int(pixel.a * 255 == type);
        }
    }
    float neighTotal = float(4*r*r);

    vec2 pos = floor(v_tex_coords/pixel_size);

    float rng = rand(vec2(floor(v_tex_coords/pixel_size)*pixel_size));
    float dustMin = 0.85;

    float gridBright = (1.0 / 255.0) * int(int(pos.x + pos.y) % 2 == 0);
    vec4 gridCol = vec4(gridBright, gridBright, gridBright, 0);

    // i really dont think this is how i will do this
    FragColor = lerp(col * 0.1, col, sameNeigh / neighTotal) * int(type == 3) + //WATR
                col * ((dustMin - 1) * rng + 1) * int(type == 2) + //DUST
                (col + gridCol) * int(type == 0) +
                col * int(type != 3 && type != 2); // other
}