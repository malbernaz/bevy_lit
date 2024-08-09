#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_lit::view_transformations::frag_coord_to_uv

const BASE: f32 = 255.0;
// Allow for 8 decimal values in seed coordinate values.
const DECIMAL_DIVIDER: f32 = 8.0;
const MAX_POSITION: f32 = (BASE * BASE - 1.0) / DECIMAL_DIVIDER;
const ONE_ZERO: vec2<f32> = vec2(1.0, 0.0);

@group(0) @binding(1) var seed_texture: texture_2d<f32>;
@group(0) @binding(1) var seed_sampler: sampler;

struct Neighbor {
    coord: vec2<f32>,
    color: vec4<f32>,
}

struct NeighborOutput {
    in_range: bool,
    neighbor: Neighbor,
}

fn sample_seed_texture(uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(seed_texture, seed_sampler, uv, 0.0);
}

fn in_range(x: f32, lower: f32, higher: f32) -> bool {
  return clamp(x, lower, higher) == x;
}

// If a left or right neighbor of this shape is on the other side
// of half grey, return it via `neighbor` and return true. Otherwise
// return false.
fn get_x_neighbor(frag_coord: vec2<f32>, self_color: vec4<f32>) -> NeighborOutput {
    let left_coord = frag_coord.xy - ONE_ZERO;
    let right_coord = frag_coord.xy + ONE_ZERO;
    let left = sample_seed_texture(frag_coord_to_uv(left_coord));
    let right = sample_seed_texture(frag_coord_to_uv(right_coord));

    var desired_range = vec2(0.5, 1.0);

    if self_color.r > 0.5 {
        desired_range = vec2(0.0, 0.5)
    }

    if (in_range(left.r, desired_range.x, desired_range.y)) {
        return NeighborOutput(true, Neighbor(left_coord, left));
    }

    if (in_range(right.r, desired_range.x, desired_range.y)) {
        return NeighborOutput(true, Neighbor(right_coord, right));
    }

    return NeighborOutput(false, Neighbor(0.0, 0.0));
}

fn get_y_neighbor(frag_coord: vec2<f32>, self_color: vec4<f32>) -> NeighborOutput {
    let up_coord = frag_coord.xy + ONE_ZERO.yx;
    let down_coord = frag_coord.xy - ONE_ZERO.yx;
    let up = sampleSeedTexture(frag_coord_to_uv(up_coord);
    let down = sampleSeedTexture(frag_coord_to_uv(down_coord));

    var desired_range = vec2(0.5, 1.0);

    if self_color.r > 0.5 {
        desired_range = vec2(0.0, 0.5)
    }

    if (inRange(up.r, desiredRange.x, desiredRange.y)) {
        return NeighborOutput(true, Neighbor(up_coord, up));
    }

    if (inRange(down.r, desiredRange.x, desiredRange.y)) {
        return NeighborOutput(true, Neighbor(down_coord, down));
    }

    return NeighborOutput(false, Neighbor(0.0, 0.0));
}

// Encode an (x, y) coordinate into a vec4. We split each co-ordinate
// across two color channels so that the maximum coordinate value is
// ~65k instead of 255.
//
// We multiply by 10 before encoding and divide when decoding as a
// hacky way to implement floating point numbers
fn encode_position_as_color(frag_coord: vec2<f32>) -> vec4<f32> {
  let screen_coord = floor(frag_coord * DECIMAL_DIVIDER);

  return vec4(
    floor(screen_coord.x / BASE),
    screen_coord.x % BASE,
    floor(screen_coord.y / BASE),
    screen_coord.y % BASE
  ) / BASE;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let self_color = sample_seed_texture(in.uv);

    if (abs(self_color.r - 0.5) < 0.1) {
        // This pixel is pretty close to middle grey which - in an antialiased
        // image of black on white - means that it's distance 0 from our shape.
        // Treat it as a seed.
        return encode_position_as_color(in.position.xy);
    } else {
        // This is a difference from the original paper that lets us generate
        // signed distance fields instead of regular distance fields. In the
        // original paper, each seed location is a real pixel coordinate. In
        // this implemenation, we generate fake seed locations that represent
        // where we _think_ the boundary of the shape is based on antialiasing
        // information.
        //
        // It's easiest to understand how this works with an example. Suppose
        // this pixel is light grey and we find out that we the left neighbor
        // is pretty dark grey. Then 50% grey must happen somewhere between
        // this pixel and our left neighbor. The seed location that we encode
        // in this pixel has an x-coordinate that is interpolated between us
        // and our left neighbor based on our grey values. If this pixel is
        // really close to 50% grey, then it's closer to us. If the neighbor
        // is close to 50% grey, then it's closer to our neighbor.
        //
        // Consider the color value line below. The left edge is the color black
        // and the right edge is white. Supose also that A is the value of our
        // pixel and B is the value of our neighbor.
        //
        //                 A             .5      B
        //  black |--------|--------------|------|--------------| white
        //
        // 50% grey is at the linear interpolation of our coordindate and our
        // neighbors coordinate with t = (.5 - A) / (B - A).
        //
        // This only takes into account up/down/left/right neighbors, so it
        // doesn't always look good for diagonal lines. I think it could be
        // extended pretty well to handle them though.

        // Start out assuming that we don't have any neighbors on the opposite
        // side of 50% grey and that this pixel is not a seed.
        var seed_coord = vec2(MAX_POSITION + 1.0);

        // Account for neighbors on the horizontal axis
        let x_result = get_x_neighbor(in.position.xy, self_color);

        if x_result.in_range {
            let a = self_color.r;
            let b = x_result.neighbor.color.r;
            let lerp_factor = abs(0.5 - a) / abs(b - a);

            seed_coord = mix(in.position.xy, x_result.neighbor.coord, lerp_factor);
        }

        // Account for neighbors on the vertical axis
        let y_result = get_y_neighbor(in.position.xy, self_color);

        if (y_result.in_range) {
            let a = self_color.r;
            let b = y_result.neighbor.color.r;
            let lerp_factor = abs(0.5 - a) / abs(b - a);

            seed_coord.x = min(seed_coord.x, in.position.x);
            seed_coord.y = mix(in.position.y, y_result.neighbor.coord.y, lerp_factor);
        }

        return encode_position_as_color(seed_coord);
    }
}
