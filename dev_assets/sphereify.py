from PIL import Image
import math

def clamp(n, min, max): 
    if n < min: 
        return min
    elif n > max: 
        return max
    else: 
        return n 

def lerp(first, second, v): 
    return first * (1.0 - v) + second * v

def get_average_color(pixels, x_float, y_float):
    lower_x = math.floor(x_float)
    upper_x = math.ceil(x_float)
    lerp_x = x_float - lower_x

    lower_y = math.floor(y_float)
    upper_y = math.ceil(y_float)
    lerp_y = y_float - lower_y

    lxly_color = pixels[lower_x, lower_y]
    lxuy_color = pixels[lower_x, upper_y]

    lx_color = (
        lerp(lxly_color[0], lxuy_color[0], lerp_y),
        lerp(lxly_color[1], lxuy_color[1], lerp_y),
        lerp(lxly_color[2], lxuy_color[2], lerp_y),
    )

    uxly_color = pixels[upper_x, lower_y]
    uxuy_color = pixels[upper_x, upper_y]

    ux_color = (
        lerp(uxly_color[0], uxuy_color[0], lerp_y),
        lerp(uxly_color[1], uxuy_color[1], lerp_y),
        lerp(uxly_color[2], uxuy_color[2], lerp_y),
    )

    return (
        int(lerp(lx_color[0], ux_color[0], lerp_x)),
        int(lerp(lx_color[1], ux_color[1], lerp_x)),
        int(lerp(lx_color[2], ux_color[2], lerp_x)),
    )

target_filename = input("Which file do you wanna sphereify?\n:")
sphereify_shrink = 2.75

image = Image.open(target_filename)

prefix = "sphereified_"

output_filename = prefix + target_filename
print("Output file will be: " + output_filename)

width, height = image.size

new_image = Image.new("RGB", (width, height))

original_pixels = image.load()
new_pixels = new_image.load()

for x in range(width):
    for y in range(height):

        uv_x = (x / width) * 2 - 1
        uv_y = (y / height) * 2 - 1

        old_length = math.sqrt(uv_x * uv_x + uv_y * uv_y)
        new_length = old_length
        if old_length == 0: 
            old_length = 1
            new_length = 1
        elif new_length < 1:
            new_length = math.asin(new_length) / (math.pi / sphereify_shrink)

        new_x = clamp((uv_x / old_length * new_length + 1) / 2 * width, 0, width - 1)
        new_y = clamp((uv_y / old_length * new_length + 1) / 2 * height, 0, height - 1)
                
        color = get_average_color(original_pixels, new_x, new_y)

        new_pixels[x, y] = color

new_image.save(output_filename)

print("Successfully sphereified the image.")